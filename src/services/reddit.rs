use crate::errors::AppError;
use crate::models::RedditPost;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) RedditResearchCLI/1.0";

/// Fetches a Reddit post including top-level comments.
pub async fn fetch_reddit_post(
    client: &reqwest::Client,
    url: &str,
    max_comments: usize,
) -> Result<RedditPost, AppError> {
    let fetch_url = format!("{}.json", url);

    let response = client
        .get(&fetch_url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| {
            eprintln!("Reddit API error: {}", e);
            AppError::ExternalService("Failed to contact Reddit. Check the URL.".into())
        })?;

    let text = response.text().await.map_err(|e| {
        eprintln!("Failed reading Reddit response text: {}", e);
        AppError::ExternalService("Could not read Reddit response.".into())
    })?;

    let data: serde_json::Value = serde_json::from_str(&text).map_err(|_| {
        eprintln!("Reddit did not return JSON. Raw response:\n{}", text);
        AppError::ExternalService(
            "Reddit did not return valid JSON. The post may be private, removed, or NSFW.".into(),
        )
    })?;

    // Extract post data from [0]
    let post_data = &data[0]["data"]["children"][0]["data"];
    let title = post_data["title"].as_str().unwrap_or("No title").to_string();
    let body = post_data["selftext"].as_str().unwrap_or("No text").to_string();

    // Extract top-level comments from [1]
    let comments = extract_comments(&data[1], max_comments);

    Ok(RedditPost {
        url: url.to_string(),
        title,
        body,
        comments,
    })
}

/// Fetches hot post URLs from a subreddit.
pub async fn fetch_subreddit_posts(
    client: &reqwest::Client,
    subreddit: &str,
    limit: usize,
) -> Result<Vec<String>, AppError> {
    let url = format!(
        "https://www.reddit.com/r/{}/hot.json?limit={}",
        subreddit, limit
    );

    let response = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            eprintln!("Subreddit fetch error: {}", e);
            AppError::ExternalService(format!("Failed to fetch r/{}", subreddit))
        })?;

    let text = response.text().await.map_err(|e| {
        eprintln!("Failed reading subreddit response: {}", e);
        AppError::ExternalService("Could not read subreddit response.".into())
    })?;

    let data: serde_json::Value = serde_json::from_str(&text).map_err(|_| {
        AppError::ExternalService(format!("r/{} did not return valid JSON.", subreddit))
    })?;

    let children = data["data"]["children"]
        .as_array()
        .ok_or_else(|| AppError::ExternalService("Unexpected subreddit JSON structure.".into()))?;

    let urls: Vec<String> = children
        .iter()
        .filter_map(|child| {
            let permalink = child["data"]["permalink"].as_str()?;
            Some(format!("https://www.reddit.com{}", permalink.trim_end_matches('/')))
        })
        .collect();

    Ok(urls)
}

fn extract_comments(comment_listing: &serde_json::Value, max: usize) -> Vec<String> {
    let Some(children) = comment_listing["data"]["children"].as_array() else {
        return Vec::new();
    };

    children
        .iter()
        .filter(|c| c["kind"].as_str() == Some("t1"))
        .filter_map(|c| {
            let body = c["data"]["body"].as_str()?;
            let author = c["data"]["author"].as_str().unwrap_or("");
            // Skip deleted/removed comments
            if author == "[deleted]" || body == "[deleted]" || body == "[removed]" {
                return None;
            }
            Some(body.to_string())
        })
        .take(max)
        .collect()
}
