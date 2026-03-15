//! A batch processing example demonstrating how to use a single `Web2llm` client
//! to fetch multiple URLs sequentially and save the results to disk.
//!
//! This example shows off the scoring engine across many different types of sites
//! including Wikipedia, GitHub, news sites, and technical blogs.

use std::path::Path;
use web2llm::{Web2llm, Web2llmConfig};

const TEST_SITES: &[&str] = &[
    // --- Simple / clean content ---
    "https://example.com",
    "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    "https://en.wikipedia.org/wiki/Web_scraping",
    "https://matklad.github.io/2023/04/02/ub-might-be-the-wrong-term.html",
    "https://fasterthanli.me/articles/whats-in-the-box",
    // --- Docs sites ---
    "https://docs.rs/reqwest/latest/reqwest/",
    "https://doc.rust-lang.org/std/string/struct.String.html",
    "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/article",
    // --- News / heavy structure ---
    "https://www.bbc.com/news",
    "https://www.reuters.com",
    "https://news.ycombinator.com",
    // --- Code heavy ---
    "https://github.com/tokio-rs/tokio",
    "https://github.com/serde-rs/serde",
    // --- Tables ---
    "https://en.wikipedia.org/wiki/Comparison_of_programming_languages",
    // --- Images heavy ---
    "https://unsplash.com",
    // --- Markdown / technical blogs ---
    "https://blog.rust-lang.org/2024/02/08/Rust-1.76.0.html",
    "https://without.boats/blog/pinned-places/",
    // --- Minimal content ---
    "https://motherfuckingwebsite.com",
    "https://txti.es",
    // --- JS heavy (expected to struggle) ---
    "https://twitter.com",
    "https://reddit.com",
    "https://notion.so",
    // --- E-commerce noise ---
    "https://www.amazon.com/dp/B08N5WRWNW",
    // --- API / JSON response ---
    "https://api.github.com/repos/tokio-rs/tokio",
];

#[tokio::main]
async fn main() {
    let client = Web2llm::new(Web2llmConfig::default()).unwrap();
    let output_dir = Path::new("test_output");

    for url in TEST_SITES {
        println!("Fetching: {url}");
        match client.fetch(url).await {
            Ok(result) => {
                result.save_auto(output_dir).unwrap();
                println!("✓ {} → saved to {}", result.title, output_dir.display());
            }
            Err(e) => println!("✗ {url} → {e}"),
        }
    }
}
