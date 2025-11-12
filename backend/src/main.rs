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
    let mut url = payload.url.clone();
    if !url.ends_with(".json") {
        url.push_str(".json");
    }

    let client = reqwest::Client::new();

    let response = client
        .get(&url)
        .header("User-Agent", "reddit-idea-generator/0.1")
        .send()
        .await
        .map_err(|_| axum::http::StatusCode::BAD_GATEWAY)?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;

    let post_data = data[0]["data"]["children"][0]["data"].clone();

    let title = post_data["title"].as_str().unwrap_or("No title").to_string();
    let body = post_data["selftext"].as_str().unwrap_or("No text").to_string();

    let result = serde_json::json!({
        "title": title,
        "body": body,
    });

    Ok(Json(result))
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
