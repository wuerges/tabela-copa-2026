use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Team {
    pub name: String,
    pub fifa_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GroupCode(pub String);

impl std::fmt::Display for GroupCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum MatchOutcome {
    HomeWin,
    AwayWin,
    Draw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub struct MatchResult {
    pub home_goals: u32,
    pub away_goals: u32,
}

impl MatchResult {
    pub fn outcome(&self) -> MatchOutcome {
        match self.home_goals.cmp(&self.away_goals) {
            std::cmp::Ordering::Greater => MatchOutcome::HomeWin,
            std::cmp::Ordering::Less => MatchOutcome::AwayWin,
            std::cmp::Ordering::Equal => MatchOutcome::Draw,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub id: String,
    pub group: GroupCode,
    pub home_team: Team,
    pub away_team: Team,
    pub result: Option<MatchResult>,
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Standing {
    pub position: u32,
    pub team: Team,
    pub played: u32,
    pub won: u32,
    pub drawn: u32,
    pub lost: u32,
    pub goals_for: u32,
    pub goals_against: u32,
    pub goal_diff: i32,
    pub points: u32,
}

pub const GROUP_CODES: &[&str] = &["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L"];
