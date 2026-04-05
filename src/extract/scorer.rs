//! The core extraction and scoring engine of `web2llm`.
//!
//! This module implements a recursive N-ary tree traversal that assigns quality
//! scores to every node in the DOM. It uses these scores to prune noise
//! (navigation, footers, sidebars) and divide the remaining content into
//! structurally-aware Markdown chunks.

use crate::config::Web2llmConfig;
use crate::error::{Result, Web2llmError};
use crate::tokens::wash_markdown;
use crate::tokens::{PageChunk, get_direct_text_metrics, is_within_budget};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use htmd::convert;
use tl::{Node, NodeHandle, Parser};

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
#[derive(Clone, Copy)]
pub(crate) struct NodeMetrics<'a> {
    /// The cumulative quality score of this node and its entire subtree.
    pub(crate) score: f32,
    /// The estimated number of tokens in this subtree.
    pub(crate) tokens: usize,
    /// A handle to the original DOM node.
    pub(crate) handle: NodeHandle,
    /// Pre-scored child nodes that survived the pruning phase.
    pub(crate) children: &'a [NodeMetrics<'a>],
}

/// Entry point: Processes the body and returns structurally-aware Markdown chunks.
pub(crate) fn process(
    body_handle: NodeHandle,
    parser: &Parser,
    config: &Web2llmConfig,
) -> Result<std::vec::Vec<PageChunk>> {
    let bump = Bump::with_capacity(64 * 1024);

    let body_node = body_handle.get(parser).unwrap();
    let body_tag = body_node.as_tag().unwrap();

    let mut roots: std::vec::Vec<NodeMetrics> = body_tag
        .children()
        .top()
        .iter()
        .filter(|&h| h.get(parser).and_then(|n| n.as_tag()).is_some())
        .map(|h| compute_metrics(*h, parser, &bump, config.sensitivity * 0.01))
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
        rebuild_to_chunks(
            &root,
            parser,
            config,
            &mut chunks,
            &mut html_buf,
            &mut token_acc,
        )?;
        emit_buffer(&mut chunks, &mut html_buf, &mut token_acc, root.score)?;
    }

    Ok(chunks)
}

/// Recursively computes extraction metrics for a node and its subtree.
///
/// This is the core of the scoring engine. It performs a bottom-up traversal,
/// calculating quality scores based on text density, tag bonuses, and child scores.
fn compute_metrics<'a>(
    node_handle: NodeHandle,
    parser: &Parser,
    bump: &'a Bump,
    prune_threshold: f32,
) -> NodeMetrics<'a> {
    let node = node_handle.get(parser).unwrap();
    let tag = node.as_tag().unwrap();
    let tag_name = std::str::from_utf8(tag.name().as_bytes()).unwrap_or("");

    if SKIP_TAGS.contains(&tag_name) {
        return NodeMetrics {
            score: 0.0,
            tokens: 0,
            handle: node_handle,
            children: &[],
        };
    }

    let (own_words, own_tokens) = get_direct_text_metrics(node_handle, parser);
    let mut children_score = 0.0;
    let mut children_tokens = 0;
    let mut children = BumpVec::new_in(bump);

    for child_handle in tag.children().top().iter() {
        if let Some(tag_node) = child_handle.get(parser).and_then(|n| n.as_tag()) {
            let child_tag_name = std::str::from_utf8(tag_node.name().as_bytes()).unwrap_or("");
            let metrics = compute_metrics(*child_handle, parser, bump, prune_threshold);

            if PENALTY_TAGS.contains(&child_tag_name) && metrics.score < prune_threshold {
                continue;
            }

            if metrics.score > 0.0 || PASSTHROUGH_TAGS.contains(&child_tag_name) {
                children_score += metrics.score;
                children_tokens += metrics.tokens;
                children.push(metrics);
            }
        }
    }

    let is_pass = PASSTHROUGH_TAGS.contains(&tag_name);
    let multiplier = if is_pass {
        1.0
    } else {
        tag_multiplier(tag_name)
    };
    let score = if is_pass {
        PASSTHROUGH_SCORE + children_score
    } else {
        (own_words + children_score) * multiplier
    };

    NodeMetrics {
        score,
        tokens: own_tokens + children_tokens,
        handle: node_handle,
        children: children.into_bump_slice(),
    }
}

/// Recursively traverses the scored tree to produce structurally-aware chunks.
///
/// If a node and its entire subtree fit within the `max_tokens` budget (plus a 10% soft limit),
/// it is added to the current chunk. Otherwise, it is broken down into its children.
fn rebuild_to_chunks(
    node: &NodeMetrics,
    parser: &Parser,
    config: &Web2llmConfig,
    chunks: &mut std::vec::Vec<PageChunk>,
    html_buf: &mut String,
    token_acc: &mut usize,
) -> Result<()> {
    if is_within_budget(node.tokens, config.max_tokens) {
        rebuild_html(node, parser, html_buf, false);
        *token_acc += node.tokens;

        if *token_acc >= config.max_tokens {
            emit_buffer(chunks, html_buf, token_acc, node.score)?;
        }
    } else {
        emit_buffer(chunks, html_buf, token_acc, node.score)?;

        for child in node.children {
            rebuild_to_chunks(child, parser, config, chunks, html_buf, token_acc)?;
        }

        if node.children.is_empty() && node.tokens > 0 {
            rebuild_html(node, parser, html_buf, false);
            *token_acc += node.tokens;
            emit_buffer(chunks, html_buf, token_acc, node.score)?;
        }
    }
    Ok(())
}

/// Converts the accumulated HTML buffer into a Markdown chunk and adds it to the list.
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

/// Rebuilds a clean HTML string from a node and its children.
/// Strips non-essential attributes and flattens nested tables.
#[allow(clippy::collapsible_if)]
fn rebuild_html(node: &NodeMetrics, parser: &Parser, out: &mut String, inside_table: bool) {
    let tag_node = node.handle.get(parser).unwrap().as_tag().unwrap();
    let tag = std::str::from_utf8(tag_node.name().as_bytes()).unwrap_or("");

    let tag_name = if inside_table && tag == "table" {
        "div"
    } else {
        tag
    };
    let next_inside_table = inside_table || tag == "table";

    out.push('<');
    out.push_str(tag_name);
    for (k, v) in tag_node.attributes().iter() {
        if k.eq_ignore_ascii_case("href") || k.eq_ignore_ascii_case("src") {
            if let Some(val) = v {
                if let Ok(val_str) = std::str::from_utf8(val.as_bytes()) {
                    out.push(' ');
                    out.push_str(&k);
                    out.push_str("=\"");
                    out.push_str(val_str);
                    out.push('"');
                }
            }
        }
    }
    out.push('>');

    let mut child_idx = 0;
    for child_handle in tag_node.children().top().iter() {
        if let Some(Node::Raw(text_bytes)) = child_handle.get(parser) {
            if let Ok(text) = std::str::from_utf8(text_bytes.as_bytes()) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    out.push(' ');
                    out.push_str(trimmed);
                }
            }
        } else if child_handle.get(parser).and_then(|n| n.as_tag()).is_some()
            && child_idx < node.children.len()
        {
            rebuild_html(&node.children[child_idx], parser, out, next_inside_table);
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
