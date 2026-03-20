//! A detailed example demonstrating semantic chunking and token budget management.
//!
//! This example shows how to configure the `max_tokens` budget and iterate
//! through individual `PageChunk` structs to see how the document was divided.

use web2llm::{FetchMode, Web2llm, Web2llmConfig};

#[tokio::main]
async fn main() {
    // 1. Configure the engine with a specific token budget.
    // The engine will try to keep chunks around this size, using a 1.1x soft limit
    // to preserve structural integrity (like not splitting a paragraph).
    let config = Web2llmConfig {
        max_tokens: 300, // Target small chunks for precision
        fetch_mode: FetchMode::Static,
        ..Default::default()
    };

    let client = Web2llm::new(config).unwrap();
    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";

    println!("Fetching and chunking {}...", url);

    match client.fetch(url).await {
        Ok(result) => {
            println!("\nSuccess! Divided into {} chunks.", result.chunks.len());
            println!("Total estimated tokens: {}\n", result.total_tokens());

            // 2. Iterate through chunks to see the structural breakdown
            for chunk in result.chunks.iter().take(5) {
                println!("--------------------------------------------------");
                println!(
                    "CHUNK #{} (Tokens: {}, Score: {:.2})",
                    chunk.index, chunk.tokens, chunk.score
                );
                println!("--------------------------------------------------");

                // Show a snippet of the content
                let snippet: String = chunk.content.chars().take(150).collect();
                println!("{}...", snippet);
                println!();
            }

            if result.chunks.len() > 5 {
                println!("... and {} more chunks.", result.chunks.len() - 5);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
