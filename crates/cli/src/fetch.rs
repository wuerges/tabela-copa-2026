use copa2026_core::*;
use std::collections::HashMap;

const URL: &str = "https://raw.githubusercontent.com/openfootball/worldcup/refs/heads/master/2026--usa/cup.txt";

pub async fn fetch_data() -> Result<HashMap<String, Vec<Match>>, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(URL)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Read error: {}", e))?;

    parse_football_txt(&body)
}

fn parse_football_txt(content: &str) -> Result<HashMap<String, Vec<Match>>, String> {
    let mut data: HashMap<String, Vec<Match>> = HashMap::new();
    let mut current_group: Option<String> = None;
    let mut match_idx: HashMap<String, usize> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('=') {
            continue;
        }

        if trimmed.starts_with("Group ") && trimmed.contains('|') {
            let parts: Vec<&str> = trimmed.split('|').collect();
            if parts.len() == 2 {
                let group_code = parts[0]
                    .trim()
                    .strip_prefix("Group ")
                    .unwrap_or("?")
                    .trim()
                    .to_string();
                data.entry(group_code).or_default();
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('\u{25aa}') {
            let rest = rest.trim();
            if rest.starts_with("Group ") {
                current_group = Some(rest.strip_prefix("Group ").unwrap_or("?").trim().to_string());
            } else {
                current_group = None;
            }
            continue;
        }

        if let Some(ref group) = current_group {
            if let Some((home_name, away_name, result)) = parse_match_line(trimmed) {
                let home_code = make_code(&home_name);
                let away_code = make_code(&away_name);

                let idx = match_idx.entry(group.clone()).or_insert(0);
                *idx += 1;
                let match_id = format!("{}-{}", group, idx);

                let m = Match {
                    id: match_id,
                    group: GroupCode(group.clone()),
                    home_team: Team {
                        name: home_name,
                        fifa_code: home_code,
                    },
                    away_team: Team {
                        name: away_name,
                        fifa_code: away_code,
                    },
                    result,
                    date: None,
                };

                data.entry(group.clone()).or_default().push(m);
            }
        }
    }

    let total: usize = data.values().map(|v| v.len()).sum();
    if total == 0 {
        return Err("No matches parsed from the data".into());
    }

    Ok(data)
}

fn make_code(name: &str) -> String {
    let code: &str = match name {
        "Austria" => "AUT",
        "Australia" => "AUS",
        "Iran" => "IRN",
        "Iraq" => "IRQ",
        "South Korea" => "KOR",
        "South Africa" => "RSA",
        "Czech Republic" => "CZE",
        "Bosnia & Herzegovina" => "BIH",
        "Bosnia and Herzegovina" => "BIH",
        "Ivory Coast" => "CIV",
        "Côte d'Ivoire" => "CIV",
        "Cape Verde" => "CPV",
        "Cabo Verde" => "CPV",
        "DR Congo" => "COD",
        "Congo DR" => "COD",
        "Saudi Arabia" => "KSA",
        "United States" | "USA" => "USA",
        "United Arab Emirates" | "UAE" => "UAE",
        "New Zealand" => "NZL",
        "North Korea" => "PRK",
        _ => "",
    };

    if !code.is_empty() {
        return code.to_string();
    }

    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .collect();
    if cleaned.is_empty() {
        let fallback: String = name.chars().take(3).collect();
        return fallback.to_uppercase();
    }
    if cleaned.len() <= 3 {
        cleaned.to_uppercase()
    } else {
        cleaned[..3].to_uppercase().to_string()
    }
}

fn parse_match_line(line: &str) -> Option<(String, String, Option<MatchResult>)> {
    let utc_pos = line.find("UTC")?;

    let after_utc = &line[utc_pos + 3..];
    let content = after_utc
        .trim_start_matches(|c: char| c == '-' || c == '+' || c.is_ascii_digit())
        .trim();

    let at_pos = content.find('@').unwrap_or(content.len());
    let match_part = content[..at_pos].trim();

    if match_part.is_empty() {
        return None;
    }

    let re = regex_lite::Regex::new(r"(\d+)-(\d+)").ok()?;
    if let Some(score_match) = re.find(match_part) {
        let score_str = score_match.as_str();
        let home_name = match_part[..score_match.start()].trim().to_string();
        let after_score = match_part[score_match.end()..].trim();
        let parentheses_end = after_score.find(')').map(|p| p + 1);
        let away_start = parentheses_end.unwrap_or(0);
        let away_name = after_score[away_start..].trim().to_string();

        let goals: Vec<&str> = score_str.split('-').collect();
        let result = if goals.len() == 2 {
            if let (Ok(h), Ok(a)) = (goals[0].parse::<u32>(), goals[1].parse::<u32>()) {
                Some(MatchResult { home_goals: h, away_goals: a })
            } else {
                None
            }
        } else {
            None
        };

        Some((home_name, away_name, result))
    } else {
        let parts: Vec<&str> = match_part.splitn(2, "  v  ").collect();
        if parts.len() == 2 {
            Some((parts[0].trim().to_string(), parts[1].trim().to_string(), None))
        } else {
            let parts: Vec<&str> = match_part.splitn(2, " v ").collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string(), None))
            } else {
                None
            }
        }
    }
}
