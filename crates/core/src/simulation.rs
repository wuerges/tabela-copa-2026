use crate::models::*;
use crate::standings::{calculate_standings, group_standings, rank_third_places};
use std::collections::{HashMap, HashSet};

const TOP_THIRDS: usize = 8;
const MAX_EXHAUSTIVE_SCENARIOS: u64 = 100_000;
const MONTE_CARLO_SAMPLES: u64 = 50_000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThirdPlaceSimulation {
    pub direct_qualified: Vec<String>,
    pub guaranteed: Vec<String>,
    pub eliminated: Vec<String>,
    pub uncertain: Vec<ThirdPlaceChance>,
    pub total_scenarios: u64,
    pub unplayed_matches: usize,
    pub method: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThirdPlaceChance {
    pub team: Team,
    pub group: GroupCode,
    pub scenarios_qualified: u64,
    pub percentage: f64,
    pub points: u32,
    pub goal_diff: i32,
}

pub fn simulate_guaranteed_thirds(matches: &[Match]) -> ThirdPlaceSimulation {
    let unplayed: Vec<usize> = matches
        .iter()
        .enumerate()
        .filter(|(_, m)| m.result.is_none())
        .map(|(i, _)| i)
        .collect();
    let n = unplayed.len();
    let total_exhaustive = 3u64.saturating_pow(n as u32);

    let mut third_qualified_counts: HashMap<String, u64> = HashMap::new();
    let mut top_two_counts: HashMap<String, u64> = HashMap::new();
    let mut all_teams: HashSet<String> = HashSet::new();

    for group_code in GROUP_CODES {
        let teams: HashSet<String> = matches
            .iter()
            .filter(|m| m.group.0 == *group_code)
            .flat_map(|m| vec![m.home_team.fifa_code.clone(), m.away_team.fifa_code.clone()])
            .collect();
        for t in teams {
            all_teams.insert(t.clone());
            third_qualified_counts.entry(t.clone()).or_insert(0);
            top_two_counts.entry(t).or_insert(0);
        }
    }

    let method;
    let total_scenarios;

    if total_exhaustive > 0 && total_exhaustive <= MAX_EXHAUSTIVE_SCENARIOS {
        method = "exhaustive".to_string();
        total_scenarios = total_exhaustive;
        let mut current = matches.to_vec();
        simulate_exhaustive(
            &matches,
            &mut current,
            &unplayed,
            0,
            &mut third_qualified_counts,
            &mut top_two_counts,
        );
    } else {
        method = format!("monte-carlo ({MONTE_CARLO_SAMPLES} samples)");
        total_scenarios = MONTE_CARLO_SAMPLES;
        simulate_monte_carlo(
            matches,
            &unplayed,
            MONTE_CARLO_SAMPLES,
            &mut third_qualified_counts,
            &mut top_two_counts,
        );
    }

    let mut team_info: HashMap<String, (Team, GroupCode)> = HashMap::new();
    for m in matches {
        for team in [&m.home_team, &m.away_team] {
            team_info
                .entry(team.fifa_code.clone())
                .or_insert_with(|| (team.clone(), m.group.clone()));
        }
    }

    let mut direct_qualified: Vec<String> = Vec::new();
    let mut guaranteed: Vec<String> = Vec::new();
    let mut eliminated: Vec<String> = Vec::new();
    let mut uncertain: Vec<ThirdPlaceChance> = Vec::new();

    for fifa_code in &all_teams {
        let third_count = third_qualified_counts.get(fifa_code).copied().unwrap_or(0);
        let top2_count = top_two_counts.get(fifa_code).copied().unwrap_or(0);
        let (team, group) = team_info[fifa_code].clone();

        let group_matches: Vec<&Match> = matches.iter().filter(|m| m.group.0 == group.0).collect();
        let standings =
            calculate_standings(&group_matches.iter().cloned().cloned().collect::<Vec<_>>());
        let standing = standings.iter().find(|s| s.team.fifa_code == *fifa_code);

        let (points, goal_diff) = standing
            .map(|s| (s.points, s.goal_diff))
            .unwrap_or((0, 0));

        if top2_count == total_scenarios {
            direct_qualified.push(team.name.clone());
        } else if third_count == total_scenarios {
            guaranteed.push(team.name.clone());
        } else if third_count > 0 {
            uncertain.push(ThirdPlaceChance {
                team,
                group,
                scenarios_qualified: third_count,
                percentage: if total_scenarios > 0 {
                    (third_count as f64 / total_scenarios as f64) * 100.0
                } else {
                    0.0
                },
                points,
                goal_diff,
            });
        } else {
            eliminated.push(team.name.clone());
        }
    }

    direct_qualified.sort();
    guaranteed.sort();
    eliminated.sort();
    uncertain.sort_by(|a, b| b.scenarios_qualified.cmp(&a.scenarios_qualified));

    ThirdPlaceSimulation {
        direct_qualified,
        guaranteed,
        eliminated,
        uncertain,
        total_scenarios,
        unplayed_matches: n,
        method,
    }
}

fn simulate_exhaustive(
    original: &[Match],
    current: &mut [Match],
    unplayed: &[usize],
    idx: usize,
    third_qualified_counts: &mut HashMap<String, u64>,
    top_two_counts: &mut HashMap<String, u64>,
) {
    if idx == unplayed.len() {
        let gs = group_standings(current);
        for (_, standings) in &gs {
            for s in standings.iter().take(2) {
                *top_two_counts.get_mut(&s.team.fifa_code).unwrap() += 1;
            }
        }
        let third_places = rank_third_places(&gs);
        for (_, standing) in third_places.iter().take(TOP_THIRDS) {
            *third_qualified_counts
                .get_mut(&standing.team.fifa_code)
                .unwrap() += 1;
        }
        return;
    }

    let match_idx = unplayed[idx];

    let results = [
        MatchResult { home_goals: 1, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 1 },
    ];

    for result in &results {
        current[match_idx].result = Some(*result);
        simulate_exhaustive(
            original,
            current,
            unplayed,
            idx + 1,
            third_qualified_counts,
            top_two_counts,
        );
    }

    current[match_idx].result = original[match_idx].result;
}

fn simulate_monte_carlo(
    original: &[Match],
    unplayed: &[usize],
    samples: u64,
    third_qualified_counts: &mut HashMap<String, u64>,
    top_two_counts: &mut HashMap<String, u64>,
) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut current = original.to_vec();

    let results = [
        MatchResult { home_goals: 1, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 1 },
    ];

    for s in 0..samples {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);

        for (i, &match_idx) in unplayed.iter().enumerate() {
            (i as u64).hash(&mut hasher);
            s.hash(&mut hasher);
            let h = hasher.finish();
            let result_idx = (h % 3) as usize;
            current[match_idx].result = Some(results[result_idx]);
        }

        let gs = group_standings(&current);
        for (_, standings) in &gs {
            for s in standings.iter().take(2) {
                if let Some(count) = top_two_counts.get_mut(&s.team.fifa_code) {
                    *count += 1;
                }
            }
        }
        let third_places = rank_third_places(&gs);
        for (_, standing) in third_places.iter().take(TOP_THIRDS) {
            if let Some(count) = third_qualified_counts.get_mut(&standing.team.fifa_code) {
                *count += 1;
            }
        }
    }
}
