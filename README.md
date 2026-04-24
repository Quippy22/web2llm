# web2llm
[![Crates.io](https://img.shields.io/crates/v/web2llm)](https://crates.io/crates/web2llm)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![CI](https://github.com/Quippy22/web2llm/actions/workflows/push.yml/badge.svg)](https://github.com/Quippy22/web2llm/actions)
[![docs.rs](https://img.shields.io/docsrs/web2llm)](https://docs.rs/web2llm)

> Fetch web pages, turn them into clean Markdown, and expose the pipeline through a Rust library, CLI, and MCP server workspace.

`web2llm` is a Rust workspace centered around a reusable extraction engine for LLM ingestion and RAG pipelines. The workspace is split into a core library crate, a user-facing CLI, and an MCP server crate that will expose the same capabilities to tool-driven clients.

## Crates

- [`web2llm`](crates/web2llm/README.md) — the core library crate for fetching pages and converting them into clean Markdown
- [`web2llm-cli`](crates/web2llm-cli/README.md) — the command-line interface for `fetch`, `batch`, `crawl`, and `urls`
- [`web2llm-mcp`](crates/web2llm-mcp/README.md) — the MCP server crate, currently scaffolded for future implementation

## Workspace Quick Start

Build the whole workspace:

```bash
cargo check --workspace
```

Run the CLI:

```bash
cargo run -p web2llm-cli -- fetch https://example.com
```

Use the library directly:

```toml
[dependencies]
web2llm = "0.4.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
```

## Features

- **Core extraction engine** — isolate article-like content and convert it into clean Markdown
- **Adaptive fetch modes** — choose `static`, `dynamic`, or `auto` fetching strategies
- **Semantic chunking** — split content into token-budgeted chunks for downstream AI workflows
- **Recursive crawling** — discover links breadth-first and fetch them in one pass
- **CLI surface** — use `web2llm fetch`, `batch`, `crawl`, and `urls` from the shell
- **Workspace release flow** — crate-scoped versions, tags, packaging, and publishing

## Performance

**`web2llm` is built for high-throughput extraction.**
Metrics below represent extraction and processing throughput excluding network latency.

| Task | Average Time | Throughput |
| :--- | :--- | :--- |
| **Simple Page Extraction** | **~0.07 ms** | ~14,000+ pages/sec |
| **Wikipedia (Large) Extraction** | **~3.1 ms** | ~320 pages/sec |
| **Batch Fetch (100x Wikipedia)** | **~100 ms** | **~1,000 pages/sec** |

## Architecture

The core pipeline executes in 5 stages:

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
- [x] Recursive spider with concurrent link queue
- [x] CLI crate scaffold and first command surface
- [ ] MCP server implementation

## Repository Layout

```text
web2llm/
├── Cargo.toml
└── crates/
    ├── web2llm/
    │   ├── Cargo.toml
    │   ├── src/
    │   ├── tests/
    │   ├── examples/
    │   └── benchmarks/
    ├── web2llm-cli/
    │   ├── Cargo.toml
    │   └── src/
    └── web2llm-mcp/
        ├── Cargo.toml
        ├── README.md
        └── src/
```
