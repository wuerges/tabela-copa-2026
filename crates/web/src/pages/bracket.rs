use copa2026_core::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
struct PageData {
    matches_by_group: Vec<(GroupCode, Vec<Match>)>,
    knockout_matches: HashMap<String, KnockoutMatch>,
}

async fn load_page_data() -> Result<PageData, String> {
    let raw = Request::get("/data.json")
        .send()
        .await
        .map_err(|e| format!("fetch: {e}"))?;
    let data: WorldCupData = raw
        .json()
        .await
        .map_err(|e| format!("json: {e}"))?;
    let matches_by_group: Vec<(GroupCode, Vec<Match>)> = GROUP_CODES
        .iter()
        .map(|code| {
            let m = data.groups.get(*code).cloned().unwrap_or_default();
            (GroupCode(code.to_string()), m)
        })
        .collect();
    let knockout_matches: HashMap<String, KnockoutMatch> = data.knockout
        .into_iter()
        .map(|m| (format!("{}-{}", m.round, m.match_number), m))
        .collect();
    Ok(PageData { matches_by_group, knockout_matches })
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

    let knockout_results: RwSignal<HashMap<String, KnockoutMatch>> =
        RwSignal::new(initial.knockout_matches);

    let user_edited: RwSignal<std::collections::HashSet<String>> =
        RwSignal::new(std::collections::HashSet::new());

    let group_standings = Signal::derive(move || {
        let all: Vec<Match> = matches_by_group.get().iter()
            .flat_map(|(_, m)| m.clone()).collect();
        group_standings(&all)
    });

    let base_bracket = Signal::derive(move || {
        generate_bracket(&group_standings.get())
    });

    let bracket = Signal::derive(move || {
        apply_knockout_results(&base_bracket.get(), &knockout_results.get())
    });

    let clinched = Signal::derive(move || {
        let all: Vec<Match> = matches_by_group.get().iter()
            .flat_map(|(_, m)| m.clone()).collect();
        clinched_positions(&all)
    });

    let clinched_labels = Signal::derive(move || {
        let c = clinched.get();
        let all: Vec<Match> = matches_by_group.get().iter()
            .flat_map(|(_, m)| m.clone()).collect();
        let mut labels: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (code, positions) in &c {
            for m in &all {
                if m.home_team.fifa_code == *code {
                    for &pos in positions { labels.insert(format!("{}{}", pos, m.group.0)); }
                    break;
                }
                if m.away_team.fifa_code == *code {
                    for &pos in positions { labels.insert(format!("{}{}", pos, m.group.0)); }
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
                    m.result = Some(MatchResult { home_goals: h, away_goals: a });
                }
            }
        });
    });

    let select_ko_winner = Callback::new(move |(round, match_number, is_home): (String, u32, bool)| {
        knockout_results.update(|results| {
            results.insert(
                format!("{round}-{match_number}"),
                KnockoutMatch {
                    round: round.clone(),
                    match_number,
                    home_goals: Some(if is_home { 1 } else { 0 }),
                    away_goals: Some(if is_home { 0 } else { 1 }),
                    winner_is_home: None,
                },
            );
        });
    });

    view! {
        <BracketTree
            bracket=Signal::derive(move || bracket.get())
            clinched_labels=Signal::derive(move || clinched_labels.get())
            on_select=select_ko_winner
        />
        <h2>Fase de Grupos</h2>
        <div class="groups-container">
            {move || matches_by_group.get().iter().map(|(code, matches)| {
                let code = code.clone();
                let matches = matches.clone();
                let gs = group_standings.get();
                let set_score = set_score.clone();
                let edited = user_edited.get();
                view! {
                    <GroupCard group=code matches=matches group_standings=gs
                        user_edited=edited on_set_score=set_score />
                }
            }).collect::<Vec<_>>()}
        </div>
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
    let standings = group_standings.iter()
        .find(|(c, _)| c.0 == group.0)
        .map(|(_, s)| s.clone()).unwrap_or_default();

    view! {
        <div class="group-card">
            <h3>{"Grupo "} {group.0.clone()}</h3>
            <table class="standings-table">
                <thead><tr>
                    <th>#</th><th>Time</th><th>P</th><th>J</th><th>V</th><th>E</th><th>D</th>
                    <th>GP</th><th>GC</th><th>SG</th>
                </tr></thead>
                <tbody>
                    {standings.iter().map(|s| {
                        let class = if s.position <= 2 { "qualified" } else { "" };
                        view! {
                            <tr class=class>
                                <td>{s.position}</td><td>{s.team.name.clone()}</td>
                                <td>{s.points}</td><td>{s.played}</td><td>{s.won}</td>
                                <td>{s.drawn}</td><td>{s.lost}</td><td>{s.goals_for}</td>
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
                        let gc1 = group.clone(); let gc2 = group.clone();
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
                                                } />
                                            <span class="score-dash">-</span>
                                            <input type="number" class="score-input" min="0" max="99"
                                                prop:value=away_goals
                                                on:input=move |ev| {
                                                    let a: u32 = event_target_value(&ev).parse().unwrap_or(0);
                                                    on_set_score.run((gc2.clone(), i, home_goals, a));
                                                } />
                                        }.into_any()
                                    } else {
                                        view! { <span class="score">{format!("{} - {}", home_goals, away_goals)}</span> }.into_any()
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

// ══════════════════════════════════════════════════════════════════════
// SVG Bracket Rendering
// ══════════════════════════════════════════════════════════════════════

const MATCH_W: f64 = 150.0;
const MATCH_H: f64 = 75.0;
const SLOT_H: f64 = 28.0;
const SLOT_TOP: f64 = 8.0;
const SLOT_GAP: f64 = 3.0;

/// Returns (x, y) for a match box top-left corner, or None for center-only matches.
fn match_position(match_num: u32) -> (f64, f64) {
    match match_num {
        // ── Left R32 (x=80) ──────────────────────────
        2  => (80.0, 150.0),
        5  => (80.0, 250.0),
        1  => (80.0, 350.0),
        3  => (80.0, 450.0),
        11 => (80.0, 550.0),
        12 => (80.0, 650.0),
        9  => (80.0, 750.0),
        10 => (80.0, 850.0),
        // ── Left R16 (x=260) ─────────────────────────
        17 => (260.0, 200.0),
        18 => (260.0, 400.0),
        21 => (260.0, 600.0),
        22 => (260.0, 800.0),
        // ── Left QF (x=440) ──────────────────────────
        25 => (440.0, 300.0),
        26 => (440.0, 700.0),
        // ── Left SF (x=620) ──────────────────────────
        29 => (620.0, 500.0),
        // ── Right SF (x=830) ─────────────────────────
        30 => (830.0, 500.0),
        // ── Right QF (x=1010) ────────────────────────
        27 => (1010.0, 300.0),
        28 => (1010.0, 700.0),
        // ── Right R16 (x=1190) ───────────────────────
        19 => (1190.0, 200.0),
        20 => (1190.0, 400.0),
        23 => (1190.0, 600.0),
        24 => (1190.0, 800.0),
        // ── Right R32 (x=1370) ───────────────────────
        4  => (1370.0, 150.0),
        6  => (1370.0, 250.0),
        7  => (1370.0, 350.0),
        8  => (1370.0, 450.0),
        14 => (1370.0, 550.0),
        16 => (1370.0, 650.0),
        13 => (1370.0, 750.0),
        15 => (1370.0, 850.0),
        // ── Center ────────────────────────────────────
        32 => (725.0, 390.0), // Final
        31 => (725.0, 650.0), // 3rd Place
        _ => (0.0, 0.0),
    }
}

fn match_center(match_num: u32) -> (f64, f64) {
    let (x, y) = match_position(match_num);
    (x + MATCH_W / 2.0, y + MATCH_H / 2.0)
}

/// Returns (date_str, venue_str) for each match.
fn match_info(match_num: u32) -> (&'static str, &'static str) {
    match match_num {
        1  => ("28 Junho - 16:00", "EUA, Los Angeles"),
        2  => ("29 Junho - 17:30", "EUA, Boston"),
        3  => ("30 Junho - 22:00", "México, Guadalupe"),
        4  => ("29 Junho - 14:00", "EUA, Houston"),
        5  => ("30 Junho - 18:00", "EUA, NY/NJ"),
        6  => ("30 Junho - 14:00", "EUA, Dallas"),
        7  => ("1 Julho - 22:00", "México, C. México"),
        8  => ("1 Julho - 13:00", "EUA, Atlanta"),
        9  => ("2 Julho - 21:00", "EUA, Santa Clara"),
        10 => ("1 Julho - 17:00", "EUA, Seattle"),
        11 => ("3 Julho - 20:00", "Canadá, Toronto"),
        12 => ("2 Julho - 16:00", "EUA, Los Angeles"),
        13 => ("3 Julho - 00:00", "Canadá, Vancouver"),
        14 => ("3 Julho - 19:00", "EUA, Miami"),
        15 => ("4 Julho - 22:30", "EUA, Kansas City"),
        16 => ("3 Julho - 15:00", "EUA, Dallas"),
        17 => ("4 Julho - 18:00", "EUA, Philadelphia"),
        18 => ("4 Julho - 14:00", "EUA, Houston"),
        19 => ("5 Julho - 17:00", "EUA, NY/NJ"),
        20 => ("6 Julho - 21:00", "México, C. México"),
        21 => ("6 Julho - 16:00", "EUA, Dallas"),
        22 => ("7 Julho - 21:00", "EUA, Seattle"),
        23 => ("7 Julho - 13:00", "EUA, Atlanta"),
        24 => ("7 Julho - 17:00", "Canadá, Vancouver"),
        25 => ("9 Julho - 17:00", "EUA, Boston"),
        26 => ("10 Julho - 16:00", "EUA, Los Angeles"),
        27 => ("11 Julho - 18:00", "EUA, Miami"),
        28 => ("12 Julho - 22:00", "EUA, Kansas City"),
        29 => ("14 Julho - 16:00", "EUA, Dallas"),
        30 => ("15 Julho - 16:00", "EUA, Atlanta"),
        31 => ("18 Julho - 18:00", "EUA, Miami"),
        32 => ("19 Julho - 16:00", "EUA, NY/New Jersey"),
        _ => ("", ""),
    }
}

/// Generate connector line paths for the bracket tree.
fn connector_paths() -> Vec<String> {
    let mut paths = Vec::new();

    // Helper: connect two source matches to one target match (all left or all right)
    let mut pair_to_target = |m_a: u32, m_b: u32, m_tgt: u32| {
        let (sx, _) = match_position(m_a); // source x
        let (tx, _) = match_position(m_tgt); // target x
        let from_x = sx + MATCH_W;
        let to_x = tx;
        let mid_x = (from_x + to_x) / 2.0;
        let tgt_cy = match_center(m_tgt).1;

        for &m in &[m_a, m_b] {
            let (_, cy) = match_center(m);
            paths.push(format!(
                "M {} {} H {} V {} H {}",
                from_x, cy, mid_x, tgt_cy, to_x
            ));
        }
    };

    // Left side R32 → R16
    pair_to_target(2, 5, 17);
    pair_to_target(1, 3, 18);
    pair_to_target(11, 12, 21);
    pair_to_target(9, 10, 22);

    // Left side R16 → QF
    pair_to_target(17, 18, 25);
    pair_to_target(21, 22, 26);

    // Left side QF → SF
    pair_to_target(25, 26, 29);

    // Right side R32 → R16
    pair_to_target(4, 6, 19);
    pair_to_target(7, 8, 20);
    pair_to_target(14, 16, 23);
    pair_to_target(13, 15, 24);

    // Right side R16 → QF
    pair_to_target(19, 20, 27);
    pair_to_target(23, 24, 28);

    // Right side QF → SF
    pair_to_target(27, 28, 30);

    // SF → Center (Final / 3rd Place)
    let (sf29_x, _) = match_position(29);
    let sf29_right = sf29_x + MATCH_W;
    let sf29_cy = match_center(29).1;
    paths.push(format!("M {} {} H 790", sf29_right, sf29_cy));

    let (sf30_x, _) = match_position(30);
    let sf30_cy = match_center(30).1;
    paths.push(format!("M {} {} H 810", sf30_x, sf30_cy));

    paths
}

fn team_display(slot: &BracketSlot, is_home: bool) -> (String, bool) {
    let team = if is_home { &slot.home_team } else { &slot.away_team };
    let label = if is_home { &slot.home_label } else { &slot.away_label };
    let name = team.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| label.clone());
    let has_team = team.is_some();
    (name, has_team)
}

/// Render one match box as SVG elements.
fn svg_match(
    slot: BracketSlot,
    clinched_labels: std::collections::HashSet<String>,
    on_select: Callback<(String, u32, bool), ()>,
) -> impl IntoView {
    let (x, y) = match_position(slot.match_number);
    let cx = x + MATCH_W / 2.0;
    let (date, _venue) = match_info(slot.match_number);

    let has_result = slot.home_result.is_some() && slot.away_result.is_some();
    let is_empty = slot.home_team.is_none() && slot.away_team.is_none();
    let home_wins = has_result && slot.home_result.unwrap() > slot.away_result.unwrap();
    let away_wins = has_result && slot.away_result.unwrap() > slot.home_result.unwrap();

    let (home_name, home_has_team) = team_display(&slot, true);
    let (away_name, away_has_team) = team_display(&slot, false);

    let is_r32 = slot.round == "Round of 32";
    let home_clinched = is_r32 && home_has_team && clinched_labels.contains(&slot.home_label);
    let away_clinched = is_r32 && away_has_team && clinched_labels.contains(&slot.away_label);
    let home_uncertain = is_r32 && home_has_team && !clinched_labels.contains(&slot.home_label)
        && !slot.home_label.starts_with('W') && !slot.home_label.starts_with('L');
    let away_uncertain = is_r32 && away_has_team && !clinched_labels.contains(&slot.away_label)
        && !slot.away_label.starts_with('W') && !slot.away_label.starts_with('L');

    // Match box styling
    let (box_fill, box_stroke, box_opacity) = if has_result {
        ("#052e16", "#166534", "1")
    } else if is_empty {
        ("#111827", "#1e293b", "0.35")
    } else {
        ("#0c1929", "#1e40af", "1")
    };

    let home_color = if home_wins { "#4ade80" }
        else if home_clinched { "#4ade80" }
        else if home_uncertain { "#fbbf24" }
        else if has_result && !home_wins { "#475569" }
        else { "#cbd5e1" };
    let away_color = if away_wins { "#4ade80" }
        else if away_clinched { "#4ade80" }
        else if away_uncertain { "#fbbf24" }
        else if has_result && !away_wins { "#475569" }
        else { "#cbd5e1" };

    let home_weight = if home_wins { "700" } else { "400" };
    let away_weight = if away_wins { "700" } else { "400" };
    let home_style = if home_uncertain { "font-style: italic" } else { "" };
    let away_style = if away_uncertain { "font-style: italic" } else { "" };

    let slot1_y = y + SLOT_TOP;
    let slot2_y = slot1_y + SLOT_H + SLOT_GAP;
    let name1_y = slot1_y + SLOT_H / 2.0 + 1.0;
    let name2_y = slot2_y + SLOT_H / 2.0 + 1.0;
    let date_y = y - 5.0;

    let round_name = slot.round.clone();
    let match_num = slot.match_number;
    let os = on_select.clone();

    view! {
        <g>
            <rect x=x y=y width=MATCH_W height=MATCH_H rx="6"
                fill=box_fill stroke=box_stroke stroke-width="1.5" opacity=box_opacity />
            <rect x=x+5.0 y=slot1_y width=MATCH_W-10.0 height=SLOT_H rx="4"
                fill="#0f172a" stroke="#334155" stroke-width="1" />
            <rect x=x+5.0 y=slot2_y width=MATCH_W-10.0 height=SLOT_H rx="4"
                fill="#0f172a" stroke="#334155" stroke-width="1" />

            {if home_has_team {
                let rn = round_name.clone(); let c = os.clone();
                view! {
                    <text x=cx y=name1_y fill=home_color font-weight=home_weight font-size="13"
                        text-anchor="middle" dominant-baseline="central"
                        style=format!("cursor:pointer;{}", home_style)
                        on:click=move |_| c.run((rn.clone(), match_num, true))>{home_name}</text>
                }.into_any()
            } else {
                view! { <text x=cx y=name1_y fill=home_color font-size="13"
                    text-anchor="middle" dominant-baseline="central"
                    style=home_style>{home_name}</text> }.into_any()
            }}

            {if away_has_team {
                let rn = round_name.clone();
                view! {
                    <text x=cx y=name2_y fill=away_color font-weight=away_weight font-size="13"
                        text-anchor="middle" dominant-baseline="central"
                        style=format!("cursor:pointer;{}", away_style)
                        on:click=move |_| os.run((rn.clone(), match_num, false))>{away_name}</text>
                }.into_any()
            } else {
                view! { <text x=cx y=name2_y fill=away_color font-size="13"
                    text-anchor="middle" dominant-baseline="central"
                    style=away_style>{away_name}</text> }.into_any()
            }}

            <text x=cx y=date_y fill="#38bdf8" font-size="10" font-weight="700"
                text-anchor="middle" dominant-baseline="central">{date}</text>
        </g>
    }.into_any()
}

#[component]
fn BracketTree(
    bracket: Signal<Bracket>,
    clinched_labels: Signal<std::collections::HashSet<String>>,
    on_select: Callback<(String, u32, bool), ()>,
) -> impl IntoView {
    view! {
        <h2>Mata-Mata</h2>
        <div class="bracket-section">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="30 85 1540 880"
                width="100%" style="min-width: 900px; display: block;">
                // Column headers
                <text x="155" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">SEGUNDA FASE</text>
                <text x="335" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">OITAVAS</text>
                <text x="515" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">QUARTAS</text>
                <text x="695" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">SEMIFINAL</text>
                <text x="905" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">SEMIFINAL</text>
                <text x="1085" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">QUARTAS</text>
                <text x="1265" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">OITAVAS</text>
                <text x="1445" y="110" fill="#64748b" font-size="11" font-weight="700"
                    text-anchor="middle">SEGUNDA FASE</text>

                // Connector lines
                {connector_paths().into_iter().map(|d| {
                    view! { <path d=d fill="none" stroke="#475569" stroke-width="2"
                        stroke-linecap="round" stroke-linejoin="round" /> }
                }).collect::<Vec<_>>()}

                // Match boxes
                {move || {
                    let b = bracket.get();
                    let labels = clinched_labels.get();
                    let cb = on_select.clone();

                    let all_slots: Vec<BracketSlot> = b.rounds.iter()
                        .flat_map(|r| r.iter().cloned())
                        .collect();

                    all_slots.into_iter().map(|slot| {
                        svg_match(slot, labels.clone(), cb.clone())
                    }).collect::<Vec<_>>()
                }}

                // Final & 3rd labels
                <text x="800" y="375" fill="#38bdf8" font-size="20" font-weight="800"
                    text-anchor="middle">FINAL</text>
                <text x="800" y="635" fill="#64748b" font-size="13" font-weight="700"
                    text-anchor="middle">3º LUGAR</text>
            </svg>
            <div class="bracket-legend">
                <span class="legend-item"><span class="legend-dot clinched"></span>Posição garantida</span>
                <span class="legend-item"><span class="legend-dot uncertain"></span>Pode mudar</span>
                <span class="legend-item"><span class="legend-dot finished"></span>Resultado</span>
                <span class="legend-item"><span class="legend-dot determined"></span>Definido</span>
                <span class="legend-item"><span class="legend-dot pending"></span>Pendente</span>
            </div>
        </div>
    }
}
