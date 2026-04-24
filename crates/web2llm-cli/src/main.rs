//! Executable entry point for `web2llm-cli`.
//!
//! The CLI wraps the `web2llm` library with a straightforward command surface
//! aimed at shell usage and scripting.

mod cli;
mod config;
mod error;
mod output;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use cli::{BatchCommand, Cli, Command, CrawlCommand, FetchCommand, UrlsCommand};
use config::{load_file_config, resolve_crawl_config, resolve_web2llm_config, validate_configs};
use error::{CliError, Result};
use output::{render_batch_report, render_fetch_result, render_urls, write_output};
use web2llm::Web2llm;

/// Parses arguments, executes the command, and returns an OS exit code.
#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Executes the parsed CLI command.
async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Fetch(command) => run_fetch(command).await,
        Command::Batch(command) => run_batch(command).await,
        Command::Crawl(command) => run_crawl(command).await,
        Command::Urls(command) => run_urls(command).await,
    }
}

/// Handles the `fetch` subcommand.
async fn run_fetch(command: FetchCommand) -> Result<()> {
    let file_config = load_file_config(command.common.config.as_deref())?;
    let web_config = resolve_web2llm_config(&file_config, &command.common);
    validate_configs(&web_config)?;

    let engine = Web2llm::new(web_config)?;
    let page = engine.fetch(&command.url).await?;
    let output = render_fetch_result(&page, command.format)?;
    write_output(&output, command.out.as_deref())
}

/// Handles the `batch` subcommand.
async fn run_batch(command: BatchCommand) -> Result<()> {
    let file_config = load_file_config(command.common.config.as_deref())?;
    let web_config = resolve_web2llm_config(&file_config, &command.common);
    validate_configs(&web_config)?;

    let engine = Web2llm::new(web_config)?;
    let results = engine.batch_fetch(command.urls).await;

    if let Some(dir) = command.out_dir.as_deref() {
        save_successful_pages(&results, dir)?;
    }

    let (output, failed) = render_batch_report(results, command.format)?;
    write_output(&output, command.out.as_deref())?;

    if failed > 0 {
        return Err(CliError::PartialFailure(failed));
    }

    Ok(())
}

/// Handles the `crawl` subcommand.
async fn run_crawl(command: CrawlCommand) -> Result<()> {
    let file_config = load_file_config(command.common.config.as_deref())?;
    let web_config = resolve_web2llm_config(&file_config, &command.common);
    let crawl_config = resolve_crawl_config(&file_config, &command);
    validate_configs(&web_config)?;

    let engine = Web2llm::new(web_config)?;
    let results = engine.crawl(&command.url, crawl_config).await;

    if let Some(dir) = command.out_dir.as_deref() {
        save_successful_pages(&results, dir)?;
    }

    let (output, failed) = render_batch_report(results, command.format)?;
    write_output(&output, command.out.as_deref())?;

    if failed > 0 {
        return Err(CliError::PartialFailure(failed));
    }

    Ok(())
}

/// Handles the `urls` subcommand.
async fn run_urls(command: UrlsCommand) -> Result<()> {
    let file_config = load_file_config(command.common.config.as_deref())?;
    let web_config = resolve_web2llm_config(&file_config, &command.common);
    validate_configs(&web_config)?;

    let engine = Web2llm::new(web_config)?;
    let urls = engine.get_urls(&command.url).await?;
    let output = render_urls(&urls, command.format)?;
    write_output(&output, command.out.as_deref())
}

/// Saves successful pages from a batch-style result set into an output directory.
fn save_successful_pages(
    results: &[(
        String,
        std::result::Result<web2llm::PageResult, web2llm::Web2llmError>,
    )],
    dir: &Path,
) -> Result<()> {
    for (_, result) in results {
        if let Ok(page) = result {
            page.save_auto(dir)?;
        }
    }

    Ok(())
}
