use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RedditPost {
    pub url: String,
    pub title: String,
    pub body: String,
    pub comments: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Idea {
    pub product_name: String,
    pub target_user: String,
    pub core_problem: String,
    pub mvp_features: Vec<String>,
    pub monetization: String,
    pub feasibility: String,
}

#[derive(Serialize)]
pub struct AnalysisResult {
    pub url: String,
    pub title: String,
    pub ideas_text: String,
    pub ideas: Vec<Idea>,
}

/// Parse a JSON array of ideas from Gemini's response text.
/// Gemini may wrap JSON in markdown fences like ```json ... ```
pub fn parse_ideas(raw: &str) -> Vec<Idea> {
    let trimmed = raw.trim();

    // Strip markdown code fences if present (```json ... ```)
    let stripped = if trimmed.starts_with("```") {
        let after_open = match trimmed.find('\n') {
            Some(pos) => &trimmed[pos + 1..],
            None => trimmed,
        };
        after_open
            .trim_end()
            .strip_suffix("```")
            .unwrap_or(after_open)
            .trim()
    } else {
        trimmed
    };

    // Find the JSON array boundaries: first '[' to last ']'
    // This handles Gemini adding extra text before or after the array
    let json_str = match (stripped.find('['), stripped.rfind(']')) {
        (Some(start), Some(end)) if start < end => &stripped[start..=end],
        _ => return Vec::new(),
    };

    serde_json::from_str::<Vec<Idea>>(json_str).unwrap_or_default()
}

/// Format parsed ideas back into readable text for CLI output.
pub fn format_ideas_text(ideas: &[Idea]) -> String {
    if ideas.is_empty() {
        return String::from("(No structured ideas parsed)");
    }

    let mut out = String::new();
    for (i, idea) in ideas.iter().enumerate() {
        if i > 0 {
            out.push_str("\n---\n\n");
        }
        out.push_str(&format!("### {}. {}\n\n", i + 1, idea.product_name));
        out.push_str(&format!("**Target User:** {}\n\n", idea.target_user));
        out.push_str(&format!("**Core Problem:** {}\n\n", idea.core_problem));
        out.push_str("**MVP Features:**\n");
        for feat in &idea.mvp_features {
            out.push_str(&format!("  - {}\n", feat));
        }
        out.push_str(&format!("\n**Monetization:** {}\n\n", idea.monetization));
        out.push_str(&format!(
            "**Why Feasible for Solo Builder:** {}\n",
            idea.feasibility
        ));
    }
    out
}

/// Extract subreddit name from a Reddit URL using path segments.
pub fn extract_subreddit(url: &str) -> String {
    // URL format: https://www.reddit.com/r/SubredditName/comments/...
    let parts: Vec<&str> = url.split('/').collect();
    for (i, segment) in parts.iter().enumerate() {
        if *segment == "r" {
            if let Some(name) = parts.get(i + 1) {
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    "unknown".to_string()
}
