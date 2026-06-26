use std::collections::BTreeMap;

pub mod models;
pub mod standings;
pub mod bracket;
pub mod simulation;

pub use models::*;
pub use standings::*;
pub use bracket::*;
pub use simulation::*;

pub type WorldCupData = BTreeMap<String, Vec<Match>>;

pub fn load_data(path: &str) -> Result<WorldCupData, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
}

pub fn save_data(data: &WorldCupData, path: &str) -> Result<(), String> {
    let content = serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path, e))
}
