use crate::config::Web2llmConfig;
use crate::error::{Result, Web2llmError};
use crate::tokens::wash_markdown;
use crate::tokens::{PageChunk, get_direct_text_metrics, is_within_budget};
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

pub(crate) struct NodeMetrics<'a> {
    pub(crate) score: f32,
    pub(crate) tokens: usize,
    pub(crate) element: ElementRef<'a>,
    pub(crate) children: Vec<NodeMetrics<'a>>,
}

/// Entry point: Processes the body and returns structurally-aware Markdown chunks.
pub(crate) fn process(body: ElementRef, config: &Web2llmConfig) -> Result<Vec<PageChunk>> {
    let mut roots: Vec<NodeMetrics> = body
        .children()
        .filter_map(ElementRef::wrap)
        .map(compute_metrics)
        .filter(|e| e.score > 0.0)
        .collect();

    let winner = roots.iter().map(|e| e.score).fold(0.0_f32, f32::max);
    if winner < MIN_SCORE_THRESHOLD {
        return Ok(Vec::new());
    }

    let threshold = winner * config.sensitivity;
    let prune_threshold = threshold * 0.01;

    // Filter and prune tree
    roots.retain(|e| e.score >= threshold);
    for node in &mut roots {
        prune_node(node, prune_threshold);
    }

    // Convert pruned tree to chunks
    let mut chunks = Vec::new();
    for root in roots {
        rebuild_to_chunks(&root, config, &mut chunks)?;
    }

    Ok(chunks)
}

fn compute_metrics(node: ElementRef<'_>) -> NodeMetrics<'_> {
    let tag = node.value().name();
    if SKIP_TAGS.contains(&tag) {
        return NodeMetrics {
            score: 0.0,
            tokens: 0,
            element: node,
            children: Vec::new(),
        };
    }

    let (own_words, own_tokens) = get_direct_text_metrics(node);
    let mut children_score = 0.0;
    let mut children_tokens = 0;
    let mut children = Vec::new();

    for child in node.children().filter_map(ElementRef::wrap) {
        let metrics = compute_metrics(child);
        children_score += metrics.score;
        children_tokens += metrics.tokens;
        children.push(metrics);
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
        children,
    }
}

fn prune_node(node: &mut NodeMetrics, threshold: f32) {
    node.children.retain(|child| {
        let tag = child.element.value().name();
        if PENALTY_TAGS.contains(&tag) {
            child.score >= threshold
        } else {
            true
        }
    });
    for child in &mut node.children {
        prune_node(child, threshold);
    }
}

fn rebuild_to_chunks(
    node: &NodeMetrics,
    config: &Web2llmConfig,
    chunks: &mut Vec<PageChunk>,
) -> Result<()> {
    if is_within_budget(node.tokens, config.max_tokens) {
        let mut html = String::with_capacity(node.tokens * 4);
        rebuild_html(node, &mut html, false);
        if !html.is_empty() {
            let raw_markdown = convert(&html).map_err(|e| Web2llmError::Markdown(e.to_string()))?;
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
                score: node.score,
            });
        }
    } else {
        for child in &node.children {
            rebuild_to_chunks(child, config, chunks)?;
        }
        // Handle oversized terminal nodes
        if node.children.is_empty() && node.tokens > 0 {
            let mut html = String::with_capacity(node.tokens * 4);
            rebuild_html(node, &mut html, false);
            let raw_markdown = convert(&html).map_err(|e| Web2llmError::Markdown(e.to_string()))?;
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
                score: node.score,
            });
        }
    }
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
        } else if let Some(_el) = ElementRef::wrap(child) {
            if child_idx < node.children.len() {
                rebuild_html(&node.children[child_idx], out, next_inside_table);
                child_idx += 1;
            }
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
