# web2llm
[![Crates.io](https://img.shields.io/crates/v/web2llm)](https://crates.io/crates/web2llm)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![CI](https://github.com/Quippy22/web2llm/actions/workflows/push.yml/badge.svg)](https://github.com/Quippy22/web2llm/actions)
[![docs.rs](https://img.shields.io/docsrs/web2llm)](https://docs.rs/web2llm)

> Fetch any web page. Get clean, token-efficient Markdown. Ready for LLMs.

`web2llm` is a high-performance, modular Rust crate that fetches web pages, strips away computational noise (ads, navbars, footers, scripts), and converts the core content into clean Markdown optimized for Large Language Model (LLM) ingestion and Retrieval-Augmented Generation (RAG) pipelines.


## Why web2llm?

Feeding raw HTML to an LLM is wasteful and noisy. A typical web page is 80% structural boilerplate — navigation, cookie banners, footers, tracking scripts — and only 20% actual content. `web2llm` inverts that ratio, giving your LLM only what matters.


## Features

- **Content-aware extraction** — scores every element by text density, tag semantics, and link ratio to isolate the main article body
- **Clean Markdown output** — preserves headings, tables, code blocks, and inline links while discarding layout noise
- **Token-efficient** — output is designed to minimize token cost in downstream LLM calls
- **Shared Headless Browser** — single persistent Chromium instance for dynamic pages (requires `rendered` feature)
- **Adaptive fetch** — automatic fallback to headless browser for JS-heavy SPAs
- **Robots.txt compliance** — respects crawl rules out of the box
- **Performance optimized** — zero-copy tree traversal, LTO, and minimal allocations


## Performance

`web2llm` is built for extreme speed and high-throughput RAG pipelines.

| Task | Average Time | Throughput |
| :--- | :--- | :--- |
| **Simple Page Extraction** | **< 1.0 ms** | ~1,000+ pages/sec |
| **Wikipedia (Large) Extraction** | ~4.3 ms | ~230 pages/sec |
| **Batch Fetch (100x Wikipedia)** | ~103.7 ms | **~960+ pages/sec** |

<sup>*Benchmarks performed on an AMD Ryzen 7 5800X. Real-world performance may vary based on network latency.*</sup>

*Note: Batch fetch utilizes true parallelism via `tokio::spawn`, saturating CPU cores for parsing and scoring while managing I/O efficiently.*

## Configuration & Features

### `rendered` Feature Flag (Headless Browser)
By default, `web2llm` is lightweight and only performs static HTTP fetches. To support Single Page Applications (SPAs) or sites that require JavaScript rendering, enable the `rendered` feature:

```toml
[dependencies]
web2llm = { version = "0.2.1", features = ["rendered"] }
```

### `FetchMode` Strategies
You can control how `web2llm` handles pages via the `fetch_mode` configuration:

- **`FetchMode::Static`**: (Default) Fast, standard HTTP request. No JavaScript execution.
- **`FetchMode::Dynamic`**: Uses a headless browser to render the page. Required for SPAs.
- **`FetchMode::Auto`**: Smart mode. Tries a fast static fetch first, detects if the page is an SPA shell, and automatically restarts using the headless browser only if needed.

```rust
let config = Web2llmConfig {
    fetch_mode: FetchMode::Auto,
    ..Default::default()
};
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
[3] Extract          — Content scoring isolates main body, link discovery
 │
 ▼
[4] Transform        — HTML → clean Markdown
 │
 ▼
[5] Output           — PageResult struct, optional disk persistence
```

## Quick Start

```toml
[dependencies]
web2llm = "0.2.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Simple Fetch (Static)

```rust
use web2llm::fetch;

#[tokio::main]
async fn main() {
    let result = fetch("https://example.com".to_string()).await.unwrap();
    println!("{}", result.markdown);
}
```

### Dynamic Fetch (SPA Support)

Enable the `rendered` feature to support JavaScript-heavy sites:

```toml
[dependencies]
web2llm = { version = "0.2.1", features = ["rendered"] }
```

```rust
use web2llm::{Web2llm, Web2llmConfig, FetchMode};

#[tokio::main]
async fn main() {
    let config = Web2llmConfig {
        fetch_mode: FetchMode::Auto, // Automatically use browser if SPA is detected
        ..Default::default()
    };
    
    let client = Web2llm::new(config).unwrap();
    let result = client.fetch("https://reddit.com").await.unwrap();
    println!("{}", result.markdown);

    // Extract links found in the scored content
    let links = result.get_urls();
}
```

### Link Extraction
`web2llm` provides two ways to extract URLs from a page:

1.  **`Web2llm::get_urls(url)`**: (Raw) Fetches the page and returns every single absolute link found in the original HTML document (includes nav, footers, etc.).
2.  **`PageResult::get_urls()`**: (Scored) Returns only the links found within the high-quality content blocks that survived the scoring process.


## Roadmap

- [x] Vertical slice — fetch, extract, score, convert to Markdown
- [x] Unified error handling
- [x] `PageResult` output struct with url, title, markdown, and timestamp
- [x] `Web2llmConfig` — user-facing configuration struct (idiomatic initialization)
- [x] Pre-flight — URL validation and `robots.txt` compliance
- [x] Performance optimizations — zero-copy traversal and shared browser
- [x] Batch fetch — fetch multiple URLs concurrently
- [x] Adaptive fetch — SPA detection and headless browser fallback
- [x] Rate limiting — per-host request throttling
- [ ] Token counting
- [ ] Semantic chunking
- [ ] Recursive spider with concurrent link queue
- [ ] MCP server — `web2llm-mcp`
- [ ] CLI — `web2llm-cli`
