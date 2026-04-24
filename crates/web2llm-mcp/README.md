# web2llm-mcp

`web2llm-mcp` is the MCP server crate in the workspace. Its role is to expose the `web2llm` extraction pipeline as MCP tools for external clients.

## Status

This crate is currently scaffolded and not yet implemented.

## Intended Scope

- expose page fetching as MCP tools
- expose batch and crawl operations through tool calls
- reuse the core [`web2llm`](../web2llm/README.md) library for extraction logic

## Workspace

See the [workspace README](../../README.md) for the overall project layout and the [`web2llm-cli`](../web2llm-cli/README.md) crate for the current user-facing interface.
