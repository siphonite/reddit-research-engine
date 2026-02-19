mod cli;
mod config;
mod errors;
mod models;
mod output;
mod services;
mod utils;

use clap::Parser;
use cli::{Cli, Command};
use errors::AppError;
use models::AnalysisResult;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = config::AppConfig::load();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client");

    if let Err(e) = run(cli.command, &client, &config.gemini_api_key).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(
    command: Command,
    client: &reqwest::Client,
    api_key: &str,
) -> Result<(), AppError> {
    match command {
        Command::Analyze {
            url,
            comments,
            format,
            save,
        } => {
            let clean_url = utils::validation::validate_reddit_url(&url)?;
            let post = services::reddit::fetch_reddit_post(client, &clean_url, comments).await?;
            let ideas = services::gemini::generate_ideas(client, api_key, &post).await?;
            let results = vec![AnalysisResult {
                url: post.url,
                title: post.title,
                ideas,
            }];
            emit(&results, &format, save.as_deref())?;
        }
        Command::Batch { file, format, save } => {
            let content = std::fs::read_to_string(&file)
                .map_err(|e| AppError::Io(format!("Failed to read {}: {}", file, e)))?;
            let mut results = Vec::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let clean_url = utils::validation::validate_reddit_url(line)?;
                eprintln!("Processing: {}", clean_url);
                let post =
                    services::reddit::fetch_reddit_post(client, &clean_url, 10).await?;
                let ideas = services::gemini::generate_ideas(client, api_key, &post).await?;
                results.push(AnalysisResult {
                    url: post.url,
                    title: post.title,
                    ideas,
                });
            }
            emit(&results, &format, save.as_deref())?;
        }
        Command::Subreddit {
            name,
            limit,
            comments,
            format,
            save,
        } => {
            eprintln!("Fetching hot posts from r/{}...", name);
            let urls = services::reddit::fetch_subreddit_posts(client, &name, limit).await?;
            let mut results = Vec::new();
            for url in &urls {
                eprintln!("Processing: {}", url);
                let post =
                    services::reddit::fetch_reddit_post(client, url, comments).await?;
                let ideas = services::gemini::generate_ideas(client, api_key, &post).await?;
                results.push(AnalysisResult {
                    url: post.url,
                    title: post.title,
                    ideas,
                });
            }
            emit(&results, &format, save.as_deref())?;
        }
    }
    Ok(())
}

fn emit(
    results: &[AnalysisResult],
    format: &cli::OutputFormat,
    save: Option<&str>,
) -> Result<(), AppError> {
    let text = output::format_results(results, format);
    println!("{}", text);
    if let Some(path) = save {
        std::fs::write(path, &text)
            .map_err(|e| AppError::Io(format!("Failed to write {}: {}", path, e)))?;
        eprintln!("Output saved to {}", path);
    }
    Ok(())
}
