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
                log::warn!(
                    "Match {} has same team ({}) as home and away — skipping",
                    m.id,
                    m.home_team.fifa_code
                );
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

    // FIFA 2026 tiebreakers:
    // 1. Points, 2. Goal difference, 3. Goals scored
    standings.sort_by(|a, b| {
        b.points
            .cmp(&a.points)
            .then_with(|| b.goal_diff.cmp(&a.goal_diff))
            .then_with(|| b.goals_for.cmp(&a.goals_for))
    });

    // Steps 4-6: Head-to-head among teams tied on points, GD, and GF
    apply_head_to_head_tiebreakers(matches, &mut standings);

    for (i, s) in standings.iter_mut().enumerate() {
        s.position = (i + 1) as u32;
    }

    standings
}

/// FIFA 2026 tiebreakers 4-6: for clusters of teams tied on points, goal difference,
/// and goals scored, re-sort by head-to-head results among only the tied teams.
fn apply_head_to_head_tiebreakers(matches: &[Match], standings: &mut [Standing]) {
    if standings.len() < 2 {
        return;
    }
    let mut start = 0;
    while start < standings.len() {
        // Find the cluster of consecutive teams tied on all three primary criteria
        let mut end = start + 1;
        while end < standings.len()
            && standings[end].points == standings[start].points
            && standings[end].goal_diff == standings[start].goal_diff
            && standings[end].goals_for == standings[start].goals_for
        {
            end += 1;
        }
        let cluster_len = end - start;
        if cluster_len > 1 {
            // Collect the fifa codes of teams in this cluster (owned to release borrow)
            let tied_codes: std::collections::HashSet<String> = standings[start..end]
                .iter()
                .map(|s| s.team.fifa_code.clone())
                .collect();
            // Compute head-to-head stats directly (not via calculate_standings
            // to avoid infinite recursion)
            let mut h2h: std::collections::HashMap<String, (u32, i32, u32)> =
                std::collections::HashMap::new(); // (pts, gd, gf)
            for code in &tied_codes {
                h2h.insert(code.clone(), (0, 0, 0));
            }
            for m in matches {
                if !tied_codes.contains(&m.home_team.fifa_code)
                    || !tied_codes.contains(&m.away_team.fifa_code)
                {
                    continue;
                }
                if let Some(ref result) = m.result {
                    let h = &m.home_team.fifa_code;
                    let a = &m.away_team.fifa_code;
                    let hg = result.home_goals;
                    let ag = result.away_goals;
                    let hg_i = hg as i32;
                    let ag_i = ag as i32;
                    match result.outcome() {
                        MatchOutcome::HomeWin => {
                            if let Some(e) = h2h.get_mut(h) { e.0 += 3; e.1 += hg_i - ag_i; e.2 += hg; }
                            if let Some(e) = h2h.get_mut(a) { e.1 += ag_i - hg_i; e.2 += ag; }
                        }
                        MatchOutcome::AwayWin => {
                            if let Some(e) = h2h.get_mut(a) { e.0 += 3; e.1 += ag_i - hg_i; e.2 += ag; }
                            if let Some(e) = h2h.get_mut(h) { e.1 += hg_i - ag_i; e.2 += hg; }
                        }
                        MatchOutcome::Draw => {
                            if let Some(e) = h2h.get_mut(h) { e.0 += 1; e.1 += hg_i - ag_i; e.2 += hg; }
                            if let Some(e) = h2h.get_mut(a) { e.0 += 1; e.1 += ag_i - hg_i; e.2 += ag; }
                        }
                    }
                }
            }
            // Re-sort the cluster by head-to-head: points, then GD, then GF
            standings[start..end].sort_by(|a, b| {
                let ha = h2h.get(&a.team.fifa_code);
                let hb = h2h.get(&b.team.fifa_code);
                match (ha, hb) {
                    (Some((pa, gda, gfa)), Some((pb, gdb, gfb))) => pb
                        .cmp(pa)
                        .then_with(|| gdb.cmp(gda))
                        .then_with(|| gfb.cmp(gfa)),
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }
        start = end;
    }
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
    // Safety guard: 5^12 ≈ 244M, but typical max is 5^6 = 15,625 per group
    assert!(
        unplayed.len() <= 6,
        "Too many unplayed matches for exhaustive enumeration"
    );

    if idx == unplayed.len() {
        callback(current);
        return;
    }

    let match_idx = unplayed[idx];
    let results = [
        MatchResult { home_goals: 1, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 1 },
        MatchResult { home_goals: 3, away_goals: 0 },
        MatchResult { home_goals: 0, away_goals: 3 },
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
