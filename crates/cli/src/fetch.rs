use copa2026_core::*;
use std::collections::HashMap;

pub async fn fetch_data() -> Result<HashMap<String, Vec<Match>>, String> {
    let url = "https://raw.githubusercontent.com/openfootball/world-cup/refs/heads/master/2026--canada-mexico-usa/cup.json";

    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Read error: {}", e))?;

    parse_openfootball(&body)
}

fn parse_openfootball(json: &str) -> Result<HashMap<String, Vec<Match>>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))?;

    let mut data: HashMap<String, Vec<Match>> = HashMap::new();

    let rounds = v["rounds"]
        .as_array()
        .ok_or("Missing 'rounds' array")?;

    let empty_vec = vec![];

    for round in rounds {
        let groups = round["groups"].as_array().unwrap_or(&empty_vec);
        for group in groups {
            let group_code = group["name"].as_str().unwrap_or("?").to_string();
            let matches = group["matches"].as_array().unwrap_or(&empty_vec);
            for m in matches {
                let team1 = m["team1"]["name"].as_str().unwrap_or("TBD");
                let team2 = m["team2"]["name"].as_str().unwrap_or("TBD");
                let code1 = m["team1"]["code"].as_str().unwrap_or(team1);
                let code2 = m["team2"]["code"].as_str().unwrap_or(team2);

                let home_goals = m["score1"].as_u64();
                let away_goals = m["score2"].as_u64();

                let result = match (home_goals, away_goals) {
                    (Some(h), Some(a)) => Some(MatchResult {
                        home_goals: h as u32,
                        away_goals: a as u32,
                    }),
                    _ if m["score1"].is_null() || m["score2"].is_null() => None,
                    _ => None,
                };

                let match_num = data.entry(group_code.clone()).or_default().len() + 1;
                let match_id = format!("{group_code}-{match_num}");

                let c_match = Match {
                    id: match_id,
                    group: GroupCode(group_code.clone()),
                    home_team: Team {
                        name: team1.to_string(),
                        fifa_code: code1.to_string(),
                    },
                    away_team: Team {
                        name: team2.to_string(),
                        fifa_code: code2.to_string(),
                    },
                    result,
                    date: m["date"].as_str().map(|s| s.to_string()),
                };

                data.entry(group_code.clone()).or_default().push(c_match);
            }
        }
    }

    Ok(data)
}
