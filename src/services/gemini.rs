use crate::errors::AppError;
use crate::models::RedditPost;

const MODELS: &[&str] = &[
    "gemini-2.5-flash",
    "gemini-flash-latest",
    "gemini-2.5-flash-lite",
    "gemini-2.0-flash",
];

pub async fn generate_ideas(
    client: &reqwest::Client,
    api_key: &str,
    post: &RedditPost,
) -> Result<String, AppError> {
    let prompt = build_prompt(post);

    let payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    for (i, model) in MODELS.iter().enumerate() {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, api_key
        );

        eprintln!("Attempting API call with model: {}", model);

        let res = match client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Request failed for {}: {}", model, e);
                continue;
            }
        };

        let status = res.status();

        if status == 503 || status == 429 {
            eprintln!(
                "{} is overloaded/rate-limited ({}), trying next model...",
                model, status
            );
            if i < MODELS.len() - 1 {
                continue;
            }
        }

        if !status.is_success() {
            let error_text = res
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!("API error ({}): {}", status, error_text);
            if i < MODELS.len() - 1 {
                continue;
            }
            return Err(AppError::ExternalService(format!(
                "All models failed. Last error {}: {}",
                status, error_text
            )));
        }

        let data: serde_json::Value = res.json().await.map_err(|e| {
            AppError::ExternalService(format!("Failed to parse Gemini response: {}", e))
        })?;

        let text = data
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                AppError::ExternalService("Failed to extract text from Gemini response".into())
            })?
            .to_string();

        eprintln!("Successfully got response from {}", model);
        return Ok(text);
    }

    Err(AppError::ExternalService(
        "All models are currently unavailable".into(),
    ))
}

fn build_prompt(post: &RedditPost) -> String {
    let mut prompt = String::from(
        "You are a pragmatic product strategist focused on small, buildable digital products.\n\n\
         Analyze the following Reddit discussion (post + comments) and identify concrete pain points, \
         frustrations, unmet needs, or repeated patterns.\n\n\
         Your task is to generate 3 highly practical micro-SaaS or small product ideas that:\n\n\
         - Can be built by a solo developer or small team\n\
         - Are realistic and narrowly scoped\n\
         - Solve a specific pain point from the discussion\n\
         - Are suitable as:\n\
         \x20 - A web app\n\
         \x20 - A mobile app\n\
         \x20 - A Chrome extension\n\
         \x20 - A lightweight SaaS tool\n\
         \x20 - A niche B2B utility\n\
         \x20 - An automation tool\n\n\
         Do NOT generate:\n\
         - Large marketplaces\n\
         - Social networks\n\
         - Venture-scale platforms\n\
         - Ideas that require massive funding\n\
         - \"Uber for X\" concepts\n\
         - Overly generic AI wrappers\n\n\
         For each idea, provide:\n\n\
         1. Product Name (short and simple)\n\
         2. Target User (very specific niche)\n\
         3. Core Problem (clearly derived from the discussion)\n\
         4. MVP Feature Set (3–6 core features only)\n\
         5. Monetization Model (subscription, one-time payment, etc.)\n\
         6. Why This Is Feasible for a Solo Builder\n\n\
         Reddit Discussion:\n\n",
    );

    prompt.push_str(&format!("Title:\n{}\n\n", post.title));
    prompt.push_str(&format!("Body:\n{}\n\n", post.body));

    if !post.comments.is_empty() {
        prompt.push_str("Top Comments:\n");
        for comment in &post.comments {
            prompt.push_str(&format!("- {}\n", comment));
        }
    }

    prompt
}
