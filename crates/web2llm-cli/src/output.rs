//! Output rendering helpers for `web2llm-cli`.
//!
//! This module converts library return values into stable CLI-facing text
//! or JSON payloads without forcing extra serialization traits onto the
//! core library crate.

use std::fs;
use std::path::Path;

use serde::Serialize;
use web2llm::{PageResult, Web2llmError};

use crate::cli::{BatchOutputFormat, FetchOutputFormat, UrlOutputFormat};
use crate::error::Result;

/// A JSON-friendly representation of one successful page fetch.
#[derive(Debug, Serialize)]
pub struct PageDocument {
    /// The fetched URL.
    pub url: String,
    /// The extracted page title.
    pub title: String,
    /// The cleaned Markdown body.
    pub markdown: String,
    /// The total number of estimated tokens across all chunks.
    pub total_tokens: usize,
    /// The number of chunks produced by the extraction pipeline.
    pub chunk_count: usize,
    /// The UTC timestamp of the fetch in RFC3339 format.
    pub timestamp: String,
}

/// A JSON-friendly representation of one batch or crawl item.
#[derive(Debug, Serialize)]
pub struct BatchItemDocument {
    /// The original URL associated with the result.
    pub url: String,
    /// Whether the URL completed successfully.
    pub ok: bool,
    /// The successful page payload, when available.
    pub page: Option<PageDocument>,
    /// The stringified error message, when available.
    pub error: Option<String>,
}

/// A JSON-friendly report for batch and crawl commands.
#[derive(Debug, Serialize)]
pub struct BatchReport {
    /// The total number of processed URLs.
    pub total: usize,
    /// The number of successful URLs.
    pub succeeded: usize,
    /// The number of failed URLs.
    pub failed: usize,
    /// The per-URL results.
    pub results: Vec<BatchItemDocument>,
}

impl From<&PageResult> for PageDocument {
    fn from(value: &PageResult) -> Self {
        Self {
            url: value.url.clone(),
            title: value.title.clone(),
            markdown: value.markdown(),
            total_tokens: value.total_tokens(),
            chunk_count: value.chunks.len(),
            timestamp: value.timestamp.to_rfc3339(),
        }
    }
}

/// Converts a single page into the requested CLI output string.
pub fn render_fetch_result(page: &PageResult, format: FetchOutputFormat) -> Result<String> {
    Ok(match format {
        FetchOutputFormat::Markdown => page.markdown(),
        FetchOutputFormat::Json => serde_json::to_string_pretty(&PageDocument::from(page))?,
    })
}

/// Converts extracted URLs into the requested CLI output string.
pub fn render_urls(urls: &[String], format: UrlOutputFormat) -> Result<String> {
    Ok(match format {
        UrlOutputFormat::Text => urls.join("\n"),
        UrlOutputFormat::Json => serde_json::to_string_pretty(urls)?,
    })
}

/// Converts batch-style results into the requested CLI output string and failure count.
pub fn render_batch_report(
    results: Vec<(String, std::result::Result<PageResult, Web2llmError>)>,
    format: BatchOutputFormat,
) -> Result<(String, usize)> {
    let documents: Vec<_> = results
        .into_iter()
        .map(|(url, result)| match result {
            Ok(page) => BatchItemDocument {
                url,
                ok: true,
                page: Some(PageDocument::from(&page)),
                error: None,
            },
            Err(error) => BatchItemDocument {
                url,
                ok: false,
                page: None,
                error: Some(error.to_string()),
            },
        })
        .collect();

    let failed = documents.iter().filter(|item| !item.ok).count();
    let succeeded = documents.len() - failed;

    let output = match format {
        BatchOutputFormat::Json => serde_json::to_string_pretty(&BatchReport {
            total: documents.len(),
            succeeded,
            failed,
            results: documents,
        })?,
        BatchOutputFormat::Jsonl => documents
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()?
            .join("\n"),
    };

    Ok((output, failed))
}

/// Writes a rendered output string to a file path or stdout.
pub fn write_output(output: &str, path: Option<&Path>) -> Result<()> {
    if let Some(path) = path {
        fs::write(path, output)?;
    } else {
        println!("{output}");
    }

    Ok(())
}
