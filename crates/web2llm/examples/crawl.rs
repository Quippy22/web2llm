//! A crawling example demonstrating breadth-first link discovery followed by a final batch fetch.

use web2llm::{CrawlConfig, FetchMode, Web2llm, Web2llmConfig};

#[tokio::main]
async fn main() {
    let seed = "https://example.com";
    let engine = Web2llm::new(Web2llmConfig {
        fetch_mode: FetchMode::Static,
        ..Default::default()
    })
    .unwrap();

    let results = engine
        .crawl(
            seed,
            CrawlConfig {
                max_depth: 1,
                preserve_domain: true,
            },
        )
        .await;

    for (url, result) in results {
        match result {
            Ok(page) => println!("{} -> {} chunks", url, page.chunks.len()),
            Err(error) => println!("{} -> {}", url, error),
        }
    }
}
