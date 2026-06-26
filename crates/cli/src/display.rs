use comfy_table::*;
use copa2026_core::*;

pub fn print_group_table(group: &GroupCode, standings: &[Standing]) {
    let mut table = Table::new();
    table
        .set_header(vec![
            "#", "Team", "P", "W", "D", "L", "GF", "GA", "GD", "Pts",
        ])
        .set_content_arrangement(ContentArrangement::Dynamic);

    for s in standings {
        let mut row = Row::new();
        row.add_cell(Cell::new(s.position.to_string()));
        row.add_cell(Cell::new(if s.position <= 2 {
            format!("{} [Q]", s.team.name)
        } else {
            s.team.name.clone()
        }).bg(if s.position <= 2 {
            Color::Green
        } else {
            Color::Reset
        }));
        row.add_cell(Cell::new(s.played.to_string()));
        row.add_cell(Cell::new(s.won.to_string()));
        row.add_cell(Cell::new(s.drawn.to_string()));
        row.add_cell(Cell::new(s.lost.to_string()));
        row.add_cell(Cell::new(s.goals_for.to_string()));
        row.add_cell(Cell::new(s.goals_against.to_string()));
        row.add_cell(Cell::new(format!("{:+}", s.goal_diff)));
        row.add_cell(Cell::new(s.points.to_string()));

        table.add_row(row);
    }

    println!("Group {}", group);
    println!("{table}");
}

pub fn print_third_place_ranking(group_standings: &[(GroupCode, Vec<Standing>)]) {
    let thirds = rank_third_places(group_standings);

    let mut table = Table::new();
    table
        .set_header(vec![
            "#", "Team", "Group", "P", "W", "D", "L", "GF", "GA", "GD", "Pts",
        ])
        .set_content_arrangement(ContentArrangement::Dynamic);

    for (i, (code, s)) in thirds.iter().enumerate() {
        let mut row = Row::new();
        row.add_cell(Cell::new((i + 1).to_string()));
        row.add_cell(Cell::new(if i < 8 {
            format!("{} [Q]", s.team.name)
        } else {
            s.team.name.clone()
        }).bg(if i < 8 {
            Color::Green
        } else {
            Color::Reset
        }));
        row.add_cell(Cell::new(code.0.clone()));
        row.add_cell(Cell::new(s.played.to_string()));
        row.add_cell(Cell::new(s.won.to_string()));
        row.add_cell(Cell::new(s.drawn.to_string()));
        row.add_cell(Cell::new(s.lost.to_string()));
        row.add_cell(Cell::new(s.goals_for.to_string()));
        row.add_cell(Cell::new(s.goals_against.to_string()));
        row.add_cell(Cell::new(format!("{:+}", s.goal_diff)));
        row.add_cell(Cell::new(s.points.to_string()));

        table.add_row(row);
    }

    println!("Third Place Ranking (Top 8 qualify):");
    println!("{table}");
}

pub fn print_bracket(bracket: &Bracket) {
    for round in &bracket.rounds {
        let round_name = if round.is_empty() {
            continue;
        } else {
            &round[0].round
        };

        println!("═══ {round_name} ═══");
        println!();

        for slot in round {
            let home = slot
                .home_team
                .as_ref()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| slot.home_label.clone());
            let away = slot
                .away_team
                .as_ref()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| slot.away_label.clone());

            match (slot.home_result, slot.away_result) {
                (Some(h), Some(a)) => {
                    println!("  {home} {h} - {a} {away}");
                }
                _ => {
                    println!("  {home} vs {away}");
                }
            }
        }
        println!();
    }
}

pub fn print_simulation(sim: &ThirdPlaceSimulation) {
    println!(
        "Simulation: {} unplayed matches, {} scenarios ({}).",
        sim.unplayed_matches, sim.total_scenarios, sim.method
    );
    println!();

    if !sim.direct_qualified.is_empty() {
        println!("Classificados diretos (1o/2o lugar garantido):");
        for team in &sim.direct_qualified {
            println!("  {} ", team);
        }
        println!();
    }

    if !sim.guaranteed.is_empty() {
        println!("3o lugar garantido (100%):");
        for team in &sim.guaranteed {
            println!("  {} ", team);
        }
        println!();
    }

    if sim.guaranteed.is_empty() && sim.direct_qualified.is_empty() {
        println!("Nenhum time matematicamente garantido ainda.");
        println!();
    }

    if !sim.uncertain.is_empty() {
        println!("Incerto:");
        for uc in &sim.uncertain {
            println!(
                "  {} (Group {}) - {:.1}% - {} pts, GD {:+}",
                uc.team.name,
                uc.group.0,
                uc.percentage,
                uc.points,
                uc.goal_diff
            );
        }
        println!();
    }

    if !sim.eliminated.is_empty() {
        println!("Desqualificados (0%):");
        for team in &sim.eliminated {
            println!("  {} ", team);
        }
    }
}

pub fn print_stats(matches: &[Match]) {
    let total = matches.len();
    let played = matches.iter().filter(|m| m.result.is_some()).count();
    let unplayed = total - played;
    let total_goals: u32 = matches
        .iter()
        .filter_map(|m| m.result.as_ref())
        .map(|r| r.home_goals + r.away_goals)
        .sum();
    let draws = matches
        .iter()
        .filter(|m| m.result.as_ref().map(|r| r.outcome() == MatchOutcome::Draw).unwrap_or(false))
        .count();

    println!("World Cup 2026 - Group Stage Statistics");
    println!("───────────────────────────────────────");
    println!("Total matches:     {total}");
    println!("Played:           {played}");
    println!("Remaining:        {unplayed}");
    println!("Total goals:      {total_goals}");
    println!(
        "Goals per match:  {:.1}",
        if played > 0 {
            total_goals as f64 / played as f64
        } else {
            0.0
        }
    );
    println!("Draws:            {draws}");
}
