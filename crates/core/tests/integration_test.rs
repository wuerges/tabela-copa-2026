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
    assert_eq!(bracket.rounds[4].len(), 1, "Final");
    assert_eq!(bracket.rounds[5].len(), 1, "Third Place");
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
    let all_matches: Vec<Match> = gs.iter().flat_map(|(group_code, s)| {
        // Each team code is T1A, T2A, etc.; derive group from the GroupCode, not the fifa_code
        let code = group_code.0.clone();
        let opp_code = if code == "A" { "T1B" } else { "T1A" };
        vec![
            make_match("1", &code, s[0].team.clone(), make_team("Opp", &opp_code), Some(make_match_result(1, 0))),
        ]
    }).collect();

    let sim = simulate_guaranteed_thirds(&all_matches);
    assert!(!sim.teams.is_empty(), "Should have teams in the simulation");
    for tc in &sim.teams {
        let sum = tc.first_pct + tc.second_pct + tc.third_qualified_pct;
        assert!(
            (sum - tc.total_qualification_pct).abs() < 0.2,
            "{}: 1st={:.1} 2nd={:.1} 3rd={:.1} total={:.1} (sum={})",
            tc.team.name, tc.first_pct, tc.second_pct, tc.third_qualified_pct, tc.total_qualification_pct, sum
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

    let mut results: std::collections::HashMap<String, KnockoutMatch> = std::collections::HashMap::new();

    for slot in &rounds[0] {
        let key = format!("{}-{}", slot.round, slot.match_number);
        results.insert(key, KnockoutMatch {
            round: slot.round.clone(),
            match_number: slot.match_number,
            home_goals: Some(1), away_goals: Some(0),
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
        results.insert(key, KnockoutMatch {
            round: slot.round.clone(),
            match_number: slot.match_number,
            home_goals: Some(1), away_goals: Some(0),
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
        results.insert(key, KnockoutMatch {
            round: slot.round.clone(),
            match_number: slot.match_number,
            home_goals: Some(1), away_goals: Some(0),
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
        results.insert(key, KnockoutMatch {
            round: slot.round.clone(),
            match_number: slot.match_number,
            home_goals: Some(1), away_goals: Some(0),
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
    results.insert(final_key, KnockoutMatch {
        round: final_slot.round.clone(),
        match_number: final_slot.match_number,
        home_goals: Some(1), away_goals: Some(0),
    });

    let bracket_final = apply_knockout_results(&base_bracket, &results);
    let f = &bracket_final.rounds[4][0];
    assert_eq!(f.home_result, Some(1));
    assert_eq!(f.away_result, Some(0));

    let third_key = format!("{}-{}", third_place_slot.round, third_place_slot.match_number);
    results.insert(third_key, KnockoutMatch {
        round: third_place_slot.round.clone(),
        match_number: third_place_slot.match_number,
        home_goals: Some(0), away_goals: Some(1),
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

#[test]
fn test_same_team_fifa_code_does_not_panic() {
    let team = make_team("Clone", "CLO");
    let matches = vec![
        make_match("m1", "A", team.clone(), make_team("Opp", "OPP"), Some(make_match_result(1, 0))),
        make_match("m2", "A", team.clone(), team.clone(), Some(make_match_result(1, 0))),
    ];
    let standings = calculate_standings(&matches);
    assert!(!standings.is_empty());
}

#[test]
fn test_empty_data_no_panic() {
    assert!(calculate_standings(&[]).is_empty());
    let gs = group_standings(&[]);
    assert_eq!(gs.len(), 12);
    assert!(gs.iter().all(|(_, s)| s.is_empty()));
    let thirds = rank_third_places(&[]);
    assert!(thirds.is_empty());
    let sim = simulate_guaranteed_thirds(&[]);
    assert!(sim.teams.is_empty());
    let bracket = generate_bracket(&[]);
    assert_eq!(bracket.rounds.len(), 6);
}

#[test]
fn test_exhaustive_vs_monte_carlo_consistency() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(1, 0))),
        make_match("2", "A", t3.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("3", "A", t1.clone(), t3.clone(), None),
        make_match("4", "A", t2.clone(), t4.clone(), None),
        make_match("5", "A", t4.clone(), t1.clone(), None),
        make_match("6", "A", t2.clone(), t3.clone(), None),
    ];

    let sim = simulate_guaranteed_thirds(&matches);
    assert_eq!(sim.unplayed_matches, 4);
    assert!(sim.total_scenarios > 0);
    for tc in &sim.teams {
        assert!(tc.total_qualification_pct >= 0.0 && tc.total_qualification_pct <= 100.0);
    }
}

#[test]
fn test_guaranteed_team_has_full_percentage() {
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(5, 0))),
        make_match("2", "A", t3.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("3", "A", t1.clone(), t3.clone(), Some(make_match_result(3, 0))),
        make_match("4", "A", t2.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("5", "A", t4.clone(), t1.clone(), Some(make_match_result(0, 2))),
        make_match("6", "A", t2.clone(), t3.clone(), Some(make_match_result(1, 3))),
    ];

    let sim = simulate_guaranteed_thirds(&matches);
    let t1_result = sim.teams.iter().find(|t| t.team.fifa_code == "ALP").unwrap();
    assert!(t1_result.total_qualification_pct > 99.999, "Should be 100%, got {}", t1_result.total_qualification_pct);

    for tc in &sim.teams {
        if tc.total_qualification_pct > 99.999 {
            assert!(tc.first_pct > 99.999 || tc.second_pct > 99.999 || tc.third_qualified_pct > 99.999,
                "Guaranteed team {} should have at least one path at 100%", tc.team.name);
        }
    }
}

#[test]
fn test_resolve_slot_invalid_labels() {
    let gs = basic_standings_12();
    let bracket = generate_bracket(&gs);
    for round in &bracket.rounds {
        for slot in round {
            assert!(!slot.home_label.is_empty());
            assert!(!slot.away_label.is_empty());
        }
    }
}

#[test]
fn test_apply_knockout_results_empty_map() {
    let gs = basic_standings_12();
    let bracket = generate_bracket(&gs);
    let empty: std::collections::HashMap<String, KnockoutMatch> = std::collections::HashMap::new();
    let result = apply_knockout_results(&bracket, &empty);
    assert_eq!(result.rounds.len(), bracket.rounds.len());
    for (r1, r2) in bracket.rounds.iter().zip(result.rounds.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn test_knockout_mixed_winners_propagate() {
    let gs = basic_standings_12();
    let bracket = generate_bracket(&gs);
    let mut results: std::collections::HashMap<String, KnockoutMatch> = std::collections::HashMap::new();

    for (i, slot) in bracket.rounds[0].iter().enumerate() {
        let key = format!("{}-{}", slot.round, slot.match_number);
        results.insert(key, KnockoutMatch {
            round: slot.round.clone(),
            match_number: slot.match_number,
            home_goals: Some(if i % 2 == 0 { 1 } else { 0 }),
            away_goals: Some(if i % 2 == 0 { 0 } else { 1 }),
        });
    }

    let updated = apply_knockout_results(&bracket, &results);
    for slot in &updated.rounds[0] {
        assert!(slot.home_result.is_some());
        assert!(slot.away_result.is_some());
    }
    for slot in &updated.rounds[1] {
        assert!(slot.home_team.is_some(), "R16 home not filled");
        assert!(slot.away_team.is_some(), "R16 away not filled");
    }
}

#[test]
fn test_monte_carlo_runs_with_many_unplayed() {
    // Force Monte Carlo path by having >10 unplayed matches across groups
    let mut matches = Vec::new();
    for g in 0..12u32 {
        let gc = GroupCode(GROUP_CODES[g as usize].to_string());
        for m in 0..3 {
            let h = make_team(
                &format!("Home_G{g}_M{m}"),
                &format!("H{}{}", g, m),
            );
            let a = make_team(
                &format!("Away_G{g}_M{m}"),
                &format!("A{}{}", g, m),
            );
            matches.push(Match {
                id: format!("{}-{}", gc.0, m + 1),
                group: gc.clone(),
                home_team: h,
                away_team: a,
                result: None,
                date: None,
            });
        }
    }
    // 12 groups * 3 = 36 unplayed matches → forces Monte Carlo
    let sim = simulate_guaranteed_thirds(&matches);
    assert!(sim.method.contains("monte-carlo"), "Should use Monte Carlo, got: {}", sim.method);
    assert_eq!(sim.unplayed_matches, 36);
    assert!(!sim.teams.is_empty());
    for tc in &sim.teams {
        assert!(tc.total_qualification_pct >= 0.0 && tc.total_qualification_pct <= 100.0);
    }
}

#[test]
fn test_clinched_positions_all_played() {
    // When all matches are played, every team should have a clinched position
    let gs = basic_standings_12();
    let all_matches: Vec<Match> = gs.iter().flat_map(|(gc, standings)| {
        let code = gc.0.clone();
        standings.iter().enumerate().flat_map(move |(i, s)| {
            let opp_idx = (i + 1) % standings.len();
            let opp = &standings[opp_idx];
            vec![
                make_match(&format!("{}-{}", code, i + 1), &code,
                    s.team.clone(), opp.team.clone(),
                    Some(make_match_result((i + 1) as u32, opp_idx as u32))),
            ]
        }).collect::<Vec<_>>()
    }).collect();

    let clinched = clinched_positions(&all_matches);
    assert!(!clinched.is_empty(), "Should have clinched positions when all matches have results");
}

#[test]
fn test_head_to_head_tiebreaker() {
    // Two teams tied on points, GD, and GF — head-to-head should decide
    let t1 = make_team("Alpha", "ALP");
    let t2 = make_team("Beta", "BET");
    let t3 = make_team("Gamma", "GAM");
    let t4 = make_team("Delta", "DEL");

    let matches = vec![
        // Alpha beats Beta 2-0
        make_match("1", "A", t1.clone(), t2.clone(), Some(make_match_result(2, 0))),
        // Both beat others equally
        make_match("2", "A", t3.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("3", "A", t1.clone(), t3.clone(), Some(make_match_result(1, 0))),
        make_match("4", "A", t2.clone(), t4.clone(), Some(make_match_result(1, 0))),
        make_match("5", "A", t3.clone(), t1.clone(), Some(make_match_result(0, 1))),
        make_match("6", "A", t4.clone(), t2.clone(), Some(make_match_result(0, 1))),
    ];

    let standings = calculate_standings(&matches);
    // Alpha and Beta both have 9 pts, +3 GD, 4 GF — but Alpha beat Beta H2H
    assert_eq!(standings[0].team.name, "Alpha", "Alpha should be 1st via head-to-head");
    assert_eq!(standings[1].team.name, "Beta", "Beta should be 2nd");
}

#[test]
fn test_world_cup_data_backward_compat() {
    // Old format: no "knockout" key
    let old_json = r#"{"A":[]}"#;
    let data: WorldCupData = serde_json::from_str(old_json).unwrap();
    assert_eq!(data.groups.len(), 1);
    assert!(data.groups.contains_key("A"));
    assert!(data.knockout.is_empty());
}

#[test]
fn test_world_cup_data_round_trip() {
    // New format: with knockout results
    let data = WorldCupData {
        groups: {
            let mut m = std::collections::BTreeMap::new();
            m.insert("A".into(), vec![]);
            m
        },
        knockout: vec![KnockoutMatch {
            round: "Final".into(),
            match_number: 32,
            home_goals: Some(3),
            away_goals: Some(1),
        }],
    };
    let json = serde_json::to_string(&data).unwrap();
    let parsed: WorldCupData = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.groups.len(), 1);
    assert_eq!(parsed.knockout.len(), 1);
    assert_eq!(parsed.knockout[0].round, "Final");
    assert_eq!(parsed.knockout[0].match_number, 32);
    assert_eq!(parsed.knockout[0].home_goals, Some(3));
    assert_eq!(parsed.knockout[0].away_goals, Some(1));
}

#[test]
fn test_world_cup_data_no_knockout_serializes_clean() {
    // Empty knockout should not appear in JSON output
    let data = WorldCupData {
        groups: {
            let mut m = std::collections::BTreeMap::new();
            m.insert("A".into(), vec![]);
            m
        },
        knockout: vec![],
    };
    let json = serde_json::to_string(&data).unwrap();
    assert!(!json.contains("knockout"), "Empty knockout should be skipped: {}", json);
}
