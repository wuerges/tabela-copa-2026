use copa2026_core::*;

fn make_team(name: &str, code: &str) -> Team {
    Team {
        name: name.into(),
        fifa_code: code.into(),
    }
}

fn make_match_result(home: u32, away: u32) -> MatchResult {
    MatchResult {
        home_goals: home,
        away_goals: away,
    }
}

fn make_match(id: &str, group: &str, home: Team, away: Team, result: Option<MatchResult>) -> Match {
    Match {
        id: id.into(),
        group: GroupCode(group.into()),
        home_team: home,
        away_team: away,
        result,
        date: None,
    }
}

#[test]
fn test_standings_all_wins() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(2, 0))),
        make_match("2", "A", t3.clone(), t4.clone(), Some(make_match_result(3, 1))),
        make_match("3", "A", t1.clone(), t3.clone(), Some(make_match_result(1, 0))),
        make_match("4", "A", t2.clone(), t4.clone(), Some(make_match_result(2, 0))),
        make_match("5", "A", t4.clone(), t1.clone(), Some(make_match_result(0, 3))),
        make_match("6", "A", t2.clone(), t3.clone(), Some(make_match_result(1, 1))),
    ];

    let standings = calculate_standings(&matches);

    assert_eq!(standings.len(), 4);
    assert_eq!(standings[0].team.name, "Alpha");
    assert_eq!(standings[0].points, 9);
    assert_eq!(standings[0].played, 3);
    assert_eq!(standings[0].won, 3);
    assert_eq!(standings[0].drawn, 0);
    assert_eq!(standings[0].lost, 0);
    assert_eq!(standings[0].position, 1);
}

#[test]
fn test_standings_tiebreaker_goal_diff() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(1, 0))),
        make_match("2", "A", t3.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("3", "A", t1.clone(), t3.clone(), Some(make_match_result(0, 1))),
        make_match("4", "A", t2.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("5", "A", t4.clone(), t1.clone(), Some(make_match_result(0, 1))),
        make_match("6", "A", t2.clone(), t3.clone(), Some(make_match_result(1, 0))),
    ];

    let standings = calculate_standings(&matches);
    let points: Vec<u32> = standings.iter().map(|s| s.points).collect();

    assert_eq!(points, vec![6, 6, 6, 0], "Three teams should have 6 pts");
}

#[test]
fn test_unplayed_matches_not_counted() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(1, 0))),
        make_match("2", "A", t3.clone(), t4.clone(), None),
        make_match("3", "A", t1.clone(), t3.clone(), None),
        make_match("4", "A", t2.clone(), t4.clone(), None),
        make_match("5", "A", t4.clone(), t1.clone(), None),
        make_match("6", "A", t2.clone(), t3.clone(), None),
    ];

    let standings = calculate_standings(&matches);

    assert_eq!(standings[0].team.name, "Alpha");
    assert_eq!(standings[0].points, 3);
    assert_eq!(standings[0].played, 1);
    assert_eq!(standings[0].goals_for, 1);
    assert_eq!(standings[0].goals_against, 0);

    let zero_played: Vec<_> = standings.iter().filter(|s| s.played == 0).collect();
    assert_eq!(zero_played.len(), 2);
}

#[test]
fn test_draw_points() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(0, 0))),
        make_match("2", "A", t3.clone(), t4.clone(), Some(make_match_result(0, 0))),
        make_match("3", "A", t1.clone(), t3.clone(), Some(make_match_result(0, 0))),
        make_match("4", "A", t2.clone(), t4.clone(), Some(make_match_result(0, 0))),
        make_match("5", "A", t4.clone(), t1.clone(), Some(make_match_result(0, 0))),
        make_match("6", "A", t2.clone(), t3.clone(), Some(make_match_result(0, 0))),
    ];

    let standings = calculate_standings(&matches);
    for s in &standings {
        assert_eq!(s.points, 3);
        assert_eq!(s.won, 0);
        assert_eq!(s.drawn, 3);
        assert_eq!(s.lost, 0);
    }
}

#[test]
fn test_group_standings_structure() {
    let t = make_team("Team", "TEA");
    let matches = vec![
        make_match("1", "A", t.clone(), make_team("Opp", "OPP"), Some(make_match_result(1, 0))),
        make_match("2", "B", t.clone(), make_team("Opp2", "OP2"), Some(make_match_result(2, 1))),
    ];

    let gs = group_standings(&matches);
    assert_eq!(gs.len(), 12, "Should have 12 groups");

    let group_a = gs.iter().find(|(c, _)| c.0 == "A").unwrap();
    assert_eq!(group_a.1.len(), 2);

    let empty_groups: Vec<_> = gs.iter().filter(|(_, s)| s.is_empty()).collect();
    assert_eq!(empty_groups.len(), 10, "10 groups should be empty");
}

#[test]
fn test_rank_third_places() {
    let gs = basic_standings_12();

    let thirds = rank_third_places(&gs);
    assert_eq!(thirds.len(), 12, "Should have 12 third-placed teams");

    for i in 1..thirds.len() {
        let prev = &thirds[i - 1].1;
        let curr = &thirds[i].1;
        assert!(
            prev.points > curr.points
                || (prev.points == curr.points && prev.goal_diff >= curr.goal_diff)
        );
    }
}

#[test]
fn test_bracket_generation() {
    let gs = basic_standings_12();
    let bracket = generate_bracket(&gs);
    assert_eq!(bracket.rounds.len(), 6);
    assert_eq!(bracket.rounds[0].len(), 16, "Round of 32 should have 16 matches");
    assert_eq!(bracket.rounds[1].len(), 8, "Round of 16 should have 8 matches");
    assert_eq!(bracket.rounds[2].len(), 4, "QF should have 4 matches");
    assert_eq!(bracket.rounds[3].len(), 2, "SF should have 2 matches");
    assert_eq!(bracket.rounds[4].len(), 1, "Third place match");
    assert_eq!(bracket.rounds[5].len(), 1, "Final");
}

#[test]
fn test_simulation_no_unplayed_matches() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("a", "A", t1.clone(), t2.clone(), Some(make_match_result(1, 0))),
        make_match("b", "A", t3.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("c", "A", t1.clone(), t3.clone(), Some(make_match_result(1, 0))),
        make_match("d", "A", t2.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("e", "A", t4.clone(), t1.clone(), Some(make_match_result(1, 0))),
        make_match("f", "A", t2.clone(), t3.clone(), Some(make_match_result(1, 0))),
    ];

    let sim = simulate_guaranteed_thirds(&matches);
    assert_eq!(sim.unplayed_matches, 0);
    assert!(sim.teams.iter().any(|t| t.first_pct == 100.0 || t.second_pct == 100.0));
}

#[test]
fn test_simulation_qualification_sums_to_total() {
    let gs = basic_standings_12();
    let all_matches: Vec<Match> = gs.iter().flat_map(|(_, s)| {
        let code = s[0].team.fifa_code.clone();
        let opp_code = if code == "ALP" { "BET" } else { "ALP" };
        vec![
            make_match("1", &code[..2], s[0].team.clone(), make_team("Opp", &opp_code), Some(make_match_result(1, 0))),
        ]
    }).collect();

    let sim = simulate_guaranteed_thirds(&all_matches);
    for tc in &sim.teams {
        let sum = tc.first_pct + tc.second_pct + tc.third_qualified_pct;
        let remainder = 100.0 - tc.total_qualification_pct;
        assert!(
            (sum - remainder).abs() < 0.1 || (sum - tc.total_qualification_pct).abs() < 0.1,
            "{}: 1st={:.1} 2nd={:.1} 3rd={:.1} total={:.1}",
            tc.team.name, tc.first_pct, tc.second_pct, tc.third_qualified_pct, tc.total_qualification_pct
        );
    }
}

fn basic_standings_12() -> Vec<(GroupCode, Vec<Standing>)> {
    GROUP_CODES
        .iter()
        .enumerate()
        .map(|(i, code)| {
            let base = (11 - i) as u32;
            let standings = vec![
                Standing {
                    position: 1, team: make_team(&format!("Team{code}_1"), &format!("T1{code}")),
                    played: 3, won: 3, drawn: 0, lost: 0,
                    goals_for: base + 5, goals_against: base, goal_diff: 5,
                    points: 9,
                },
                Standing {
                    position: 2, team: make_team(&format!("Team{code}_2"), &format!("T2{code}")),
                    played: 3, won: 1, drawn: 1, lost: 1,
                    goals_for: base + 2, goals_against: base + 1, goal_diff: 1,
                    points: 3 + (i as u32 % 2),
                },
                Standing {
                    position: 3, team: make_team(&format!("Team{code}_3"), &format!("T3{code}")),
                    played: 3, won: 1, drawn: 0, lost: 2,
                    goals_for: base + 1, goals_against: base + 3, goal_diff: -2,
                    points: 2 + (i as u32 % 2),
                },
                Standing {
                    position: 4, team: make_team(&format!("Team{code}_4"), &format!("T4{code}")),
                    played: 3, won: 0, drawn: 1, lost: 2,
                    goals_for: base, goals_against: base + 4, goal_diff: -4,
                    points: 0,
                },
            ];
            (GroupCode(code.to_string()), standings)
        })
        .collect()
}

#[test]
fn test_knockout_full_bracket_to_final() {
    let gs = basic_standings_12();
    let base_bracket = generate_bracket(&gs);

    let rounds = &base_bracket.rounds;
    assert_eq!(rounds.len(), 6, "R32, R16, QF, SF, 3rd, Final");
    assert_eq!(rounds[0].len(), 16);
    assert_eq!(rounds[1].len(), 8);
    assert_eq!(rounds[2].len(), 4);
    assert_eq!(rounds[3].len(), 2);
    assert_eq!(rounds[4].len(), 1);
    assert_eq!(rounds[5].len(), 1);

    assert!(rounds[0].iter().all(|s| s.home_team.is_some() && s.away_team.is_some()),
        "All R32 slots should have teams from group standings");
    assert!(rounds[1].iter().all(|s| s.home_team.is_none() && s.away_team.is_none()),
        "R16 should be empty before any results");

    let mut results: std::collections::HashMap<String, KnockoutResult> = std::collections::HashMap::new();

    for slot in &rounds[0] {
        let key = format!("{}-{}", slot.round, slot.match_number);
        results.insert(key, KnockoutResult {
            round: slot.round.clone(),
            match_number: slot.match_number,
            winner_is_home: true,
        });
    }

    let bracket_r32 = apply_knockout_results(&base_bracket, &results);

    let r32_slots = &bracket_r32.rounds[0];
    for slot in r32_slots {
        assert_eq!(slot.home_result, Some(1));
        assert_eq!(slot.away_result, Some(0));
    }

    let r16_slots = &bracket_r32.rounds[1];
    for slot in r16_slots {
        assert!(slot.home_team.is_some(), "R16 home team should be filled after R32 results");
        assert!(slot.away_team.is_some(), "R16 away team should be filled after R32 results");
    }

    for slot in &bracket_r32.rounds[1] {
        let key = format!("{}-{}", slot.round, slot.match_number);
        results.insert(key, KnockoutResult {
            round: slot.round.clone(),
            match_number: slot.match_number,
            winner_is_home: true,
        });
    }

    let bracket_r16 = apply_knockout_results(&base_bracket, &results);
    let r16 = &bracket_r16.rounds[1];
    for slot in r16 {
        assert_eq!(slot.home_result, Some(1));
        assert_eq!(slot.away_result, Some(0));
    }

    let qf_slots = &bracket_r16.rounds[2];
    for slot in qf_slots {
        assert!(slot.home_team.is_some(), "QF home team should be filled after R16");
        assert!(slot.away_team.is_some(), "QF away team should be filled after R16");
    }

    for slot in &bracket_r16.rounds[2] {
        let key = format!("{}-{}", slot.round, slot.match_number);
        results.insert(key, KnockoutResult {
            round: slot.round.clone(),
            match_number: slot.match_number,
            winner_is_home: true,
        });
    }

    let bracket_qf = apply_knockout_results(&base_bracket, &results);
    let qf = &bracket_qf.rounds[2];
    for slot in qf {
        assert_eq!(slot.home_result, Some(1));
        assert_eq!(slot.away_result, Some(0));
    }

    let sf_slots = &bracket_qf.rounds[3];
    for slot in sf_slots {
        assert!(slot.home_team.is_some(), "SF home team should be filled after QF");
        assert!(slot.away_team.is_some(), "SF away team should be filled after QF");
    }

    for slot in &bracket_qf.rounds[3] {
        let key = format!("{}-{}", slot.round, slot.match_number);
        results.insert(key, KnockoutResult {
            round: slot.round.clone(),
            match_number: slot.match_number,
            winner_is_home: true,
        });
    }

    let bracket_sf = apply_knockout_results(&base_bracket, &results);
    let sf = &bracket_sf.rounds[3];
    for slot in sf {
        assert_eq!(slot.home_result, Some(1));
        assert_eq!(slot.away_result, Some(0));
    }

    let third_place_slot = &bracket_sf.rounds[5][0];
    assert!(third_place_slot.home_team.is_some(), "3rd place home should be filled");
    assert!(third_place_slot.away_team.is_some(), "3rd place away should be filled");

    let final_slot = &bracket_sf.rounds[4][0];
    assert!(final_slot.home_team.is_some(), "Final home team should be filled after SF");
    assert!(final_slot.away_team.is_some(), "Final away team should be filled after SF");

    let final_key = format!("{}-{}", final_slot.round, final_slot.match_number);
    results.insert(final_key, KnockoutResult {
        round: final_slot.round.clone(),
        match_number: final_slot.match_number,
        winner_is_home: true,
    });

    let bracket_final = apply_knockout_results(&base_bracket, &results);
    let f = &bracket_final.rounds[4][0];
    assert_eq!(f.home_result, Some(1));
    assert_eq!(f.away_result, Some(0));

    let third_key = format!("{}-{}", third_place_slot.round, third_place_slot.match_number);
    results.insert(third_key, KnockoutResult {
        round: third_place_slot.round.clone(),
        match_number: third_place_slot.match_number,
        winner_is_home: false,
    });

    let bracket_all = apply_knockout_results(&base_bracket, &results);

    for round in &bracket_all.rounds {
        for slot in round {
            assert!(slot.home_result.is_some(), "Every match should have a result");
            assert!(slot.away_result.is_some(), "Every match should have a result");
        }
    }

    let champion = &bracket_all.rounds[4][0];
    assert!(champion.home_result.unwrap() > champion.away_result.unwrap() || champion.away_result.unwrap() > champion.home_result.unwrap());
    let champ_name = champion.home_team.as_ref().unwrap().name.clone();
    let sb1_name = bracket_all.rounds[3][0].home_team.as_ref().unwrap().name.clone();
    assert_eq!(champ_name, sb1_name);
}
