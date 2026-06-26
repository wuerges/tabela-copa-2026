use crate::models::*;
use std::collections::HashMap;

pub fn calculate_standings(matches: &[Match]) -> Vec<Standing> {
    let mut team_data: Vec<TeamStats> = Vec::new();
    let mut code_to_idx: HashMap<String, usize> = HashMap::new();

    let get_or_create_idx = |team_data: &mut Vec<TeamStats>, code_to_idx: &mut HashMap<String, usize>, team: &Team| -> usize {
        if let Some(&idx) = code_to_idx.get(&team.fifa_code) {
            idx
        } else {
            let idx = team_data.len();
            team_data.push(TeamStats {
                team: team.clone(),
                ..Default::default()
            });
            code_to_idx.insert(team.fifa_code.clone(), idx);
            idx
        }
    };

    for m in matches {
        let h_idx = get_or_create_idx(&mut team_data, &mut code_to_idx, &m.home_team);
        let a_idx = get_or_create_idx(&mut team_data, &mut code_to_idx, &m.away_team);

        if let Some(ref result) = m.result {
            let home_goals = result.home_goals;
            let away_goals = result.away_goals;
            let outcome = result.outcome();

            let (h, a) = if h_idx == a_idx {
                continue;
            } else if h_idx < a_idx {
                let (left, right) = team_data.split_at_mut(a_idx);
                (&mut left[h_idx], &mut right[0])
            } else {
                let (left, right) = team_data.split_at_mut(h_idx);
                (&mut right[0], &mut left[a_idx])
            };

            h.played += 1;
            a.played += 1;
            h.goals_for += home_goals;
            h.goals_against += away_goals;
            a.goals_for += away_goals;
            a.goals_against += home_goals;

            match outcome {
                MatchOutcome::HomeWin => {
                    h.won += 1;
                    h.points += 3;
                    a.lost += 1;
                }
                MatchOutcome::AwayWin => {
                    a.won += 1;
                    a.points += 3;
                    h.lost += 1;
                }
                MatchOutcome::Draw => {
                    h.drawn += 1;
                    h.points += 1;
                    a.drawn += 1;
                    a.points += 1;
                }
            }
        }
    }

    let mut standings: Vec<Standing> = team_data
        .into_iter()
        .map(|ts| {
            let goal_diff = ts.goals_for as i32 - ts.goals_against as i32;
            Standing {
                position: 0,
                team: ts.team,
                played: ts.played,
                won: ts.won,
                drawn: ts.drawn,
                lost: ts.lost,
                goals_for: ts.goals_for,
                goals_against: ts.goals_against,
                goal_diff,
                points: ts.points,
            }
        })
        .collect();

    standings.sort_by(|a, b| {
        b.points
            .cmp(&a.points)
            .then_with(|| b.goal_diff.cmp(&a.goal_diff))
            .then_with(|| b.goals_for.cmp(&a.goals_for))
    });

    for (i, s) in standings.iter_mut().enumerate() {
        s.position = (i + 1) as u32;
    }

    standings
}

pub fn group_standings(all_matches: &[Match]) -> Vec<(GroupCode, Vec<Standing>)> {
    let mut grouped: HashMap<String, Vec<Match>> = HashMap::new();
    for m in all_matches {
        grouped
            .entry(m.group.0.clone())
            .or_default()
            .push(m.clone());
    }

    let mut result: Vec<(GroupCode, Vec<Standing>)> = GROUP_CODES
        .iter()
        .map(|code| {
            let matches = grouped.remove(*code).unwrap_or_default();
            let standings = calculate_standings(&matches);
            (GroupCode(code.to_string()), standings)
        })
        .collect();

    result.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));
    result
}

pub fn rank_third_places(group_standings: &[(GroupCode, Vec<Standing>)]) -> Vec<(GroupCode, Standing)> {
    let mut thirds: Vec<(GroupCode, Standing)> = group_standings
        .iter()
        .filter_map(|(code, standings)| {
            standings.get(2).map(|s| (code.clone(), s.clone()))
        })
        .collect();

    thirds.sort_by(|a, b| {
        b.1.points
            .cmp(&a.1.points)
            .then_with(|| b.1.goal_diff.cmp(&a.1.goal_diff))
            .then_with(|| b.1.goals_for.cmp(&a.1.goals_for))
    });

    thirds
}

pub fn clinched_positions(all_matches: &[Match]) -> HashMap<String, Vec<u32>> {
    let mut result = HashMap::new();

    for group_code in GROUP_CODES {
        let group_matches: Vec<&Match> = all_matches
            .iter()
            .filter(|m| m.group.0 == *group_code)
            .collect();

        let unplayed: Vec<usize> = group_matches
            .iter()
            .enumerate()
            .filter(|(_, m)| m.result.is_none())
            .map(|(i, _)| i)
            .collect();

        let mut position_sets: HashMap<String, (u32, u32)> = HashMap::new();

        let mut current: Vec<Match> = group_matches.iter().map(|&m| m.clone()).collect();
        for_each_permutation(&mut current, &unplayed, 0, &mut |matches| {
            let standings = calculate_standings(matches);
            for s in &standings {
                let entry = position_sets
                    .entry(s.team.fifa_code.clone())
                    .or_insert((u32::MAX, 0));
                entry.0 = entry.0.min(s.position);
                entry.1 = entry.1.max(s.position);
            }
        });

        for (code, (min_pos, max_pos)) in position_sets {
            if min_pos == max_pos {
                result
                    .entry(code)
                    .or_insert_with(Vec::new)
                    .push(min_pos);
            }
        }
    }

    result
}

fn for_each_permutation(
    current: &mut [Match],
    unplayed: &[usize],
    idx: usize,
    callback: &mut dyn FnMut(&[Match]),
) {
    if idx == unplayed.len() {
        callback(current);
        return;
    }

    let match_idx = unplayed[idx];
    let results = [
        MatchResult { home_goals: 1, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 1 },
    ];

    let original = current[match_idx].result;

    for result in &results {
        current[match_idx].result = Some(*result);
        for_each_permutation(current, unplayed, idx + 1, callback);
    }

    current[match_idx].result = original;
}

#[derive(Default)]
struct TeamStats {
    team: Team,
    played: u32,
    won: u32,
    drawn: u32,
    lost: u32,
    goals_for: u32,
    goals_against: u32,
    points: u32,
}
