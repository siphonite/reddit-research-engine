use crate::errors::AppError;

/// Validates that the URL is a Reddit post URL containing `/comments/`.
/// Returns the cleaned URL (query params stripped, trailing slash removed).
pub fn validate_reddit_url(url: &str) -> Result<String, AppError> {
    let trimmed = url.trim();

    if trimmed.is_empty() {
        return Err(AppError::InvalidInput("URL cannot be empty".into()));
    }

    // Must be a reddit.com domain
    let lower = trimmed.to_lowercase();
    let is_reddit = lower.contains("reddit.com/") || lower.contains("redd.it/");
    if !is_reddit {
        return Err(AppError::InvalidInput(format!(
            "Not a Reddit URL: {}",
            trimmed
        )));
    }

    // Must be a post URL (contains /comments/)
    if !trimmed.contains("/comments/") {
        return Err(AppError::InvalidInput(format!(
            "URL must be a Reddit post (must contain /comments/): {}",
            trimmed
        )));
    }

    // Clean: strip query params, remove trailing slash
    let mut clean = trimmed.split('?').next().unwrap_or("").to_string();
    if clean.ends_with('/') {
        clean.pop();
    }

    Ok(clean)
}
