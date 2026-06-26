use crate::models::*;
use crate::standings::rank_third_places;

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

fn resolve_slot(label: &str, team_map: &std::collections::HashMap<String, Team>, thirds: &[(GroupCode, Standing)]) -> Option<Team> {
    if label == "TBD" || label.is_empty() {
        return None;
    }

    if label.starts_with('3') && label.len() > 1 {
        let group_code = &label[1..];
        if let Some((_, standing)) = thirds.iter().find(|(gc, _)| gc.0 == group_code) {
            return Some(standing.team.clone());
        }
        return None;
    }

    team_map.get(label).cloned()
}

pub fn generate_bracket(group_standings: &[(GroupCode, Vec<Standing>)]) -> Bracket {
    let mut team_map: std::collections::HashMap<String, Team> = std::collections::HashMap::new();
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

    let mut r16: Vec<BracketSlot> = Vec::new();
    for i in 0..8 {
        r16.push(BracketSlot {
            round: "Round of 16".into(),
            match_number: (i + 1) as u32,
            home_label: format!("W{}", i * 2 + 1),
            away_label: format!("W{}", i * 2 + 2),
            home_team: None,
            away_team: None,
            home_result: None,
            away_result: None,
        });
    }

    let qf: Vec<BracketSlot> = (0..4)
        .map(|i| BracketSlot {
            round: "Quarter-finals".into(),
            match_number: (i + 1) as u32,
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
            match_number: (i + 1) as u32,
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
        match_number: 1,
        home_label: "L29".into(),
        away_label: "L30".into(),
        home_team: None,
        away_team: None,
        home_result: None,
        away_result: None,
    }];

    let final_match = vec![BracketSlot {
        round: "Final".into(),
        match_number: 1,
        home_label: "W29".into(),
        away_label: "W30".into(),
        home_team: None,
        away_team: None,
        home_result: None,
        away_result: None,
    }];

    Bracket {
        rounds: vec![r32, r16, qf, sf, third_place, final_match],
    }
}
