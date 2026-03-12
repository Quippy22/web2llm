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
- **Modular pipeline** — each stage is independently swappable
- **robots.txt compliance** *(coming soon)* — respects crawl rules out of the box
- **Rate limiting** *(coming soon)* — per-host request throttling built in
- **Adaptive fetch** *(coming soon)* — static fetch with automatic headless browser fallback for JS-heavy pages
- **Recursive spidering** *(coming soon)* — discovers and follows internal links concurrently


## Architecture

The pipeline executes in 5 stages:

```
URL
 │
 ▼
[1] Pre-flight       — URL validation, robots.txt check, rate limiting
 │
 ▼
[2] Fetch            — Static fetch (reqwest) with adaptive SPA fallback (chromiumoxide)
 │
 ▼
[3] Extract          — Content scoring isolates main body, link discovery
 │
 ▼
[4] Transform        — HTML → clean Markdown, semantic chunking, token counting
 │
 ▼
[5] Output           — PageResult struct, optional disk persistence
```

## Quick Start

```toml
[dependencies]
web2llm = "0.0.2"
tokio = { version = "1", features = ["full"] }
```

```rust
use web2llm::fetch;

#[tokio::main]
async fn main() {
    let result = fetch("https://example.com").await.unwrap();
    println!("{}", result.markdown);
}
```

## Feature Flags

| Flag        | Description                                                      | Status      | Default |
|-------------|------------------------------------------------------------------|-------------|---------|
| `static`    | Static HTTP fetching via `reqwest` only                          | coming soon | ❌ off  |
| `adaptive`  | Static fetch with automatic headless fallback for JS-heavy pages | coming soon | ✅ on   |
| `rendered`  | Forces full JS rendering via `chromiumoxide` for every page      | coming soon | ❌ off  |


## Roadmap

- [x] Vertical slice — fetch, extract, score, convert to Markdown
- [x] Unified error handling
- [x] `PageResult` output struct with url, title, markdown, and timestamp
- [x] `Web2llmConfig` — user-facing configuration struct
- [x] Pre-flight — URL validation and `robots.txt` compliance
- [ ] Adaptive fetch — SPA detection and headless browser fallback
- [ ] Batch fetch — fetch multiple URLs concurrently
- [ ] Rate limiting — per-host request throttling
- [ ] Token counting
- [ ] Semantic chunking
- [ ] Recursive spider with concurrent link queue
- [ ] MCP server — `web2llm-mcp`
- [ ] CLI — `web2llm-cli`
