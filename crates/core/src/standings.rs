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

            let (h, a) = if h_idx < a_idx {
                let (left, right) = team_data.split_at_mut(a_idx);
                (&mut left[h_idx], &mut right[0])
            } else if a_idx < h_idx {
                let (left, right) = team_data.split_at_mut(h_idx);
                (&mut right[0], &mut left[a_idx])
            } else {
                unreachable!()
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
