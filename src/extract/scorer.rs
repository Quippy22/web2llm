//! The core extraction and scoring engine of `web2llm`.
//!
//! This module implements a recursive N-ary tree traversal that assigns quality
//! scores to every node in the DOM. It uses these scores to prune noise
//! (navigation, footers, sidebars) and divide the remaining content into
//! structurally-aware Markdown chunks.
//!
//! # Architecture
//! 1. **Scoring (Bottom-Up)**: `compute_metrics` traverses the DOM and builds a
//!    `NodeMetrics` tree. Scores and token estimates bubble up from leaves to roots.
//! 2. **Pruning (Inline)**: Nodes identified as `PENALTY_TAGS` with low subtree scores
//!    are discarded during the initial pass to minimize memory overhead.
//! 3. **Chunking (Top-Down)**: `rebuild_to_chunks` traverses the clean metrics tree.
//!    If a branch fits the token budget, it's "flattened" into HTML. If not, the
//!    engine recurses into children to find smaller islands of content.
//! 4. **Greedy Grouping**: To minimize expensive Markdown conversions, adjacent
//!    sibling nodes are grouped into a single HTML buffer before being converted.

use crate::config::Web2llmConfig;
use crate::error::{Result, Web2llmError};
use crate::tokens::wash_markdown;
use crate::tokens::{PageChunk, get_direct_text_metrics, is_within_budget};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use htmd::convert;
use scraper::ElementRef;

const TAG_BONUS_HIGH: f32 = 2.0;
const TAG_BONUS_MED: f32 = 1.0;
const TAG_BONUS_NEUTRAL: f32 = 1.0;
const TAG_BONUS_LOW: f32 = 0.7;
const TAG_BONUS_POOR: f32 = 0.5;
const TAG_BONUS_PENALTY: f32 = 0.05;
const PASSTHROUGH_SCORE: f32 = 10.0;

const HIGH_BONUS_TAGS: &[&str] = &["article", "main", "section"];
const MED_BONUS_TAGS: &[&str] = &[
    "div", "p", "span", "table", "thead", "tbody", "tfoot", "tr", "th", "td",
];
const LOW_BONUS_TAGS: &[&str] = &["figure", "figcaption", "details"];
const POOR_BONUS_TAGS: &[&str] = &["form", "button", "label", "ul", "ol", "li"];
const PENALTY_TAGS: &[&str] = &["nav", "footer", "header", "aside", "menu"];
const PASSTHROUGH_TAGS: &[&str] = &[
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "pre",
    "code",
    "blockquote",
];
const SKIP_TAGS: &[&str] = &["script", "style", "noscript", "template"];

const MIN_SCORE_THRESHOLD: f32 = 5.0;

/// A mirror of the DOM tree that stores pre-calculated extraction metrics.
///
/// By storing children's metrics in the parent, the chunking engine can make
/// $O(1)$ decisions about where to split the document without re-scanning the DOM.
///
/// Allocated using `bumpalo` for maximum performance and zero heap fragmentation.
#[derive(Clone, Copy)]
pub(crate) struct NodeMetrics<'a> {
    /// The cumulative quality score of this node and its entire subtree.
    pub(crate) score: f32,
    /// The estimated number of tokens in this subtree.
    pub(crate) tokens: usize,
    /// A reference to the original DOM element.
    pub(crate) element: ElementRef<'a>,
    /// Pre-scored child nodes that survived the pruning phase.
    pub(crate) children: &'a [NodeMetrics<'a>],
}

/// Entry point: Processes the body and returns structurally-aware Markdown chunks.
///
/// This function coordinates the full extraction pipeline: scoring, pruning,
/// sibling grouping, and Markdown conversion.
pub(crate) fn process(
    body: ElementRef,
    config: &Web2llmConfig,
) -> Result<std::vec::Vec<PageChunk>> {
    let bump = Bump::with_capacity(64 * 1024);

    let mut roots: std::vec::Vec<NodeMetrics> = body
        .children()
        .filter_map(ElementRef::wrap)
        .map(|el| compute_metrics(el, &bump, config.sensitivity * 0.01)) // Prune during construction
        .filter(|e| e.score > 0.0)
        .collect();

    let winner = roots.iter().map(|e| e.score).fold(0.0_f32, f32::max);
    if winner < MIN_SCORE_THRESHOLD {
        return Ok(std::vec::Vec::new());
    }

    let threshold = winner * config.sensitivity;
    roots.retain(|e| e.score >= threshold);

    let mut chunks = std::vec::Vec::new();
    for root in roots {
        let mut html_buf = String::with_capacity(8192);
        let mut token_acc = 0;
        rebuild_to_chunks(&root, config, &mut chunks, &mut html_buf, &mut token_acc)?;
        emit_buffer(&mut chunks, &mut html_buf, &mut token_acc, root.score)?;
    }

    Ok(chunks)
}

fn compute_metrics<'a>(
    node: ElementRef<'a>,
    bump: &'a Bump,
    prune_threshold: f32,
) -> NodeMetrics<'a> {
    let tag = node.value().name();
    if SKIP_TAGS.contains(&tag) {
        return NodeMetrics {
            score: 0.0,
            tokens: 0,
            element: node,
            children: &[],
        };
    }

    let (own_words, own_tokens) = get_direct_text_metrics(node);
    let mut children_score = 0.0;
    let mut children_tokens = 0;
    let mut children = BumpVec::new_in(bump);

    for child in node.children().filter_map(ElementRef::wrap) {
        let metrics = compute_metrics(child, bump, prune_threshold);

        // Inline pruning: don't even add penalty nodes if they are too small
        if PENALTY_TAGS.contains(&child.value().name()) && metrics.score < prune_threshold {
            continue;
        }

        if metrics.score > 0.0 || PASSTHROUGH_TAGS.contains(&child.value().name()) {
            children_score += metrics.score;
            children_tokens += metrics.tokens;
            children.push(metrics);
        }
    }

    let is_pass = PASSTHROUGH_TAGS.contains(&tag);
    let multiplier = if is_pass { 1.0 } else { tag_multiplier(tag) };
    let score = if is_pass {
        PASSTHROUGH_SCORE + children_score
    } else {
        (own_words + children_score) * multiplier
    };

    NodeMetrics {
        score,
        tokens: own_tokens + children_tokens,
        element: node,
        children: children.into_bump_slice(),
    }
}

fn rebuild_to_chunks(
    node: &NodeMetrics,
    config: &Web2llmConfig,
    chunks: &mut std::vec::Vec<PageChunk>,
    html_buf: &mut String,
    token_acc: &mut usize,
) -> Result<()> {
    if is_within_budget(node.tokens, config.max_tokens) {
        // If it fits, add to current buffer instead of emitting immediately
        rebuild_html(node, html_buf, false);
        *token_acc += node.tokens;

        // If buffer is getting large, emit it
        if *token_acc >= config.max_tokens {
            emit_buffer(chunks, html_buf, token_acc, node.score)?;
        }
    } else {
        // Too big, must break it down. Emit whatever we have first.
        emit_buffer(chunks, html_buf, token_acc, node.score)?;

        for child in node.children {
            rebuild_to_chunks(child, config, chunks, html_buf, token_acc)?;
        }

        // Special case: if a terminal node (no children) is still too big
        if node.children.is_empty() && node.tokens > 0 {
            rebuild_html(node, html_buf, false);
            *token_acc += node.tokens;
            emit_buffer(chunks, html_buf, token_acc, node.score)?;
        }
    }
    Ok(())
}

fn emit_buffer(
    chunks: &mut std::vec::Vec<PageChunk>,
    html_buf: &mut String,
    token_acc: &mut usize,
    score: f32,
) -> Result<()> {
    if html_buf.is_empty() {
        return Ok(());
    }

    let raw_markdown = convert(html_buf).map_err(|e| Web2llmError::Markdown(e.to_string()))?;
    let content = wash_markdown(&raw_markdown);

    // Accurate final count
    let tokens = content
        .chars()
        .filter(|c: &char| !c.is_whitespace())
        .count()
        / 4
        + 1;

    chunks.push(PageChunk {
        index: chunks.len(),
        content,
        tokens,
        score,
    });

    html_buf.clear();
    *token_acc = 0;
    Ok(())
}

fn rebuild_html(node: &NodeMetrics, out: &mut String, inside_table: bool) {
    let tag = node.element.value().name();
    let tag_name = if inside_table && tag == "table" {
        "div"
    } else {
        tag
    };
    let next_inside_table = inside_table || tag == "table";

    out.push('<');
    out.push_str(tag_name);
    for (k, v) in node.element.value().attrs() {
        if k == "href" || k == "src" {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            out.push_str(v);
            out.push('"');
        }
    }
    out.push('>');

    let mut child_idx = 0;
    for child in node.element.children() {
        if let Some(text) = child.value().as_text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                out.push(' ');
                out.push_str(trimmed);
            }
        } else if scraper::ElementRef::wrap(child).is_some() && child_idx < node.children.len() {
            rebuild_html(&node.children[child_idx], out, next_inside_table);
            child_idx += 1;
        }
    }
    out.push_str("</");
    out.push_str(tag_name);
    out.push('>');
}

fn tag_multiplier(tag: &str) -> f32 {
    if HIGH_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_HIGH
    } else if MED_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_MED
    } else if LOW_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_LOW
    } else if POOR_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_POOR
    } else if PENALTY_TAGS.contains(&tag) {
        TAG_BONUS_PENALTY
    } else {
        TAG_BONUS_NEUTRAL
    }
}
