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
    let mut prompt = format!(
        "Analyze the following Reddit discussion (post + comments) \
         and extract actionable startup opportunities.\n\n\
         Title: {}\n\n\
         Body: {}\n\n",
        post.title, post.body
    );

    if !post.comments.is_empty() {
        prompt.push_str("Top Comments:\n");
        for comment in &post.comments {
            prompt.push_str(&format!("- {}\n", comment));
        }
        prompt.push('\n');
    }

    prompt.push_str("Generate 3 potential startup ideas with short explanations.");
    prompt
}
