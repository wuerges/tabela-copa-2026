use copa2026_core::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use std::collections::HashMap;

async fn load_simulation() -> Result<ThirdPlaceSimulation, String> {
    let raw = Request::get("/data.json")
        .send()
        .await
        .map_err(|e| format!("fetch: {e}"))?;

    let data: HashMap<String, Vec<Match>> = raw
        .json()
        .await
        .map_err(|e| format!("json: {e}"))?;

    let all_matches: Vec<Match> = data.values().flatten().cloned().collect();
    Ok(simulate_guaranteed_thirds(&all_matches))
}

#[component]
pub fn GuaranteedThirdsPage() -> impl IntoView {
    let data = LocalResource::new(|| load_simulation());

    view! {
        <div class="guaranteed-thirds-page">
            <h2>Probabilidade de Classificacao</h2>
            <Suspense fallback=|| view! { <p class="loading">Simulando cenarios...</p> }>
                {move || data.get().map(|result| match result {
                    Ok(sim) => view! {
                        <div class="simulation-info">
                            <p>{format!("{} jogos pendentes, {} cenarios ({}).",
                                sim.unplayed_matches, sim.total_scenarios, sim.method)}</p>
                        </div>
                        <table class="prob-table">
                            <thead>
                                <tr>
                                    <th>Time</th><th>Gr</th><th>1o%</th><th>2o%</th>
                                    <th>3o%</th><th>Total%</th><th>Pts</th><th>GD</th>
                                </tr>
                            </thead>
                            <tbody>
                                {sim.teams.iter().map(|tc| {
                                    let class = if tc.total_qualification_pct == 100.0 {
                                        "guaranteed"
                                    } else if tc.total_qualification_pct == 0.0 {
                                        "eliminated"
                                    } else {
                                        ""
                                    };
                                    view! {
                                        <tr class=class>
                                            <td>{tc.team.name.clone()}</td>
                                            <td>{tc.group.0.clone()}</td>
                                            <td>{format!("{:.1}", tc.first_pct)}</td>
                                            <td>{format!("{:.1}", tc.second_pct)}</td>
                                            <td>{format!("{:.1}", tc.third_qualified_pct)}</td>
                                            <td class="total">{format!("{:.1}", tc.total_qualification_pct)}</td>
                                            <td>{tc.points}</td>
                                            <td>{format!("{:+}", tc.goal_diff)}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                        <p class="summary">{
                            let guaranteed = sim.teams.iter().filter(|t| t.total_qualification_pct == 100.0).count();
                            let eliminated = sim.teams.iter().filter(|t| t.total_qualification_pct == 0.0).count();
                            let uncertain = sim.teams.len() - guaranteed - eliminated;
                            view! {
                                <span class="badge guaranteed-badge">{"Garantidos: "}{guaranteed}</span>
                                <span class="badge uncertain-badge">{"Incertos: "}{uncertain}</span>
                                <span class="badge eliminated-badge">{"Desqualificados: "}{eliminated}</span>
                            }
                        }</p>
                    }.into_any(),
                    Err(e) => view! { <p class="error">Erro: {e}</p> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
