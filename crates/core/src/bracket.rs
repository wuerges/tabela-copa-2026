use crate::models::*;
use crate::standings::rank_third_places;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BracketSlot {
    pub round: String,
    pub match_number: u32,
    pub home_label: String,
    pub away_label: String,
    pub home_team: Option<Team>,
    pub away_team: Option<Team>,
    pub home_result: Option<u32>,
    pub away_result: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bracket {
    pub rounds: Vec<Vec<BracketSlot>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnockoutResult {
    pub round: String,
    pub match_number: u32,
    pub winner_is_home: bool,
}

/// Third-place combination table: each entry is (match_index, allowed_groups).
/// Match indices are 0-based (M1=0, M2=1, ..., M16=15).
/// The 8 matches that host a third-place team: M2, M5, M7, M8, M9, M10, M13, M15.
const THIRD_PLACE_SLOTS: &[(usize, &[&str])] = &[
    (1,  &["A", "B", "C", "D", "F"]),       // M2
    (4,  &["C", "D", "F", "G", "H"]),       // M5
    (6,  &["C", "E", "F", "H", "I"]),       // M7
    (7,  &["E", "H", "I", "J", "K"]),       // M8
    (8,  &["B", "E", "F", "I", "J"]),       // M9
    (9,  &["A", "E", "H", "I", "J"]),       // M10
    (12, &["E", "F", "G", "I", "J"]),       // M13
    (14, &["D", "E", "I", "J", "L"]),       // M15
];

/// Priority order for third-place group assignment.
/// Groups earlier in this list are tried first during backtracking,
/// which biases the algorithm toward the FIFA-canonical assignment.
/// This order was derived from the known Wikipedia/CBS assignment:
///   D→M2, F→M5, E→M7, K→M8, B→M9, I→M10, J→M13, L→M15
const THIRD_PLACE_PRIORITY: &[&str] = &["D", "F", "E", "K", "B", "I", "J", "L", "A", "C", "G", "H"];

/// Given the 8 qualifying third-place groups (top 8 of 12 by points/GD/GF),
/// find the unique valid assignment to the 8 third-place match slots using
/// backtracking DFS. Groups are tried in THIRD_PLACE_PRIORITY order so the
/// result matches the official FIFA combination table.
fn match_third_places(qualifying: &[String]) -> HashMap<usize, GroupCode> {
    // Edge case: fewer than 8 qualifying groups (e.g., empty data).
    // Return empty assignment; third-place slots will show "TBD".
    if qualifying.len() < 8 {
        return HashMap::new();
    }

    // Sort qualifying groups by their position in the priority list
    let mut sorted: Vec<String> = qualifying.to_vec();
    sorted.sort_by_key(|g| {
        THIRD_PLACE_PRIORITY
            .iter()
            .position(|&p| p == g.as_str())
            .unwrap_or(usize::MAX)
    });

    let mut assignment = HashMap::new();
    let mut used: HashSet<String> = HashSet::new();
    let found = backtrack_third_places(&sorted, 0, &mut used, &mut assignment);
    debug_assert!(found, "No valid third-place assignment found — check combination table");
    assignment
}

fn backtrack_third_places(
    groups: &[String],
    slot_idx: usize,
    used: &mut HashSet<String>,
    assignment: &mut HashMap<usize, GroupCode>,
) -> bool {
    if slot_idx >= THIRD_PLACE_SLOTS.len() {
        return used.len() == groups.len();
    }
    let (match_idx, allowed) = THIRD_PLACE_SLOTS[slot_idx];
    for group in groups {
        if used.contains(group) {
            continue;
        }
        if !allowed.contains(&group.as_str()) {
            continue;
        }
        used.insert(group.clone());
        assignment.insert(match_idx, GroupCode(group.clone()));
        if backtrack_third_places(groups, slot_idx + 1, used, assignment) {
            return true;
        }
        assignment.remove(&match_idx);
        used.remove(group);
    }
    false
}

/// Returns which visual half of the bracket a match belongs to.
/// The bracket is split into left and right halves that converge at the Final.
/// Left half (R32→SF): matches 1-3,5,9-12, 17-18,21-22, 25-26, 29
/// Right half (R32→SF): matches 4,6-8,13-16, 19-20,23-24, 27-28, 30
/// Center: matches 31 (3rd Place) and 32 (Final)
pub fn bracket_side(match_number: u32) -> &'static str {
    match match_number {
        // Round of 32
        1 | 2 | 3 | 5 | 9 | 10 | 11 | 12 => "left",
        4 | 6 | 7 | 8 | 13 | 14 | 15 | 16 => "right",
        // Round of 16
        17 | 18 | 21 | 22 => "left",
        19 | 20 | 23 | 24 => "right",
        // Quarter-finals
        25 | 26 => "left",
        27 | 28 => "right",
        // Semi-finals
        29 => "left",
        30 => "right",
        // Final & Third Place
        31 | 32 => "center",
        _ => "left",
    }
}

fn resolve_slot(
    label: &str,
    team_map: &HashMap<String, Team>,
) -> Option<Team> {
    if label == "TBD" || label.is_empty() {
        return None;
    }
    team_map.get(label).cloned()
}

/// Build the 16 R32 label pairs. Third-place slots are filled with the
/// group-specific label (e.g. "3D" = 3rd-place team from Group D) as
/// determined by the combination table.
fn build_r32_pairings(
    third_assignments: &HashMap<usize, GroupCode>,
) -> Vec<(String, String)> {
    /// Helper: get the third-place group for a given match index (0-based).
    /// Returns "TBD" when the combination table has no assignment (e.g., empty data).
    fn third_label(assignments: &HashMap<usize, GroupCode>, idx: usize) -> String {
        match assignments.get(&idx) {
            Some(gc) => format!("3{}", gc.0),
            None => "TBD".to_string(),
        }
    }

    vec![
        // M1: 2A vs 2B
        ("2A".to_string(), "2B".to_string()),
        // M2: 1E vs 3rd (A/B/C/D/F)
        ("1E".to_string(), third_label(third_assignments, 1)),
        // M3: 1F vs 2C
        ("1F".to_string(), "2C".to_string()),
        // M4: 1C vs 2F
        ("1C".to_string(), "2F".to_string()),
        // M5: 1I vs 3rd (C/D/F/G/H)
        ("1I".to_string(), third_label(third_assignments, 4)),
        // M6: 2E vs 2I
        ("2E".to_string(), "2I".to_string()),
        // M7: 1A vs 3rd (C/E/F/H/I)
        ("1A".to_string(), third_label(third_assignments, 6)),
        // M8: 1L vs 3rd (E/H/I/J/K)
        ("1L".to_string(), third_label(third_assignments, 7)),
        // M9: 1D vs 3rd (B/E/F/I/J)
        ("1D".to_string(), third_label(third_assignments, 8)),
        // M10: 1G vs 3rd (A/E/H/I/J)
        ("1G".to_string(), third_label(third_assignments, 9)),
        // M11: 2K vs 2L
        ("2K".to_string(), "2L".to_string()),
        // M12: 1H vs 2J
        ("1H".to_string(), "2J".to_string()),
        // M13: 1B vs 3rd (E/F/G/I/J)
        ("1B".to_string(), third_label(third_assignments, 12)),
        // M14: 1J vs 2H
        ("1J".to_string(), "2H".to_string()),
        // M15: 1K vs 3rd (D/E/I/J/L)
        ("1K".to_string(), third_label(third_assignments, 14)),
        // M16: 2D vs 2G
        ("2D".to_string(), "2G".to_string()),
    ]
}

pub fn generate_bracket(group_standings: &[(GroupCode, Vec<Standing>)]) -> Bracket {
    let mut team_map: HashMap<String, Team> = HashMap::new();
    for (code, standings) in group_standings {
        for s in standings {
            let key = format!("{}{}", s.position, code.0);
            team_map.insert(key, s.team.clone());
        }
    }
    let thirds = rank_third_places(group_standings);

    // Determine the 8 qualifying third-place groups (top 8 of 12)
    let qualifying_groups: Vec<String> = thirds
        .iter()
        .take(8)
        .map(|(code, _)| code.0.clone())
        .collect();

    // Run the combination table to assign groups to match slots
    let third_assignments = match_third_places(&qualifying_groups);

    // Build R32 pairings with group-specific third-place labels
    let r32_pairings = build_r32_pairings(&third_assignments);

    let r32: Vec<BracketSlot> = r32_pairings
        .iter()
        .enumerate()
        .map(|(i, (home_label, away_label))| BracketSlot {
            round: "Round of 32".into(),
            match_number: (i + 1) as u32,
            home_label: home_label.clone(),
            away_label: away_label.clone(),
            home_team: resolve_slot(home_label, &team_map),
            away_team: resolve_slot(away_label, &team_map),
            home_result: None,
            away_result: None,
        })
        .collect();

    // R16 pairings (FIFA 2026 official):
    // M17: W2 vs W5   M18: W1 vs W3   M19: W4 vs W6   M20: W7 vs W8
    // M21: W11 vs W12 M22: W9 vs W10  M23: W14 vs W16 M24: W13 vs W15
    let r16_specs: &[(u32, u32)] = &[
        (2, 5), (1, 3), (4, 6), (7, 8),
        (11, 12), (9, 10), (14, 16), (13, 15),
    ];
    let r16: Vec<BracketSlot> = r16_specs
        .iter()
        .enumerate()
        .map(|(i, &(w_a, w_b))| BracketSlot {
            round: "Round of 16".into(),
            match_number: (17 + i) as u32,
            home_label: format!("W{w_a}"),
            away_label: format!("W{w_b}"),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        })
        .collect();

    // QF pairings:
    // M25: W17 vs W18  M26: W21 vs W22  M27: W19 vs W20  M28: W23 vs W24
    let qf_specs: &[(u32, u32)] = &[(17, 18), (21, 22), (19, 20), (23, 24)];
    let qf: Vec<BracketSlot> = qf_specs
        .iter()
        .enumerate()
        .map(|(i, &(w_a, w_b))| BracketSlot {
            round: "Quarter-finals".into(),
            match_number: (25 + i) as u32,
            home_label: format!("W{w_a}"),
            away_label: format!("W{w_b}"),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        })
        .collect();

    // SF pairings:
    // M29: W25 vs W26  M30: W27 vs W28
    let sf: Vec<BracketSlot> = vec![
        BracketSlot {
            round: "Semi-finals".into(),
            match_number: 29,
            home_label: "W25".into(),
            away_label: "W26".into(),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        },
        BracketSlot {
            round: "Semi-finals".into(),
            match_number: 30,
            home_label: "W27".into(),
            away_label: "W28".into(),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        },
    ];

    let third_place = vec![BracketSlot {
        round: "Third Place".into(),
        match_number: 31,
        home_label: "L29".into(),
        away_label: "L30".into(),
        home_team: None,
        away_team: None,
        home_result: None,
        away_result: None,
    }];

    let final_match = vec![BracketSlot {
        round: "Final".into(),
        match_number: 32,
        home_label: "W29".into(),
        away_label: "W30".into(),
        home_team: None,
        away_team: None,
        home_result: None,
        away_result: None,
    }];

    Bracket {
        rounds: vec![r32, r16, qf, sf, final_match, third_place],
    }
}

pub fn apply_knockout_results(
    base: &Bracket,
    results: &HashMap<String, KnockoutResult>,
) -> Bracket {
    let mut bracket = base.clone();

    for round_idx in 0..bracket.rounds.len() {
        for slot_idx in 0..bracket.rounds[round_idx].len() {
            let slot = &bracket.rounds[round_idx][slot_idx];

            if slot.home_team.is_none() && slot.away_team.is_none() {
                continue;
            }

            let key = format!("{}-{}", slot.round, slot.match_number);
            if let Some(res) = results.get(&key) {
                let slot = &mut bracket.rounds[round_idx][slot_idx];
                if res.winner_is_home {
                    slot.home_result = Some(1);
                    slot.away_result = Some(0);
                } else {
                    slot.home_result = Some(0);
                    slot.away_result = Some(1);
                }
            }
        }

        propagate_winners(&mut bracket, round_idx);
    }

    bracket
}

fn propagate_winners(bracket: &mut Bracket, from_round: usize) {
    let mut winners: HashMap<String, Team> = HashMap::new();

    for slot in &bracket.rounds[from_round] {
        let has_result = slot.home_result.is_some() && slot.away_result.is_some();
        if !has_result {
            continue;
        }
        let home_wins = slot.home_result.unwrap() > slot.away_result.unwrap();
        if home_wins {
            if let Some(ref team) = slot.home_team {
                winners.insert(format!("W{}", slot.match_number), team.clone());
            }
            if let Some(ref team) = slot.away_team {
                winners.insert(format!("L{}", slot.match_number), team.clone());
            }
        } else {
            if let Some(ref team) = slot.away_team {
                winners.insert(format!("W{}", slot.match_number), team.clone());
            }
            if let Some(ref team) = slot.home_team {
                winners.insert(format!("L{}", slot.match_number), team.clone());
            }
        }
    }

    for next_round in (from_round + 1)..bracket.rounds.len() {
        for slot in &mut bracket.rounds[next_round] {
            if let Some(team) = winners.get(&slot.home_label) {
                slot.home_team = Some(team.clone());
            }
            if let Some(team) = winners.get(&slot.away_label) {
                slot.away_team = Some(team.clone());
            }
        }
    }
}
