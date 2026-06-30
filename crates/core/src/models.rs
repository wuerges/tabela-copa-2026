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
pub struct KnockoutMatch {
    pub round: String,
    pub match_number: u32,
    pub home_goals: Option<u32>,
    pub away_goals: Option<u32>,
    /// Penalty shootout scores (only set when match was decided by penalties).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_pen: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub away_pen: Option<u32>,
    /// When the match was a draw after regulation/extra time and decided by
    /// penalties, this indicates which side won. `None` for matches that
    /// ended in a true draw or were decided in regulation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner_is_home: Option<bool>,
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
