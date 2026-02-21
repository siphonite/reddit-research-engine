use crate::errors::AppError;
use crate::models::Idea;

use chrono::Utc;
use google_sheets4::api::ValueRange;
use google_sheets4::Sheets;

/// Append a batch of ideas as rows to Google Sheet in a single API call.
///
/// Each row contains 10 columns:
/// Date | Subreddit | Post URL | Post Title | Product Name |
/// Target User | Core Problem | MVP Features | Monetization | Feasibility
pub async fn append_ideas_batch(
    sheet_id: &str,
    credentials_path: &str,
    subreddit: &str,
    post_url: &str,
    post_title: &str,
    ideas: &[Idea],
) -> Result<(), AppError> {
    let client = build_sheets_client(credentials_path).await?;

    let timestamp = Utc::now().to_rfc3339();

    let rows: Vec<Vec<serde_json::Value>> = ideas
        .iter()
        .map(|idea| {
            vec![
                serde_json::Value::String(timestamp.clone()),
                serde_json::Value::String(subreddit.to_string()),
                serde_json::Value::String(post_url.to_string()),
                serde_json::Value::String(post_title.to_string()),
                serde_json::Value::String(idea.product_name.clone()),
                serde_json::Value::String(idea.target_user.clone()),
                serde_json::Value::String(idea.core_problem.clone()),
                serde_json::Value::String(idea.mvp_features.join("; ")),
                serde_json::Value::String(idea.monetization.clone()),
                serde_json::Value::String(idea.feasibility.clone()),
            ]
        })
        .collect();

    let value_range = ValueRange {
        range: Some("Sheet1!A:J".to_string()),
        major_dimension: Some("ROWS".to_string()),
        values: Some(rows),
    };

    client
        .spreadsheets()
        .values_append(value_range, sheet_id, "Sheet1!A:J")
        .value_input_option("USER_ENTERED")
        .insert_data_option("INSERT_ROWS")
        .doit()
        .await
        .map_err(|e| AppError::SheetsExport(format!("Failed to append rows: {}", e)))?;

    Ok(())
}

async fn build_sheets_client(
    credentials_path: &str,
) -> Result<Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>, AppError> {
    let secret = yup_oauth2::read_service_account_key(credentials_path)
        .await
        .map_err(|e| {
            AppError::SheetsExport(format!(
                "Failed to read credentials from '{}': {}",
                credentials_path, e
            ))
        })?;

    let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .map_err(|e| AppError::SheetsExport(format!("Failed to build authenticator: {}", e)))?;

    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .map_err(|e| AppError::SheetsExport(format!("Failed to load native roots: {}", e)))?
        .https_or_http()
        .enable_http1()
        .build();

    let hyper_client = hyper::Client::builder().build(connector);

    Ok(Sheets::new(hyper_client, auth))
}
