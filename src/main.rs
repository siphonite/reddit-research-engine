mod cli;
mod config;
mod errors;
mod export;
mod models;
mod output;
mod services;
mod utils;

use clap::Parser;
use cli::{Cli, Command};
use config::AppConfig;
use errors::AppError;
use models::{parse_ideas, format_ideas_text, extract_subreddit, AnalysisResult};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = AppConfig::load();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to build HTTP client");

    if let Err(e) = run(cli.command, &client, &config).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(
    command: Command,
    client: &reqwest::Client,
    config: &AppConfig,
) -> Result<(), AppError> {
    match command {
        Command::Analyze {
            url,
            comments,
            format,
            save,
        } => {
            let clean_url = utils::validation::validate_reddit_url(&url)?;
            let subreddit = extract_subreddit(&clean_url);
            let post = services::reddit::fetch_reddit_post(client, &clean_url, comments).await?;
            let raw_ideas = services::gemini::generate_ideas(client, &config.gemini_api_key, &post).await?;
            let ideas = parse_ideas(&raw_ideas);
            let ideas_text = if ideas.is_empty() { raw_ideas.clone() } else { format_ideas_text(&ideas) };

            export_to_sheets(config, &subreddit, &post.url, &post.title, &ideas).await;

            let results = vec![AnalysisResult {
                url: post.url,
                title: post.title,
                ideas_text,
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
                let subreddit = extract_subreddit(&clean_url);
                eprintln!("Processing: {}", clean_url);
                let post =
                    services::reddit::fetch_reddit_post(client, &clean_url, 10).await?;
                let raw_ideas = services::gemini::generate_ideas(client, &config.gemini_api_key, &post).await?;
                let ideas = parse_ideas(&raw_ideas);
                let ideas_text = if ideas.is_empty() { raw_ideas.clone() } else { format_ideas_text(&ideas) };

                export_to_sheets(config, &subreddit, &post.url, &post.title, &ideas).await;

                results.push(AnalysisResult {
                    url: post.url,
                    title: post.title,
                    ideas_text,
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
                let raw_ideas = services::gemini::generate_ideas(client, &config.gemini_api_key, &post).await?;
                let ideas = parse_ideas(&raw_ideas);
                let ideas_text = if ideas.is_empty() { raw_ideas.clone() } else { format_ideas_text(&ideas) };

                // In subreddit mode, we already know the subreddit name
                export_to_sheets(config, &name, &post.url, &post.title, &ideas).await;

                results.push(AnalysisResult {
                    url: post.url,
                    title: post.title,
                    ideas_text,
                    ideas,
                });
            }
            emit(&results, &format, save.as_deref())?;
        }
    }
    Ok(())
}

/// Export ideas to Google Sheets if configured. Prints error but never crashes.
async fn export_to_sheets(
    config: &AppConfig,
    subreddit: &str,
    post_url: &str,
    post_title: &str,
    ideas: &[models::Idea],
) {
    if !config.sheets_enabled() || ideas.is_empty() {
        return;
    }

    let sheet_id = config.google_sheet_id.as_deref().unwrap();
    let creds_path = config.google_credentials_path.as_deref().unwrap();

    match export::sheets::append_ideas_batch(sheet_id, creds_path, subreddit, post_url, post_title, ideas).await {
        Ok(()) => eprintln!("✅ Exported {} ideas to Google Sheet", ideas.len()),
        Err(e) => eprintln!("⚠️  Sheet export failed (continuing): {}", e),
    }
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
