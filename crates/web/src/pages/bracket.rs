use copa2026_core::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
struct PageData {
    matches_by_group: Vec<(GroupCode, Vec<Match>)>,
}

async fn load_page_data() -> Result<PageData, String> {
    let raw = Request::get("/data.json")
        .send()
        .await
        .map_err(|e| format!("fetch: {e}"))?;
    let data: HashMap<String, Vec<Match>> = raw
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

    let group_standings = Signal::derive(move || {
        let all: Vec<Match> = matches_by_group
            .get()
            .iter()
            .flat_map(|(_, m)| m.clone())
            .collect();
        group_standings(&all)
    });

    let bracket = Signal::derive(move || {
        generate_bracket(&group_standings.get())
    });

    let set_score = Callback::new(move |(group_code, match_idx, h, a): (GroupCode, usize, u32, u32)| {
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

    view! {
        <h2>Fase de Grupos</h2>
        <div class="groups-container">
            {move || matches_by_group.get().iter().map(|(code, matches)| {
                let code = code.clone();
                let matches = matches.clone();
                let gs = group_standings.get();
                let set_score = set_score.clone();
                view! {
                    <GroupCard
                        group=code
                        matches=matches
                        group_standings=gs
                        on_set_score=set_score
                    />
                }
            }).collect::<Vec<_>>()}
        </div>
        <BracketTree bracket=Signal::derive(move || bracket.get())/>
    }
}

#[component]
fn GroupCard(
    group: GroupCode,
    matches: Vec<Match>,
    group_standings: Vec<(GroupCode, Vec<Standing>)>,
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
                                <td>{format!("{:+}", s.goal_diff)}</td>
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
                        let has_result = m.result.is_some();
                        let home_goals = m.result.map(|r| r.home_goals).unwrap_or(0);
                        let away_goals = m.result.map(|r| r.away_goals).unwrap_or(0);
                        let group_clone = group.clone();
                        let on_set_score = on_set_score.clone();
                        view! {
                            <tr>
                                <td>{m.home_team.name.clone()}</td>
                                <td class="score-cell">
                                    {if has_result {
                                        view! {
                                            <span class="score">{format!("{} - {}", home_goals, away_goals)}</span>
                                        }.into_any()
                                    } else {
                                        let g = group_clone.clone();
                                        let cb = on_set_score.clone();
                                        view! {
                                            <input type="number" class="score-input" min="0" max="99"
                                                value=home_goals
                                                on:input=move |ev| {
                                                    let h: u32 = event_target_value(&ev).parse().unwrap_or(0);
                                                    cb.run((g.clone(), i, h, away_goals));
                                                }
                                            />
                                            <span class="score-dash">-</span>
                                            <input type="number" class="score-input" min="0" max="99"
                                                value=away_goals
                                                on:input=move |ev| {
                                                    let a: u32 = event_target_value(&ev).parse().unwrap_or(0);
                                                    let h2 = home_goals;
                                                    on_set_score.run((group_clone.clone(), i, h2, a));
                                                }
                                            />
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
fn BracketTree(bracket: Signal<Bracket>) -> impl IntoView {
    view! {
        <h2>Mata-Mata</h2>
        <div class="bracket-tree">
            {move || {
                let b = bracket.get();
                b.rounds.iter().enumerate().map(|(ri, round)| {
                    if round.is_empty() { return view! {}.into_any(); }
                    let round_name = &round[0].round;
                    let match_count = round.len();
                    let mut round_slots = Vec::new();

                    for (_mi, slot) in round.iter().enumerate() {
                        let home = slot.home_team.as_ref()
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| slot.home_label.clone());
                        let away = slot.away_team.as_ref()
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| slot.away_label.clone());

                        let has_result = slot.home_result.is_some() && slot.away_result.is_some();
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

                        let top_gap = if ri == 0 {
                            0
                        } else {
                            let prev_count = b.rounds[ri - 1].len();
                            let cells_per_slot = prev_count / match_count;
                            if cells_per_slot >= 2 {
                                cells_per_slot / 2 * 3 - 1
                            } else {
                                0
                            }
                        };

                        round_slots.push(view! {
                            <div class=extra_class style=format!("margin-top: {}rem; margin-bottom: {}rem", top_gap, top_gap)>
                                <div class="match-teams">
                                    <span class="team home-team">{home}</span>
                                    <span class="team-score">{score_display}</span>
                                    <span class="team away-team">{away}</span>
                                </div>
                            </div>
                        }.into_any());
                    }

                    view! {
                        <div class="bracket-round">
                            <h3>{round_name.clone()}</h3>
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
