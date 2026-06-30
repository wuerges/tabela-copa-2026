use clap::{Parser, Subcommand};
use copa2026_core::*;
use std::path::PathBuf;

mod fetch;
mod display;

#[derive(Parser)]
#[command(name = "copa2026")]
#[command(about = "Classificados da Copa do Mundo 2026")]
struct Cli {
    #[arg(short, long, default_value = "data.json")]
    data_file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download match data from openfootball
    Fetch {
        /// Use local cup.txt instead of downloading
        #[arg(long)]
        cup: Option<PathBuf>,
        /// Use local cup_finals.txt instead of downloading
        #[arg(long)]
        finals: Option<PathBuf>,
    },
    /// Show group standings, optionally filtered by group
    Standings {
        #[arg(short, long)]
        group: Option<String>,
    },
    /// Show ranking of best third-placed teams
    #[command(name = "best-thirds")]
    BestThirds,
    /// Show the full knockout bracket
    Bracket,
    /// Simulate third-place qualification probabilities
    #[command(name = "guaranteed-thirds")]
    GuaranteedThirds,
    /// Show general statistics (goals, draws, etc.)
    Stats,
}

fn load(path: &std::path::Path) -> Result<Vec<Match>, String> {
    load_data(&path.to_string_lossy()).map(|d| d.groups.into_values().flatten().collect())
}

fn main() {
    let cli = Cli::parse();
    let path = cli.data_file.clone();

    let result = run(cli, &path);
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli, path: &std::path::Path) -> Result<(), String> {
    match cli.command {
        Commands::Fetch { cup, finals } => {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create async runtime: {e}"))?;
            runtime.block_on(async {
                let data = fetch::fetch_data(
                    cup.as_deref(),
                    finals.as_deref(),
                ).await?;
                save_data(&data, &path.to_string_lossy())?;
                let total: usize = data.groups.values().map(|v| v.len()).sum();
                let ko_count = data.knockout.len();
                if ko_count > 0 {
                    println!("Fetched {total} group matches across {} groups, and {ko_count} knockout results.", data.groups.len());
                } else {
                    println!("Fetched {total} group matches across {} groups.", data.groups.len());
                }
                Ok(())
            })
        }

        Commands::Standings { group } => {
            let all_matches = load(path)?;
            if all_matches.is_empty() {
                eprintln!("No matches loaded.");
                return Ok(());
            }

            if let Some(g) = group {
                let group_matches: Vec<Match> = all_matches
                    .into_iter()
                    .filter(|m| m.group.0 == g.to_uppercase())
                    .collect();
                if group_matches.is_empty() {
                    eprintln!("Group '{g}' not found.");
                    return Ok(());
                }
                let standings = calculate_standings(&group_matches);
                display::print_group_table(&GroupCode(g.to_uppercase()), &standings);
            } else {
                let gs = group_standings(&all_matches);
                for (code, standings) in &gs {
                    display::print_group_table(code, standings);
                    println!();
                }
                display::print_third_place_ranking(&gs);
            }
            Ok(())
        }

        Commands::BestThirds => {
            let all_matches = load(path)?;
            if all_matches.is_empty() {
                eprintln!("No matches loaded.");
                return Ok(());
            }
            let gs = group_standings(&all_matches);
            display::print_third_place_ranking(&gs);
            Ok(())
        }

        Commands::Bracket => {
            let all_matches = load(path)?;
            if all_matches.is_empty() {
                eprintln!("No matches loaded.");
                return Ok(());
            }
            let gs = group_standings(&all_matches);
            let bracket = generate_bracket(&gs);
            display::print_bracket(&bracket);
            Ok(())
        }

        Commands::GuaranteedThirds => {
            let all_matches = load(path)?;
            if all_matches.is_empty() {
                eprintln!("No matches loaded.");
                return Ok(());
            }
            let sim = simulate_guaranteed_thirds(&all_matches);
            display::print_simulation(&sim);
            Ok(())
        }

        Commands::Stats => {
            let all_matches = load(path)?;
            if all_matches.is_empty() {
                eprintln!("No matches loaded.");
                return Ok(());
            }
            display::print_stats(&all_matches);
            Ok(())
        }
    }
}
