use axum::{routing::{get, post}, Router, Json}; // imports axom types, router - builds the route tables, get and post are helpers to quickly attach handlers to HTTP methods. 
use std::net::SocketAddr;          // a small type to tell server which IP address and port to bind to.
use serde::{Deserialize, Serialize}; // required to convert Rust structs to/from JSON when sending/receiving data.
use tower_http::cors::{CorsLayer, Any}; // CORS middleware to handle cross-origin requests. 

#[derive(Deserialize)] // derive macros to automatically implement deserialization for the struct. 
struct AnalyzeRequest {  // AnalyzeRequest models the JSON body our /analyze_post endpoint will accept. For now it only needs a url field.
    url: String,
}

#[derive(Serialize)] // derive macros to automatically implement serialization for the struct.
struct HealthResponse { // HealthResponse is a tiny typed response for /health. Using typed structs makes the API predictable and easier to debug.
    status: &'static str,
}   

async fn health_handler() -> Json<HealthResponse> { // health_handler responds to GET /health requests with a simple JSON payload.
    let response = HealthResponse { status: "OK" }; // create a HealthResponse instance with status "OK".
    Json(response) // wrap it in Json to serialize it to JSON format.
}
async fn analyze_post_handler(
    Json(payload): Json<AnalyzeRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // 1. Prepare Reddit URL
    let mut url = payload.url.clone();
    if !url.ends_with(".json") {
        url.push_str(".json");
    }

    // 2. Make request to Reddit
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "reddit-idea-generator/0.1")
        .send()
        .await
        .map_err(|_| axum::http::StatusCode::BAD_GATEWAY)?;

    // 3. Parse Reddit response JSON
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    // 4. Extract post title and body text
    let post_data = data[0]["data"]["children"][0]["data"].clone();
    let title = post_data["title"].as_str().unwrap_or("No title").to_string();
    let body = post_data["selftext"].as_str().unwrap_or("No text").to_string();

    // 5. Build LLM prompt for Gemini
    let prompt = format!(
        "You are an expert startup mentor. Read this Reddit post and generate 3 potential startup ideas with short explanations.\n\nTitle: {}\n\nBody: {}",
        title, body
    );

    // 6. Call Gemini API for idea generation
    let ai_response = match call_gemini_api(&prompt).await {
        Ok(text) => text,
        Err(_) => "LLM call failed".to_string(),
    };

    // 7. Combine everything into the response
    let result = serde_json::json!({
        "title": title,
        "body": body,
        "ideas": ai_response
    });

    Ok(Json(result))
}


use std::env;

async fn call_gemini_api(prompt: &str) -> Result<String, anyhow::Error> {
    dotenvy::dotenv().ok(); // Load .env file
    let api_key = env::var("GEMINI_API_KEY")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-latest:generateContent?key={}",
        api_key
    );

    let payload = serde_json::json!({
    "contents": [
        {
            "role": "user",
            "parts": [
                { "text": prompt }
            ]
        }
    ]
   });


    let client = reqwest::Client::new();
    let res = client.post(&url).json(&payload).send().await?;
    let data: serde_json::Value = res.json().await?;

    // Extract text safely
    let text = data["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("No response")
        .to_string();

    Ok(text)
}


#[tokio::main]
async fn main() {
    // Build our application with some routes.
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/analyze_post", post(analyze_post_handler));
    // Add CORS layer to allow cross-origin requests from any origin.

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    let app = app.layer(cors);

    // Define the address to bind the server to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    // NEW version: use Tokio's TcpListener + axum::serve
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind port 3000");
    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
