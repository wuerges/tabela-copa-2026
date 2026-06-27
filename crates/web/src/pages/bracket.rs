use copa2026_core::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Serialize, Deserialize)]
struct PageData {
    matches_by_group: Vec<(GroupCode, Vec<Match>)>,
}

async fn load_page_data() -> Result<PageData, String> {
    let raw = Request::get("/data.json")
        .send()
        .await
        .map_err(|e| format!("fetch: {e}"))?;
    let data: BTreeMap<String, Vec<Match>> = raw
        .json()
        .await
        .map_err(|e| format!("json: {e}"))?;
    let matches_by_group: Vec<(GroupCode, Vec<Match>)> = GROUP_CODES
        .iter()
        .map(|code| {
            let m = data.get(*code).cloned().unwrap_or_default();
            (GroupCode(code.to_string()), m)
        })
        .collect();
    Ok(PageData { matches_by_group })
}

#[component]
pub fn BracketPage() -> impl IntoView {
    let data = LocalResource::new(|| load_page_data());

    view! {
        <div class="bracket-page">
            <Suspense fallback=|| view! { <p class="loading">Carregando dados...</p> }>
                {move || data.get().map(|result| match result {
                    Ok(bd) => view! { <EditableApp initial=bd/> }.into_any(),
                    Err(e) => view! { <p class="error">Erro: {e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn EditableApp(initial: PageData) -> impl IntoView {
    let matches_by_group: RwSignal<Vec<(GroupCode, Vec<Match>)>> =
        RwSignal::new(initial.matches_by_group);

    let knockout_results: RwSignal<HashMap<String, KnockoutResult>> =
        RwSignal::new(HashMap::new());

    let user_edited: RwSignal<std::collections::HashSet<String>> =
        RwSignal::new(std::collections::HashSet::new());

    let group_standings = Signal::derive(move || {
        let all: Vec<Match> = matches_by_group
            .get()
            .iter()
            .flat_map(|(_, m)| m.clone())
            .collect();
        group_standings(&all)
    });

    let base_bracket = Signal::derive(move || {
        generate_bracket(&group_standings.get())
    });

    let bracket = Signal::derive(move || {
        apply_knockout_results(&base_bracket.get(), &knockout_results.get())
    });

    let clinched = Signal::derive(move || {
        let all: Vec<Match> = matches_by_group
            .get()
            .iter()
            .flat_map(|(_, m)| m.clone())
            .collect();
        clinched_positions(&all)
    });

    let clinched_labels = Signal::derive(move || {
        let c = clinched.get();
        let all: Vec<Match> = matches_by_group
            .get()
            .iter()
            .flat_map(|(_, m)| m.clone())
            .collect();
        let mut labels: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (code, positions) in &c {
            for m in &all {
                if m.home_team.fifa_code == *code {
                    for &pos in positions {
                        labels.insert(format!("{}{}", pos, m.group.0));
                    }
                    break;
                }
                if m.away_team.fifa_code == *code {
                    for &pos in positions {
                        labels.insert(format!("{}{}", pos, m.group.0));
                    }
                    break;
                }
            }
        }
        labels
    });

    let set_score = Callback::new(move |(group_code, match_idx, h, a): (GroupCode, usize, u32, u32)| {
        let id = format!("{}-{}", group_code.0, match_idx + 1);
        user_edited.update(|s| { s.insert(id); });
        matches_by_group.update(|groups| {
            if let Some((_, matches)) = groups.iter_mut().find(|(c, _)| c.0 == group_code.0) {
                if let Some(m) = matches.get_mut(match_idx) {
                    m.result = Some(MatchResult {
                        home_goals: h,
                        away_goals: a,
                    });
                }
            }
        });
    });

    let select_ko_winner = Callback::new(move |(round, match_number, is_home): (String, u32, bool)| {
        knockout_results.update(|results| {
            results.insert(
                format!("{round}-{match_number}"),
                KnockoutResult {
                    round: round.clone(),
                    match_number,
                    winner_is_home: is_home,
                },
            );
        });
    });

    view! {
        <h2>Fase de Grupos</h2>
        <div class="groups-container">
            {move || matches_by_group.get().iter().map(|(code, matches)| {
                let code = code.clone();
                let matches = matches.clone();
                let gs = group_standings.get();
                let set_score = set_score.clone();
                let edited = user_edited.get();
                view! {
                    <GroupCard
                        group=code
                        matches=matches
                        group_standings=gs
                        user_edited=edited
                        on_set_score=set_score
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
        <BracketTree bracket=Signal::derive(move || bracket.get()) clinched_labels=Signal::derive(move || clinched_labels.get()) on_select=select_ko_winner/>
    }
}

#[component]
fn GroupCard(
    group: GroupCode,
    matches: Vec<Match>,
    group_standings: Vec<(GroupCode, Vec<Standing>)>,
    user_edited: std::collections::HashSet<String>,
    on_set_score: Callback<(GroupCode, usize, u32, u32), ()>,
) -> impl IntoView {
    let standings = group_standings
        .iter()
        .find(|(c, _)| c.0 == group.0)
        .map(|(_, s)| s.clone())
        .unwrap_or_default();

    view! {
        <div class="group-card">
            <h3>{"Grupo "} {group.0.clone()}</h3>
            <table class="standings-table">
                <thead>
                    <tr>
                        <th>#</th><th>Time</th><th>P</th><th>J</th><th>V</th><th>E</th><th>D</th>
                        <th>GP</th><th>GC</th><th>SG</th>
                    </tr>
                </thead>
                <tbody>
                    {standings.iter().map(|s| {
                        let class = if s.position <= 2 { "qualified" } else { "" };
                        view! {
                            <tr class=class>
                                <td>{s.position}</td>
                                <td>{s.team.name.clone()}</td>
                                <td>{s.points}</td>
                                <td>{s.played}</td>
                                <td>{s.won}</td>
                                <td>{s.drawn}</td>
                                <td>{s.lost}</td>
                                <td>{s.goals_for}</td>
                                <td>{s.goals_against}</td>
                                <td>{if s.goal_diff == 0 { "0".to_string() } else { format!("{:+}", s.goal_diff) }}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
            <h4>Jogos</h4>
            <table class="matches-table">
                <thead><tr><th>Casa</th><th>Placar</th><th>Fora</th></tr></thead>
                <tbody>
                    {matches.iter().enumerate().map(|(i, m)| {
                        let home_goals = m.result.map(|r| r.home_goals).unwrap_or(0);
                        let away_goals = m.result.map(|r| r.away_goals).unwrap_or(0);
                        let match_id = format!("{}-{}", group.0, i + 1);
                        let is_editable = m.result.is_none() || user_edited.contains(&match_id);
                        let gc1 = group.clone();
                        let gc2 = group.clone();
                        let cb = on_set_score.clone();
                        view! {
                            <tr>
                                <td>{m.home_team.name.clone()}</td>
                                <td class="score-cell">
                                    {if is_editable {
                                        view! {
                                            <input type="number" class="score-input" min="0" max="99"
                                                prop:value=home_goals
                                                on:input=move |ev| {
                                                    let h: u32 = event_target_value(&ev).parse().unwrap_or(0);
                                                    cb.run((gc1.clone(), i, h, away_goals));
                                                }
                                            />
                                            <span class="score-dash">-</span>
                                            <input type="number" class="score-input" min="0" max="99"
                                                prop:value=away_goals
                                                on:input=move |ev| {
                                                    let a: u32 = event_target_value(&ev).parse().unwrap_or(0);
                                                    on_set_score.run((gc2.clone(), i, home_goals, a));
                                                }
                                            />
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="score">{format!("{} - {}", home_goals, away_goals)}</span>
                                        }.into_any()
                                    }}
                                </td>
                                <td>{m.away_team.name.clone()}</td>
                            </tr>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn BracketTree(
    bracket: Signal<Bracket>,
    clinched_labels: Signal<std::collections::HashSet<String>>,
    on_select: Callback<(String, u32, bool), ()>,
) -> impl IntoView {
    view! {
        <h2>Mata-Mata</h2>
        <div class="bracket-legend">
            <span class="legend-item clinched">Posicao garantida</span>
            <span class="legend-item uncertain">Pode mudar</span>
        </div>
        <div class="bracket-tree">
            {move || {
                let b = bracket.get();
                b.rounds.iter().enumerate().map(|(ri, round)| {
                    if round.is_empty() { return view! {}.into_any(); }
                    let round_name = round[0].round.clone();
                    let match_count = round.len();
                    let mut round_slots = Vec::new();

                    for (_mi, slot) in round.iter().enumerate() {
                        let home_name = slot.home_team.as_ref()
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| slot.home_label.clone());
                        let away_name = slot.away_team.as_ref()
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| slot.away_label.clone());

                        let has_result = slot.home_result.is_some() && slot.away_result.is_some();
                        let home_clickable = slot.home_team.is_some();
                        let away_clickable = slot.away_team.is_some();

                        let home_wins = has_result && slot.home_result.unwrap() > slot.away_result.unwrap();
                        let away_wins = has_result && slot.away_result.unwrap() > slot.home_result.unwrap();

                        let score_display = match (slot.home_result, slot.away_result) {
                            (Some(h), Some(a)) => format!("{h} - {a}"),
                            _ => "vs".into(),
                        };

                        let is_empty = slot.home_team.is_none() && slot.away_team.is_none();
                        let extra_class = if has_result {
                            "match-node finished"
                        } else if is_empty {
                            "match-node pending"
                        } else {
                            "match-node determined"
                        };

                        let labels = clinched_labels.get();
                        let home_clinched = ri == 0 && slot.home_team.is_some() && labels.contains(&slot.home_label);
                        let away_clinched = ri == 0 && slot.away_team.is_some() && labels.contains(&slot.away_label);
                        let home_uncertain = ri == 0 && slot.home_team.is_some() && !labels.contains(&slot.home_label) && !slot.home_label.starts_with('W') && !slot.home_label.starts_with('L');
                        let away_uncertain = ri == 0 && slot.away_team.is_some() && !labels.contains(&slot.away_label) && !slot.away_label.starts_with('W') && !slot.away_label.starts_with('L');

                        let top_gap = if ri == 0 {
                            0
                        } else {
                            let prev_count = b.rounds[ri - 1].len();
                            let cells_per_slot = prev_count / match_count;
                            if cells_per_slot >= 2 {
                                cells_per_slot / 2 * 2
                            } else {
                                0
                            }
                        };

                        let round_clone = round_name.clone();
                        let match_num = slot.match_number;
                        let on_select = on_select.clone();

                        let home_class = if home_wins {
                            "team home-team winner"
                        } else if home_clinched {
                            "team home-team clinched"
                        } else if home_uncertain {
                            "team home-team uncertain"
                        } else {
                            "team home-team"
                        };
                        let away_class = if away_wins {
                            "team away-team winner"
                        } else if away_clinched {
                            "team away-team clinched"
                        } else if away_uncertain {
                            "team away-team uncertain"
                        } else {
                            "team away-team"
                        };
                        let clickable_class = " clickable";

                        round_slots.push(view! {
                            <div class=extra_class style=format!("margin-top: {}rem", top_gap)>
                                <div class="match-teams">
                                    {if home_clickable {
                                        let cb = on_select.clone();
                                        let rn = round_clone.clone();
                                        let cb2 = cb.clone();
                                        let rn2 = rn.clone();
                                        view! {
                                            <span class=format!("{home_class}{clickable_class}")
                                                role="button"
                                                tabindex="0"
                                                on:click=move |_| cb.run((rn.clone(), match_num, true))
                                                on:keydown=move |ev| {
                                                    if ev.key() == "Enter" || ev.key() == " " {
                                                        ev.prevent_default();
                                                        cb2.run((rn2.clone(), match_num, true));
                                                    }
                                                }>
                                                {home_name}
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span class=home_class>{home_name}</span> }.into_any()
                                    }}
                                    <span class="team-score">{score_display}</span>
                                    {if away_clickable {
                                        let rn = round_clone.clone();
                                        let os = on_select.clone();
                                        let rn2 = rn.clone();
                                        let os2 = os.clone();
                                        view! {
                                            <span class=format!("{away_class}{clickable_class}")
                                                role="button"
                                                tabindex="0"
                                                on:click=move |_| os.run((rn.clone(), match_num, false))
                                                on:keydown=move |ev| {
                                                    if ev.key() == "Enter" || ev.key() == " " {
                                                        ev.prevent_default();
                                                        os2.run((rn2.clone(), match_num, false));
                                                    }
                                                }>
                                                {away_name}
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! { <span class=away_class>{away_name}</span> }.into_any()
                                    }}
                                </div>
                            </div>
                        }.into_any());
                    }

                    view! {
                        <div class="bracket-round">
                            <h3>{round_name}</h3>
                            <div class="bracket-matches">
                                {round_slots}
                            </div>
                        </div>
                    }.into_any()
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
