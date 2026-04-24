//! Command-line interface definitions for `web2llm-cli`.
//!
//! This module contains the `clap` parser tree for the CLI surface:
//! global configuration overrides, command-specific flags, and output
//! format selection.

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};

/// The top-level command-line parser for `web2llm-cli`.
///
/// Each subcommand maps directly onto one of the user-facing library flows:
/// fetch one page, fetch multiple pages, crawl recursively, or return raw
/// discovered URLs from a page.
#[derive(Debug, Parser)]
#[command(name = "web2llm")]
#[command(version)]
#[command(about = "Fetch web pages and convert them into clean Markdown.")]
#[command(long_about = None)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Fetch one page and emit cleaned Markdown or JSON.
    Fetch(FetchCommand),
    /// Fetch multiple pages in one run.
    Batch(BatchCommand),
    /// Crawl outward from one seed URL and fetch discovered pages.
    Crawl(CrawlCommand),
    /// Return absolute URLs discovered on a single page.
    Urls(UrlsCommand),
}

/// Shared runtime overrides for `web2llm` engine configuration.
///
/// These flags layer on top of defaults and optional TOML config files.
#[derive(Debug, Args, Clone, Default)]
pub struct CommonOptions {
    /// Path to a TOML config file containing `[web2llm]` and optional `[crawl]` sections.
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Override the user-agent string sent with each request.
    #[arg(long)]
    pub user_agent: Option<String>,

    /// Override the request timeout in whole seconds.
    #[arg(long)]
    pub timeout_secs: Option<u64>,

    /// Allow requests to localhost and private network hosts.
    #[arg(long, action = ArgAction::SetTrue)]
    pub allow_private_hosts: bool,

    /// Disable `robots.txt` checks.
    #[arg(long, action = ArgAction::SetTrue)]
    pub no_robots: bool,

    /// Override the extraction sensitivity in the inclusive `0.0..=1.0` range.
    #[arg(long)]
    pub sensitivity: Option<f32>,

    /// Override the target token budget for each chunk.
    #[arg(long)]
    pub max_tokens: Option<usize>,

    /// Override the per-second rate limit.
    #[arg(long)]
    pub rate_limit: Option<u32>,

    /// Override the maximum concurrent request count.
    #[arg(long)]
    pub max_concurrency: Option<usize>,

    /// Override the fetch strategy used for page retrieval.
    #[arg(long, value_enum)]
    pub fetch_mode: Option<FetchModeArg>,

    /// Preserve the input order for batch and crawl result lists.
    #[arg(long, action = ArgAction::SetTrue)]
    pub ordered: bool,
}

/// Supported fetch strategies exposed by the CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FetchModeArg {
    /// Automatically fall back to a browser if the page looks JS-driven.
    Auto,
    /// Use a plain HTTP fetch without JavaScript execution.
    Static,
    /// Render the page through the browser-backed fetcher.
    Dynamic,
}

/// Output formats for single-page fetches.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FetchOutputFormat {
    /// Emit only the cleaned Markdown body.
    Markdown,
    /// Emit structured JSON including metadata and Markdown.
    Json,
}

/// Output formats for batch and crawl operations.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BatchOutputFormat {
    /// Emit one JSON object containing the full report.
    Json,
    /// Emit one JSON object per result line.
    Jsonl,
}

/// Output formats for raw URL extraction.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum UrlOutputFormat {
    /// Emit one URL per line.
    Text,
    /// Emit a JSON array of URLs.
    Json,
}

/// Fetch a single URL through the full pipeline.
#[derive(Debug, Args)]
pub struct FetchCommand {
    /// The URL to fetch.
    pub url: String,

    /// Shared engine configuration overrides.
    #[command(flatten)]
    pub common: CommonOptions,

    /// Output format for the fetch result.
    #[arg(long, value_enum, default_value_t = FetchOutputFormat::Markdown)]
    pub format: FetchOutputFormat,

    /// Optional file path to write the output to instead of stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// Fetch multiple URLs in one invocation.
#[derive(Debug, Args)]
pub struct BatchCommand {
    /// The URLs to fetch.
    #[arg(required = true, num_args = 1..)]
    pub urls: Vec<String>,

    /// Shared engine configuration overrides.
    #[command(flatten)]
    pub common: CommonOptions,

    /// Output format for the aggregated result report.
    #[arg(long, value_enum, default_value_t = BatchOutputFormat::Json)]
    pub format: BatchOutputFormat,

    /// Optional file path to write the aggregated report to instead of stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Optional directory to write each successful page as a Markdown file.
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

/// Crawl outward from one seed URL before fetching discovered pages.
#[derive(Debug, Args)]
pub struct CrawlCommand {
    /// The seed URL to crawl from.
    pub url: String,

    /// Shared engine configuration overrides.
    #[command(flatten)]
    pub common: CommonOptions,

    /// Maximum number of link-expansion steps from the seed URL.
    #[arg(long)]
    pub depth: Option<usize>,

    /// Allow following links outside the seed origin.
    #[arg(long, action = ArgAction::SetTrue)]
    pub cross_origin: bool,

    /// Output format for the aggregated crawl report.
    #[arg(long, value_enum, default_value_t = BatchOutputFormat::Json)]
    pub format: BatchOutputFormat,

    /// Optional file path to write the aggregated report to instead of stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Optional directory to write each successful page as a Markdown file.
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
}

/// Extract raw absolute URLs from a page.
#[derive(Debug, Args)]
pub struct UrlsCommand {
    /// The URL whose links should be extracted.
    pub url: String,

    /// Shared engine configuration overrides.
    #[command(flatten)]
    pub common: CommonOptions,

    /// Output format for the extracted URLs.
    #[arg(long, value_enum, default_value_t = UrlOutputFormat::Text)]
    pub format: UrlOutputFormat,

    /// Optional file path to write the URL list to instead of stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
}
