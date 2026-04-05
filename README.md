# web2llm
[![Crates.io](https://img.shields.io/crates/v/web2llm)](https://crates.io/crates/web2llm)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![CI](https://github.com/Quippy22/web2llm/actions/workflows/push.yml/badge.svg)](https://github.com/Quippy22/web2llm/actions)
[![docs.rs](https://img.shields.io/docsrs/web2llm)](https://docs.rs/web2llm)

> ### Fetch any web page. Get clean Markdown. Ready for LLMs.

#### `web2llm` is a high-performance, modular Rust crate that fetches web pages, strips away computational noise (ads, navbars, footers, scripts), and converts the core content into clean Markdown optimized for Large Language Model (LLM) ingestion and Retrieval-Augmented Generation (RAG) pipelines.


## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
web2llm = "0.3.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
```

Fetch and print Markdown in one call:

```rust
use web2llm::fetch;

#[tokio::main]
async fn main() {
    // 1. Simple fetch (Uses Auto-mode + default settings)
    let result = fetch("https://example.com".to_string()).await.unwrap();
    
    // 2. Print the cleaned Markdown
    println!("{}", result.markdown());
}
```

## Features

- **Content-aware extraction** — isolates the main article body with extreme precision.
- **Clean Markdown output** — preserves headings, tables, code blocks, and inline links.
- **Adaptive fetch** — automatic fallback to headless browser for JS-heavy SPAs.
- **High Performance** — zero-copy traversal and bump-allocation (~3.9ms for Wikipedia).
- **Semantic Chunking** — divide content into logical, token-budgeted islands for AI apps.


## Configuration & Fetch Strategies

You can control how `web2llm` handles pages via the `FetchMode` configuration:

- **`FetchMode::Static`**: Fast, standard HTTP request. No JavaScript execution.
- **`FetchMode::Dynamic`**: Uses a headless browser to render the page. Required for SPAs.
- **`FetchMode::Auto`**: (Default) Smart mode. Tries a fast static fetch first, detects if the page is an SPA shell, and automatically restarts using the browser only if needed.

```rust
use web2llm::{Web2llm, Web2llmConfig, FetchMode};

let config = Web2llmConfig {
    fetch_mode: FetchMode::Auto,
    ..Default::default()
};
```

### Lightweight Build (Optional)
`web2llm` includes Chromium support by default for a "plug-and-play" experience. Power users who only need static scraping can disable defaults to remove the Chromium dependency (~50 sub-dependencies):

```toml
[dependencies]
web2llm = { version = "0.3.1", default-features = false }
```
## Performance

**`web2llm` is built for extreme speed and high-throughput ingestion.**
##### Note: Metrics represent pure extraction and processing throughput, excluding network latency.

| Task | Average Time | Throughput |
| :--- | :--- | :--- |
| **Simple Page Extraction** | **~0.07 ms** | ~14,000+ pages/sec |
| **Wikipedia (Large) Extraction** | **~3.1 ms** | ~320 pages/sec |
| **Batch Fetch (100x Wikipedia)** | **~100 ms** | **~1,000 pages/sec** |

Speed may vary on different systems


## Advanced: Semantic Chunking

For "true AI" applications and RAG pipelines, `web2llm` can divide documents into logical, structurally-aware chunks that fit your token budget without splitting paragraphs mid-sentence.

```rust
let config = Web2llmConfig {
    max_tokens: 500, // Target 500 tokens per chunk
    ..Default::default()
};

let client = Web2llm::new(config).unwrap();
let result = client.fetch(url).await.unwrap();

// Access granular chunks for precise vector embedding
for chunk in result.chunks {
    println!("Chunk #{} ({} tokens): {:.2} quality score", chunk.index, chunk.tokens, chunk.score);
}
```


## Architecture

The pipeline executes in 5 stages:

```
URL
 │
 ▼
[1] Pre-flight       — URL validation, robots.txt check, rate limiting
 │
 ▼
[2] Fetch            — Static fetch (reqwest) or Dynamic fallback (chromiumoxide)
 │
 ▼
[3] Score            — Bottom-up recursive scoring builds a "Scored Tree" (Bump-allocated)
 │
 ▼
[4] Chunk & Wash     — Top-down "Flatten or Recurse" chunking + Markdown optimization
 │
 ▼
[5] Output           — PageResult struct containing Vec<PageChunk>
```


## Roadmap

- [x] Vertical slice — fetch, extract, score, convert to Markdown
- [x] Unified error handling
- [x] `Web2llmConfig` — idiomatic initialization
- [x] Performance optimizations — bump-allocation and zero-copy traversal
- [x] Batch fetch — parallel fetching across CPU cores
- [x] Adaptive fetch — SPA detection and browser fallback
- [x] Rate limiting — per-host throttling
- [x] Token counting & Semantic chunking
- [ ] Recursive spider with concurrent link queue
- [ ] MCP server — `web2llm-mcp`
- [ ] CLI — `web2llm-cli`
