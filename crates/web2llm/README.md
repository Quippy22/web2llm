# web2llm

The `web2llm` crate is the core extraction engine in the workspace. It fetches web pages, isolates the most relevant content, and converts it into clean Markdown optimized for LLM ingestion and RAG pipelines.

## Features

- Content-aware extraction for article-like pages
- Clean Markdown output with headings, links, tables, and code blocks
- Fetch strategies via `FetchMode::Static`, `FetchMode::Dynamic`, and `FetchMode::Auto`
- Semantic chunking with token budgeting
- Recursive crawling with breadth-first link discovery
- Built-in URL validation, robots handling, rate limiting, and concurrency control

## Installation

```toml
[dependencies]
web2llm = "0.4.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
```

If you only want static scraping and do not need the browser-backed renderer:

```toml
[dependencies]
web2llm = { version = "0.4.0", default-features = false }
```

## Quick Start

```rust
use web2llm::fetch;

#[tokio::main]
async fn main() {
    let result = fetch("https://example.com".to_string()).await.unwrap();
    println!("{}", result.markdown());
}
```

## Configuring Fetch Mode

```rust
use web2llm::{FetchMode, Web2llm, Web2llmConfig};

#[tokio::main]
async fn main() {
    let config = Web2llmConfig {
        fetch_mode: FetchMode::Auto,
        ..Default::default()
    };

    let client = Web2llm::new(config).unwrap();
    let page = client.fetch("https://example.com").await.unwrap();
    println!("{}", page.title);
}
```

## Semantic Chunking

```rust
use web2llm::{Web2llm, Web2llmConfig};

#[tokio::main]
async fn main() {
    let config = Web2llmConfig {
        max_tokens: 500,
        ..Default::default()
    };

    let client = Web2llm::new(config).unwrap();
    let result = client.fetch("https://example.com").await.unwrap();

    for chunk in result.chunks {
        println!("chunk {} => {} tokens", chunk.index, chunk.tokens);
    }
}
```

## Crawling

```rust
use web2llm::{CrawlConfig, Web2llm, Web2llmConfig};

#[tokio::main]
async fn main() {
    let client = Web2llm::new(Web2llmConfig::default()).unwrap();

    let results = client
        .crawl(
            "https://example.com",
            CrawlConfig {
                max_depth: 1,
                preserve_domain: true,
            },
        )
        .await;

    for (url, result) in results {
        match result {
            Ok(page) => println!("{} -> {}", url, page.title),
            Err(error) => eprintln!("{} -> {}", url, error),
        }
    }
}
```

## Workspace

This crate is part of the larger workspace:

- [`web2llm-cli`](../web2llm-cli/README.md) provides the shell-facing interface
- [`web2llm-mcp`](../web2llm-mcp/README.md) will expose the same pipeline through an MCP server
- the workspace overview lives at the [repository root](../../README.md)
