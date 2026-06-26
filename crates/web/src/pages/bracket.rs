use copa2026_core::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
struct PageData {
    group_standings: Vec<(GroupCode, Vec<Standing>)>,
    bracket: Bracket,
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

    let all_matches: Vec<Match> = data.values().flatten().cloned().collect();
    let gs = group_standings(&all_matches);
    let bracket = generate_bracket(&gs);

    let matches_by_group: Vec<(GroupCode, Vec<Match>)> = GROUP_CODES
        .iter()
        .map(|code| {
            let m = data.get(*code).cloned().unwrap_or_default();
            (GroupCode(code.to_string()), m)
        })
        .collect();

    Ok(PageData {
        group_standings: gs,
        bracket,
        matches_by_group,
    })
}

#[component]
pub fn BracketPage() -> impl IntoView {
    let data = LocalResource::new(|| load_page_data());

    view! {
        <div class="bracket-page">
            <Suspense fallback=|| view! { <p class="loading">Carregando dados...</p> }>
                {move || data.get().map(|result| match result {
                    Ok(bd) => view! {
                        <GroupList data=bd/>
                    }.into_any(),
                    Err(e) => view! { <p class="error">Erro: {e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn GroupList(data: PageData) -> impl IntoView {
    view! {
        <h2>Fase de Grupos</h2>
        <div class="groups-container">
            {data.matches_by_group.iter().map(|(code, matches)| {
                let code = code.clone();
                let matches = matches.clone();
                let gs = data.group_standings.clone();
                view! {
                    <GroupCard group=code matches=matches group_standings=gs/>
                }
            }).collect::<Vec<_>>()}
        </div>
        <BracketView bracket=data.bracket/>
    }
}

#[component]
fn GroupCard(
    group: GroupCode,
    matches: Vec<Match>,
    group_standings: Vec<(GroupCode, Vec<Standing>)>,
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
                    {matches.iter().map(|m| {
                        let score = match &m.result {
                            Some(r) => format!("{} - {}", r.home_goals, r.away_goals),
                            None => "? - ?".to_string(),
                        };
                        view! {
                            <tr>
                                <td>{m.home_team.name.clone()}</td>
                                <td class="score">{score}</td>
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
fn BracketView(bracket: Bracket) -> impl IntoView {
    view! {
        <h2>Mata-Mata</h2>
        <div class="bracket-container">
            {bracket.rounds.iter().map(|round| {
                let round_name = if round.is_empty() { "" } else { &round[0].round };
                view! {
                    <div class="round">
                        <h3>{round_name.to_string()}</h3>
                        {round.iter().map(|slot| {
                            let home = slot.home_team.as_ref().map(|t| t.name.clone())
                                .unwrap_or_else(|| slot.home_label.clone());
                            let away = slot.away_team.as_ref().map(|t| t.name.clone())
                                .unwrap_or_else(|| slot.away_label.clone());
                            let score = match (slot.home_result, slot.away_result) {
                                (Some(h), Some(a)) => format!("{h} - {a}"),
                                _ => "vs".to_string(),
                            };
                            let filled = slot.home_team.is_some() || slot.away_team.is_some();
                            let class = if filled { "match-slot filled" } else { "match-slot" };
                            view! {
                                <div class=class>
                                    <span class="home">{home}</span>
                                    <span class="score">{score}</span>
                                    <span class="away">{away}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
