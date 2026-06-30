use copa2026_core::*;
use std::collections::BTreeMap;
use std::path::Path;

const URL: &str = "https://raw.githubusercontent.com/openfootball/worldcup/refs/heads/master/2026--usa/cup.txt";
const FINALS_URL: &str = "https://raw.githubusercontent.com/openfootball/worldcup/refs/heads/master/2026--usa/cup_finals.txt";

async fn fetch_text(url: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| format!("Read error: {}", e))
}

pub async fn fetch_data(
    cup_path: Option<&Path>,
    finals_path: Option<&Path>,
) -> Result<WorldCupData, String> {
    // Group stage: from local file or download
    let cup_body = if let Some(p) = cup_path {
        std::fs::read_to_string(p).map_err(|e| format!("read {}: {e}", p.display()))?
    } else {
        fetch_text(URL).await?
    };

    // Knockout stage: from local file or download
    let finals_body = if let Some(p) = finals_path {
        std::fs::read_to_string(p).map_err(|e| format!("read {}: {e}", p.display()))?
    } else {
        match fetch_text(FINALS_URL).await {
            Ok(body) => body,
            Err(_) => String::new(),
        }
    };

    let groups = parse_football_txt(&cup_body)?;
    let knockout = if finals_body.is_empty() {
        vec![]
    } else {
        parse_knockout_txt(&finals_body)
    };

    Ok(WorldCupData { groups, knockout })
}

fn parse_football_txt(content: &str) -> Result<BTreeMap<String, Vec<Match>>, String> {
    let mut data: BTreeMap<String, Vec<Match>> = BTreeMap::new();
    let mut current_group: Option<String> = None;
    let mut match_idx: BTreeMap<String, usize> = BTreeMap::new();

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
        // Explicitly handled: ambiguous or non-standard prefixes
        "Austria" => "AUT",
        "Australia" => "AUS",
        "Iran" => "IRN",
        "Iraq" => "IRQ",
        "South Korea" => "KOR",
        "Korea Republic" => "KOR",
        "North Korea" => "PRK",
        "Korea DPR" => "PRK",
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
        // Teams whose first-3-letters heuristic produces wrong codes
        "Netherlands" => "NED",
        "Switzerland" => "SUI",
        "China PR" => "CHN",
        "China" => "CHN",
        "Croatia" => "CRO",
        "Portugal" => "POR",
        "Spain" => "ESP",
        "Germany" => "GER",
        "Italy" => "ITA",
        "Japan" => "JPN",
        "Morocco" => "MAR",
        "Senegal" => "SEN",
        "Serbia" => "SRB",
        "Slovakia" => "SVK",
        "Slovenia" => "SVN",
        "Türkiye" | "Turkey" => "TUR",
        "Wales" => "WAL",
        "Scotland" => "SCO",
        "Paraguay" => "PAR",
        "Poland" => "POL",
        "Nigeria" => "NGA",
        "Algeria" => "ALG",
        "Denmark" => "DEN",
        "Sweden" => "SWE",
        "Greece" => "GRE",
        "Albania" => "ALB",
        "Belgium" => "BEL",
        "Canada" => "CAN",
        "Qatar" => "QAT",
        "Ecuador" => "ECU",
        "Egypt" => "EGY",
        "Ghana" => "GHA",
        "Tunisia" => "TUN",
        "Cameroon" => "CMR",
        "Romania" => "ROU",
        "Ukraine" => "UKR",
        "Finland" => "FIN",
        "Norway" => "NOR",
        "Iceland" => "ISL",
        "Hungary" => "HUN",
        "Bulgaria" => "BUL",
        "Republic of Ireland" => "IRL",
        "Ireland" => "IRL",
        "Northern Ireland" => "NIR",
        "Chile" => "CHI",
        "Colombia" => "COL",
        "Peru" => "PER",
        "Uruguay" => "URU",
        "Costa Rica" => "CRC",
        "Mexico" => "MEX",
        "Panama" => "PAN",
        "Honduras" => "HON",
        "Jamaica" => "JAM",
        "El Salvador" => "SLV",
        "Venezuela" => "VEN",
        "Bolivia" => "BOL",
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

    static SCORE_RE: std::sync::OnceLock<regex_lite::Regex> = std::sync::OnceLock::new();
    let re = SCORE_RE.get_or_init(|| regex_lite::Regex::new(r"(\d+)-(\d+)").unwrap());
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

/// Map round names from cup_finals.txt to our canonical names.
fn map_knockout_round(name: &str) -> &str {
    match name {
        "Round of 32" => "Round of 32",
        "Round of 16" => "Round of 16",
        "Quarter-final" => "Quarter-finals",
        "Semi-final" => "Semi-finals",
        "Match for third place" => "Third Place",
        "Final" => "Final",
        other => other,
    }
}

/// Parse knockout match results from cup_finals.txt format.
fn parse_knockout_txt(content: &str) -> Vec<KnockoutMatch> {
    let mut results = Vec::new();
    let mut current_round = String::new();

    // Regex to extract the match number from "(NN)"
    static MATCH_NUM_RE: std::sync::OnceLock<regex_lite::Regex> = std::sync::OnceLock::new();
    let match_num_re = MATCH_NUM_RE
        .get_or_init(|| regex_lite::Regex::new(r"^\s*\((\d+)\)").unwrap());

    // Regex for the first score in a line: "X-Y"
    static SCORE_RE: std::sync::OnceLock<regex_lite::Regex> = std::sync::OnceLock::new();
    let score_re = SCORE_RE.get_or_init(|| regex_lite::Regex::new(r"(\d+)-(\d+)").unwrap());

    // Regex for penalty score: "X-Y pen."
    static PEN_RE: std::sync::OnceLock<regex_lite::Regex> = std::sync::OnceLock::new();
    let pen_re = PEN_RE.get_or_init(|| regex_lite::Regex::new(r"(\d+)-(\d+)\s+pen\.").unwrap());

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('=') {
            continue;
        }

        // Detect round headers: "▪ Round of 32", "▪ Final", etc.
        if let Some(rest) = trimmed.strip_prefix('\u{25aa}') {
            current_round = map_knockout_round(rest.trim()).to_string();
            continue;
        }

        // Match lines have match number in parens: "(73) ..."
        // Openfootball numbers are 73-104; ours are 1-32 (offset 72).
        if let Some(caps) = match_num_re.captures(trimmed) {
            let src_num: u32 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            if src_num < 73 {
                continue;
            }
            let match_number = src_num - 72;

            // Strip trailing "## comments" and "@ venue"
            let after_paren = &trimmed[caps.get(0).unwrap().end()..];
            let clean = after_paren
                .split("##")
                .next()
                .unwrap_or("")
                .split("   @")
                .next()
                .unwrap_or("")
                .split("  @")
                .next()
                .unwrap_or("")
                .trim();

            if clean.is_empty() {
                continue;
            }

            // Check for "v" (match not yet played)
            if clean.contains(" v ") {
                continue;
            }

            // Find the first score (regulation time)
            if let Some(first_score) = score_re.find(clean) {
                let parts: Vec<&str> = first_score.as_str().split('-').collect();
                if parts.len() != 2 {
                    continue;
                }
                let home_goals: u32 = match parts[0].parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let away_goals: u32 = match parts[1].parse() {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                // Check for penalty shootout
                let (home_pen, away_pen, winner_is_home) =
                    if let Some(pen_cap) = pen_re.captures(clean) {
                    let ph: u32 = pen_cap.get(1).unwrap().as_str().parse().unwrap_or(0);
                    let pa: u32 = pen_cap.get(2).unwrap().as_str().parse().unwrap_or(0);
                    (Some(ph), Some(pa), Some(ph > pa))
                } else {
                    (None, None, None)
                };

                results.push(KnockoutMatch {
                    round: current_round.clone(),
                    match_number,
                    home_goals: Some(home_goals),
                    away_goals: Some(away_goals),
                    home_pen,
                    away_pen,
                    winner_is_home,
                });
            }
        }
    }

    results
}
