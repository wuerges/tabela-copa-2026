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
    Fetch,
    Standings {
        #[arg(short, long)]
        group: Option<String>,
    },
    #[command(name = "best-thirds")]
    BestThirds,
    Bracket,
    #[command(name = "guaranteed-thirds")]
    GuaranteedThirds,
    Stats,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch => {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                match fetch::fetch_data().await {
                    Ok(data) => {
                        save_data(&data, cli.data_file.to_str().unwrap()).unwrap();
                        let total: usize = data.values().map(|v| v.len()).sum();
                        println!("Fetched {} matches across {} groups.", total, data.len());
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            });
        }

        Commands::Standings { group } => {
            let data = load_data(cli.data_file.to_str().unwrap()).unwrap_or_default();
            let all_matches: Vec<Match> = data.into_values().flatten().collect();

            if let Some(g) = group {
                let group_matches: Vec<Match> = all_matches
                    .into_iter()
                    .filter(|m| m.group.0 == g.to_uppercase())
                    .collect();
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
        }

        Commands::BestThirds => {
            let data = load_data(cli.data_file.to_str().unwrap()).unwrap_or_default();
            let all_matches: Vec<Match> = data.into_values().flatten().collect();
            let gs = group_standings(&all_matches);
            display::print_third_place_ranking(&gs);
        }

        Commands::Bracket => {
            let data = load_data(cli.data_file.to_str().unwrap()).unwrap_or_default();
            let all_matches: Vec<Match> = data.into_values().flatten().collect();
            let gs = group_standings(&all_matches);
            let bracket = generate_bracket(&gs);
            display::print_bracket(&bracket);
        }

        Commands::GuaranteedThirds => {
            let data = load_data(cli.data_file.to_str().unwrap()).unwrap_or_default();
            let all_matches: Vec<Match> = data.into_values().flatten().collect();
            let sim = simulate_guaranteed_thirds(&all_matches);
            display::print_simulation(&sim);
        }

        Commands::Stats => {
            let data = load_data(cli.data_file.to_str().unwrap()).unwrap_or_default();
            let all_matches: Vec<Match> = data.into_values().flatten().collect();
            display::print_stats(&all_matches);
        }
    }
}
