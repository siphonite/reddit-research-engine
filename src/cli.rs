use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "reddit-research-cli")]
#[command(about = "Turn Reddit discussions into actionable startup ideas")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Analyze a single Reddit post URL
    Analyze {
        /// Reddit post URL
        url: String,

        /// Number of top comments to include
        #[arg(long, default_value_t = 10)]
        comments: usize,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Save output to file
        #[arg(long)]
        save: Option<String>,
    },

    /// Process multiple Reddit URLs from a file
    Batch {
        /// Path to file containing one URL per line
        file: String,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Save output to file
        #[arg(long)]
        save: Option<String>,
    },

    /// Analyze hot posts from a subreddit
    Subreddit {
        /// Subreddit name (without r/)
        name: String,

        /// Number of posts to fetch
        #[arg(long, default_value_t = 5)]
        limit: usize,

        /// Number of top comments per post
        #[arg(long, default_value_t = 10)]
        comments: usize,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Save output to file
        #[arg(long)]
        save: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}
