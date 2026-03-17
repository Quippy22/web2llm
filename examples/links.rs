//! An example demonstrating both Raw and Scored link extraction.
//!
//! 1. Raw extraction: Gets every single link from the HTML document.
//! 2. Scored extraction: Gets only links from the high-quality content blocks.

use web2llm::{FetchMode, Web2llm, Web2llmConfig};

#[tokio::main]
async fn main() {
    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    let config = Web2llmConfig {
        fetch_mode: FetchMode::Static,
        ..Default::default()
    };

    let engine = Web2llm::new(config).unwrap();

    println!("Fetching {}...", url);

    // --- Scenario 1: Raw URL Extraction (No Scoring) ---
    // engine.get_urls() fetches the page and returns every link found in the HTML.
    let raw_links = engine.get_urls(url).await.unwrap();
    println!(
        "\n[RAW] Found {} total links in the document.",
        raw_links.len()
    );
    println!("First 5 raw links:");
    for link in raw_links.iter().take(5) {
        println!("  - {link}");
    }

    // --- Scenario 2: Scored URL Extraction ---
    // engine.fetch() runs the full scoring pipeline. We then extract links
    // only from the blocks that were considered "main content".
    let result = engine.fetch(url).await.unwrap();
    let processed_links = result.get_urls();

    println!(
        "\n[SCORED] Found {} links in the high-quality content blocks.",
        processed_links.len()
    );
    println!("First 5 scored links:");
    for link in processed_links.iter().take(5) {
        println!("  - {link}");
    }

    println!(
        "\nDifference: Scored extraction removed {} 'noisy' links (navbars, footers, etc.)",
        raw_links.len() - processed_links.len()
    );
}
