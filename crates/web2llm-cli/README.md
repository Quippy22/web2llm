# web2llm-cli

`web2llm-cli` is the command-line interface for the `web2llm` extraction engine. It lets you fetch a page, fetch multiple pages, crawl from a seed URL, or extract raw discovered links directly from the shell.

When installed, the binary name is `web2llm`.

## Installation

```bash
cargo install web2llm-cli
```

## Commands

```bash
web2llm fetch <url>
web2llm batch <url> <url> <url>
web2llm crawl <url>
web2llm urls <url>
```

## Examples

Fetch one page and print Markdown to stdout:

```bash
web2llm fetch https://example.com
```

Fetch one page as JSON:

```bash
web2llm fetch https://example.com --format json
```

Fetch multiple URLs and write the report to a file:

```bash
web2llm batch https://example.com https://www.rust-lang.org --out report.json
```

Crawl one level deep and save successful pages as Markdown files:

```bash
web2llm crawl https://example.com --depth 1 --out-dir output/
```

Extract discovered links only:

```bash
web2llm urls https://example.com
```

## Config File

The CLI supports a TOML config file layered between library defaults and explicit command-line flags:

```toml
[web2llm]
fetch_mode = "auto"
timeout_secs = 30
max_tokens = 1000
rate_limit = 5
max_concurrency = 10
ordered = false

[crawl]
max_depth = 1
preserve_domain = true
```

Use it with:

```bash
web2llm fetch https://example.com --config web2llm.toml
```

## Output Modes

- `fetch` supports `--format markdown` and `--format json`
- `batch` and `crawl` support `--format json` and `--format jsonl`
- `urls` supports `--format text` and `--format json`

## Workspace

This crate depends on the core [`web2llm`](../web2llm/README.md) library crate. The workspace overview lives at the [repository root](../../README.md).
