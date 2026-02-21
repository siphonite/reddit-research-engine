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
use models::{parse_ideas, format_ideas_text, extract_subreddit, AnalysisResult, Idea};

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
            let results = process_subreddit(client, config, &name, limit, comments).await?;
            emit(&results, &format, save.as_deref())?;
        }
        Command::Multi {
            subreddits,
            limit,
            comments,
            max_ideas,
            format,
            save,
        } => {
            let sub_list: Vec<String> = subreddits
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if sub_list.is_empty() {
                return Err(AppError::InvalidInput(
                    "No valid subreddit names provided".into(),
                ));
            }

            let mut all_results = Vec::new();
            let mut total_ideas: usize = 0;
            let mut total_posts: usize = 0;
            let mut failed_posts: usize = 0;
            let mut subs_processed: usize = 0;
            let mut hit_limit = false;

            for sub in &sub_list {
                eprintln!("\n📡 Scanning r/{}...", sub);
                subs_processed += 1;

                let urls = match services::reddit::fetch_subreddit_posts(client, sub, limit).await {
                    Ok(u) => u,
                    Err(e) => {
                        eprintln!("⚠️  Failed to fetch r/{}: {}", sub, e);
                        continue;
                    }
                };

                for url in &urls {
                    eprintln!("Processing: {}", url);
                    let result = process_post(client, config, sub, url, comments).await;

                    match result {
                        Ok(r) => {
                            total_posts += 1;
                            total_ideas += r.ideas.len();
                            all_results.push(r);

                            if let Some(max) = max_ideas {
                                if total_ideas >= max {
                                    hit_limit = true;
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  Failed to process post: {}", e);
                            failed_posts += 1;
                        }
                    }
                }

                if hit_limit {
                    eprintln!("\n🛑 Reached max-ideas limit ({})", max_ideas.unwrap());
                    break;
                }
            }

            emit(&all_results, &format, save.as_deref())?;

            eprintln!("\n────────────────────────────────────────");
            eprintln!("Scan complete.\n");
            eprintln!("Subreddits processed: {}", subs_processed);
            eprintln!("Posts analyzed: {}", total_posts);
            eprintln!("Ideas generated: {}", total_ideas);
            eprintln!("Posts failed: {}", failed_posts);
            eprintln!("────────────────────────────────────────");
        }
    }
    Ok(())
}

/// Process all hot posts from a single subreddit. Reused by both `subreddit` and `multi` modes.
async fn process_subreddit(
    client: &reqwest::Client,
    config: &AppConfig,
    name: &str,
    limit: usize,
    comments: usize,
) -> Result<Vec<AnalysisResult>, AppError> {
    eprintln!("Fetching hot posts from r/{}...", name);
    let urls = services::reddit::fetch_subreddit_posts(client, name, limit).await?;
    let mut results = Vec::new();

    for url in &urls {
        eprintln!("Processing: {}", url);
        let result = process_post(client, config, name, url, comments).await?;
        results.push(result);
    }

    Ok(results)
}

/// Process a single Reddit post: fetch, generate ideas, parse, and export to Sheets.
async fn process_post(
    client: &reqwest::Client,
    config: &AppConfig,
    subreddit: &str,
    url: &str,
    comments: usize,
) -> Result<AnalysisResult, AppError> {
    let post = services::reddit::fetch_reddit_post(client, url, comments).await?;
    let raw_ideas = services::gemini::generate_ideas(client, &config.gemini_api_key, &post).await?;
    let ideas = parse_ideas(&raw_ideas);
    let ideas_text = if ideas.is_empty() { raw_ideas.clone() } else { format_ideas_text(&ideas) };

    export_to_sheets(config, subreddit, &post.url, &post.title, &ideas).await;

    Ok(AnalysisResult {
        url: post.url,
        title: post.title,
        ideas_text,
        ideas,
    })
}

/// Export ideas to Google Sheets if configured. Prints error but never crashes.
async fn export_to_sheets(
    config: &AppConfig,
    subreddit: &str,
    post_url: &str,
    post_title: &str,
    ideas: &[Idea],
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
