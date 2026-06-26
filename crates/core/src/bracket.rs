use crate::models::*;
use crate::standings::rank_third_places;
use std::collections::HashMap;

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

fn resolve_slot(label: &str, team_map: &HashMap<String, Team>, thirds: &[(GroupCode, Standing)]) -> Option<Team> {
    if label == "TBD" || label.is_empty() {
        return None;
    }

    if label.starts_with('3') && label.len() == 2 {
        let rank_char = label.chars().nth(1).unwrap_or('A');
        let rank = (rank_char as u32).saturating_sub('A' as u32) as usize;
        if rank < thirds.len() {
            return Some(thirds[rank].1.team.clone());
        }
        return None;
    }

    team_map.get(label).cloned()
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

    let r32_slots = vec![
        ("1A", "3C"),  // M1
        ("1B", "2C"),  // M2
        ("1C", "3A"),  // M3
        ("1D", "2B"),  // M4
        ("1E", "2D"),  // M5
        ("1F", "3B"),  // M6
        ("1G", "3D"),  // M7
        ("1H", "2E"),  // M8
        ("1I", "3E"),  // M9
        ("1J", "2F"),  // M10
        ("1K", "3F"),  // M11
        ("1L", "2G"),  // M12
        ("2A", "2H"),  // M13
        ("2I", "2J"),  // M14
        ("2K", "2L"),  // M15
        ("3G", "3H"),  // M16 (best thirds)
    ];

    let r32: Vec<BracketSlot> = r32_slots
        .iter()
        .enumerate()
        .map(|(i, (home_label, away_label))| BracketSlot {
            round: "Round of 32".into(),
            match_number: (i + 1) as u32,
            home_label: home_label.to_string(),
            away_label: away_label.to_string(),
            home_team: resolve_slot(home_label, &team_map, &thirds),
            away_team: resolve_slot(away_label, &team_map, &thirds),
            home_result: None,
            away_result: None,
        })
        .collect();

    let r16: Vec<BracketSlot> = (0..8)
        .map(|i| BracketSlot {
            round: "Round of 16".into(),
            match_number: (17 + i) as u32,
            home_label: format!("W{}", i * 2 + 1),
            away_label: format!("W{}", i * 2 + 2),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        })
        .collect();

    let qf: Vec<BracketSlot> = (0..4)
        .map(|i| BracketSlot {
            round: "Quarter-finals".into(),
            match_number: (25 + i) as u32,
            home_label: format!("W{}", 17 + i * 2),
            away_label: format!("W{}", 18 + i * 2),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        })
        .collect();

    let sf: Vec<BracketSlot> = (0..2)
        .map(|i| BracketSlot {
            round: "Semi-finals".into(),
            match_number: (29 + i) as u32,
            home_label: format!("W{}", 25 + i * 2),
            away_label: format!("W{}", 26 + i * 2),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        })
        .collect();

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

