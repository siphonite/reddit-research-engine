use crate::cli::OutputFormat;
use crate::models::AnalysisResult;

pub fn format_results(results: &[AnalysisResult], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_text(results),
        OutputFormat::Json => format_json(results),
        OutputFormat::Markdown => format_markdown(results),
    }
}

fn format_text(results: &[AnalysisResult]) -> String {
    let mut out = String::new();
    for (i, r) in results.iter().enumerate() {
        if i > 0 {
            out.push_str("\n════════════════════════════════════════\n\n");
        }
        out.push_str(&format!("URL: {}\n", r.url));
        out.push_str(&format!("Title: {}\n\n", r.title));
        out.push_str(&format!("Ideas:\n{}\n", r.ideas));
    }
    out
}

fn format_json(results: &[AnalysisResult]) -> String {
    serde_json::to_string_pretty(results).unwrap_or_else(|_| "[]".to_string())
}

fn format_markdown(results: &[AnalysisResult]) -> String {
    let mut out = String::from("# Reddit Startup Analysis\n\n");
    for (i, r) in results.iter().enumerate() {
        if i > 0 {
            out.push_str("---\n\n");
        }
        out.push_str(&format!("## Post {}\n\n", i + 1));
        out.push_str(&format!("**URL:** {}\n\n", r.url));
        out.push_str(&format!("**Title:** {}\n\n", r.title));
        out.push_str(&format!("### Ideas\n\n{}\n\n", r.ideas));
    }
    out
}
