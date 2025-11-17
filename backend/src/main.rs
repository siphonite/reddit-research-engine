use axum::{routing::{get, post}, Router, Json};
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
use std::env;
use axum::response::Html;

#[derive(Deserialize)]
struct AnalyzeRequest {
    url: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
} 

async fn root_handler() -> Html<&'static str> {
    Html(r#"
        <!DOCTYPE html>
        <html>
        <head><title>Reddit Ideas API</title></head>
        <body>
            <h1>Reddit Ideas Generator API</h1>
            <p>Status: Running ✅</p>
            <p>Available endpoints:</p>
            <ul>
                <li>GET /health - Health check</li>
                <li>POST /analyze_post - Analyze Reddit posts</li>
            </ul>
        </body>
        </html>
    "#)
}

async fn health_handler() -> Json<HealthResponse> {
    let response = HealthResponse { status: "OK" };
    Json(response)
}

async fn analyze_post_handler(
    Json(payload): Json<AnalyzeRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {

    // --- 1. Clean Reddit URL ---
    let mut url = payload.url.split('?').next().unwrap_or("").to_string();

    if url.ends_with('/') {
        url.pop();
    }

    url.push_str(".json");

    // --- 2. Request Reddit ---
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "reddit-idea-generator/0.1")
        .send()
        .await
        .map_err(|e| {
            eprintln!("Reddit API error: {}", e);
            (
                axum::http::StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "Failed to contact Reddit. Check the URL."
                })),
            )
        })?;

// --- Read raw text first (Reddit may return HTML instead of JSON) ---
    let text = response
    .text()
    .await
    .map_err(|e| {
        eprintln!("Failed reading Reddit response text: {}", e);
        (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({
                "error": "Could not read Reddit response."
            }))
        )
    })?;

// --- Attempt to parse JSON manually ---
    let data: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
    eprintln!("Reddit did not return JSON. Raw response:\n{}", text);

    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": "Reddit did not return valid JSON. The post may be private, removed, NSFW, blocked in your region, or require login."
        }))
    )
})?;


    // --- 4. Extract content safely ---
    let post = &data[0]["data"]["children"][0]["data"];
    let title = post["title"].as_str().unwrap_or("No title").to_string();
    let body = post["selftext"].as_str().unwrap_or("No text").to_string();

    // --- 5. Build prompt ---
    let prompt = format!(
        "You are an expert startup mentor. Read this Reddit post and generate 3 potential startup ideas with short explanations.\n\nTitle: {}\n\nBody: {}",
        title, body
    );

    // --- 6. Call Gemini ---
    let ai_response = match call_gemini_api(&prompt).await {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Gemini API error: {}", e);
            format!("LLM call failed: {}", e)
        }
    };

    // --- 7. Response ---
    let result = serde_json::json!({
        "title": title,
        "body": body,
        "ideas": ai_response
    });

    Ok(Json(result))
}


async fn call_gemini_api(prompt: &str) -> Result<String, anyhow::Error> {
    dotenvy::dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY not found in environment"))?;
    
    // Try multiple models in order of preference
    let models = vec![
        "gemini-2.5-flash",           // Primary: stable and fast
        "gemini-flash-latest",        // Backup 1: latest flash
        "gemini-2.5-flash-lite",      // Backup 2: lighter version
        "gemini-2.0-flash",           // Backup 3: older stable
    ];

    let payload = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    let client = reqwest::Client::new();
    
    // Try each model
    for (i, model) in models.iter().enumerate() {
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
            .await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Request failed for {}: {}", model, e);
                    continue;
                }
            };

        let status = res.status();
        
        // If overloaded (503) or rate limited (429), try next model
        if status == 503 || status == 429 {
            eprintln!("{} is overloaded/rate-limited ({}), trying next model...", model, status);
            if i < models.len() - 1 {
                continue;
            }
        }
        
        if !status.is_success() {
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!("API error ({}): {}", status, error_text);
            if i < models.len() - 1 {
                continue;
            }
            return Err(anyhow::anyhow!("All models failed. Last error {}: {}", status, error_text));
        }

        let data: serde_json::Value = res.json().await?;
        
        // Extract text
        let text = data
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to extract text from response"))?
            .to_string();

        eprintln!("Successfully got response from {}", model);
        return Ok(text);
    }

    Err(anyhow::anyhow!("All models are currently unavailable"))
}

#[tokio::main]
async fn main() {
    // Build our application with some routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/analyze_post", post(analyze_post_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    // === FIX: Read port from Railway ===
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    // === FIX: Bind to 0.0.0.0 instead of 127.0.0.1 ===
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Server running on http://{}", addr);

    // Use the dynamic bind
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
