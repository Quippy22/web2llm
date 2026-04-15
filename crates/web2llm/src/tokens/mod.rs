//! Logic for token counting, budget management, and structural chunking.
//!
//! This module provides the infrastructure for dividing a web page into
//! token-efficient "chunks" suitable for LLM ingestion and RAG pipelines.

use tl::{NodeHandle, Parser};

mod optimize;
pub use optimize::wash_markdown;

/// A structurally-aware slice of the page's content converted to Markdown.
///
/// Chunks are the atomic units of `web2llm`. Each chunk represents a contiguous
/// block of the original document (like a section, an article, or a group of
/// related paragraphs) that fits within a specific token budget.
#[derive(Debug, Clone)]
pub struct PageChunk {
    /// The position of this chunk in the document (0-indexed).
    pub index: usize,
    /// The cleaned Markdown content.
    pub content: String,
    /// The estimated number of tokens in the Markdown content.
    /// Calculated after "washing" the Markdown for maximum precision.
    pub tokens: usize,
    /// The extraction quality score for this chunk.
    /// Higher scores indicate more "meaty" content (prose, code, headers).
    pub score: f32,
}

/// Estimates the number of tokens and words in the direct text children of `node`.
///
/// Uses a high-performance, zero-allocation heuristic:
/// 1. Every 4 characters in a word counts as 1 token (BPE average).
/// 2. Any remaining characters in a word count as an additional token.
/// 3. Words are delimited by whitespace.
///
/// This estimation happens during the initial DOM traversal to avoid
/// redundant string processing.
pub(crate) fn get_direct_text_metrics(node_handle: NodeHandle, parser: &Parser) -> (f32, usize) {
    let mut total_words = 0.0;
    let mut total_tokens = 0;
    let mut char_in_word = 0;

    let node = node_handle.get(parser);
    if let Some(tag) = node.and_then(|n| n.as_tag()) {
        for child_handle in tag.children().top().iter() {
            if let Some(text) = child_handle
                .get(parser)
                .and_then(|n| n.as_raw())
                .and_then(|b| std::str::from_utf8(b.as_bytes()).ok())
            {
                let mut in_word = false;
                for c in text.chars() {
                    if c.is_whitespace() {
                        if in_word && char_in_word > 0 {
                            total_tokens += 1;
                            char_in_word = 0;
                        }
                        in_word = false;
                    } else {
                        if !in_word {
                            total_words += 1.0;
                            in_word = true;
                        }
                        char_in_word += 1;
                        if char_in_word == 4 {
                            total_tokens += 1;
                            char_in_word = 0;
                        }
                    }
                }
                if in_word && char_in_word > 0 {
                    total_tokens += 1;
                    char_in_word = 0;
                }
            }
        }
    }

    (total_words, total_tokens)
}

/// Checks if a token count is within the budget, allowing a 10% "soft limit".
///
/// This "soft limit" ensures that small structural units (like a short paragraph
/// at the end of a section) stay grouped with their context rather than being
/// forced into a separate, fragmented chunk.
pub(crate) fn is_within_budget(tokens: usize, max: usize) -> bool {
    tokens <= (max as f64 * 1.1) as usize
}
