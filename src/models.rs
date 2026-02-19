use serde::Serialize;

#[derive(Serialize)]
pub struct RedditPost {
    pub url: String,
    pub title: String,
    pub body: String,
    pub comments: Vec<String>,
}

#[derive(Serialize)]
pub struct AnalysisResult {
    pub url: String,
    pub title: String,
    pub ideas: String,
}
