# web2llm

> Fetch any web page. Get clean, token-efficient Markdown. Ready for LLMs.

`web2llm` is a high-performance, modular Rust crate that fetches web pages, strips away computational noise (ads, navbars, footers, scripts), and converts the core content into clean Markdown optimized for Large Language Model (LLM) ingestion and Retrieval-Augmented Generation (RAG) pipelines.

---

## Why web2llm?

Feeding raw HTML to an LLM is wasteful and noisy. A typical web page is 80% structural boilerplate — navigation, cookie banners, footers, tracking scripts — and only 20% actual content. `web2llm` inverts that ratio, giving your LLM only what matters.

---

## Features

- **Content-aware extraction** — uses readability heuristics to isolate the main article body, not just strip tags
- **Clean Markdown output** — GitHub Flavored Markdown that preserves headings, tables, and code blocks while discarding layout noise
- **Token-efficient** — output is designed to minimize token cost in downstream LLM calls
- **robots.txt compliance** — respects crawl rules out of the box
- **Rate limiting** — per-host request throttling built in
- **SPA support** *(optional feature)* — headless browser fallback for JavaScript-rendered pages
- **Recursive spidering** — discovers and follows internal links concurrently
- **Modular pipeline** — each stage is independently swappable

---

## Architecture

The pipeline executes in 5 stages:

```
URL
 │
 ▼
[1] Pre-flight       — URL validation, robots.txt check, rate limiting
 │
 ▼
[2] Fetch            — Static fetch (reqwest) with optional SPA fallback (chromiumoxide)
 │
 ▼
[3] Extract          — Readability heuristics isolate main content, link discovery
 │
 ▼
[4] Transform        — HTML → clean Markdown, semantic chunking, token counting
 │
 ▼
[5] Output           — PageResult struct, optional disk persistence
```

---

## Quick Start

```toml
[dependencies]
web2llm = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use web2llm::Web2Llm;

#[tokio::main]
async fn main() {
    let result = Web2Llm::fetch("https://example.com").await.unwrap();

    println!("{}", result.markdown);
    println!("~{} tokens", result.token_count);
}
```

---

## Feature Flags

| Flag        | Description                                                                 | Default |
|-------------|-----------------------------------------------------------------------------|---------|
| `static`    | Static HTTP fetching via `reqwest` only                                     | ❌ off  |
| `adaptive`  | Static fetch with automatic headless fallback for JS-heavy pages            | ✅ on   |
| `rendered`  | Forces full JS rendering via `chromiumoxide` for every page                 | ❌ off  |

Enable full JS rendering:

```toml
web2llm = { version = "0.1", features = ["rendered"] }
```

---

## License

MIT
