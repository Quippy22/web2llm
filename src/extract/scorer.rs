use crate::extract::ExtractedElement;
use scraper::{ElementRef, Html, Selector};

/// Multiplier for high-signal semantic tags — article, main, section.
const TAG_BONUS_HIGH: f32 = 2.0;
/// Multiplier for common content containers — div, p, span.
const TAG_BONUS_MED: f32 = 1.2;
/// Neutral multiplier for unknown tags — no opinion.
const TAG_BONUS_NEUTRAL: f32 = 1.0;
/// Reduced multiplier for tags unlikely to be primary content.
const TAG_BONUS_LOW: f32 = 0.8;
/// Heavily reduced multiplier for tags rarely containing prose.
const TAG_BONUS_POOR: f32 = 0.6;
/// Penalty multiplier for known noise tags — nav, footer, header etc.
const TAG_BONUS_PENALTY: f32 = 0.1;
/// Fixed score assigned to passthrough elements — bypasses the formula entirely.
const PASSTHROUGH_SCORE: f32 = 10.0;

/// Tags that are very likely to contain the main page content.
const HIGH_BONUS_TAGS: &[&str] = &["article", "main", "section"];
/// Tags that may contain content but are less reliable signals.
const MED_BONUS_TAGS: &[&str] = &["div", "p", "span"];
/// Tags that occasionally contain content but are weak signals.
const LOW_BONUS_TAGS: &[&str] = &["figure", "figcaption", "details"];
/// Tags that rarely contain prose — forms, controls, labels.
const POOR_BONUS_TAGS: &[&str] = &["form", "button", "label", "ul", "ol", "li"];
/// Tags that are almost never content — navigation, layout, chrome.
const PENALTY_TAGS: &[&str] = &["nav", "footer", "header", "aside", "menu"];
/// Tags always included regardless of score — structural content that bypasses the formula.
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
/// Tags excluded before any traversal — contain code or styles, never prose.
const SKIP_TAGS: &[&str] = &["script", "style", "noscript", "template"];

/// An owned, scraper-independent representation of a single HTML node.
/// Built once from the live scraper tree in `build_tree`, after which
/// the scraper document is dropped and this tree is traversed freely.
struct HtmlNode {
    tag: String,
    attrs: Vec<(String, String)>,
    /// Direct text nodes only — does not include text from child elements.
    text: String,
    children: Vec<HtmlNode>,
}

/// Internal result of visiting a single node during tree traversal.
/// Holds the cumulative score of this subtree and the reconstructed
/// html with penalty and skip subtrees removed.
/// Never exposed outside this module.
struct NodeResult {
    score: f32,
    html: String,
}

/// An element paired with its content score.
/// Higher score means more likely to be the main content.
/// The element's html has been cleaned — skip and penalty subtrees removed.
pub(crate) struct ScoredElement {
    pub(crate) element: ExtractedElement,
    pub(crate) score: f32,
}

/// Scores the body html and returns a filtered, sorted vec of [`ScoredElement`].
///
/// Parses the body into an owned [`HtmlNode`] tree, visits each top-level
/// branch to compute scores and rebuild clean html, then filters by a
/// dynamic threshold derived from the highest scoring branch.
///
/// `sensitivity` controls how aggressively secondary content is filtered.
/// A value of `0.1` keeps everything within 10x of the best scoring branch.
/// A value of `0.5` keeps only branches close to the best.
pub(crate) fn score(body_html: &str, sensitivity: f32) -> Vec<ScoredElement> {
    let wrapped = format!("<html><body>{}</body></html>", body_html);
    let document = Html::parse_document(&wrapped);
    let selector = Selector::parse("body > *").unwrap();

    let nodes: Vec<HtmlNode> = document
        .select(&selector)
        .map(|el| build_tree(el))
        .collect();

    // scraper document dropped here — nodes are fully owned from this point

    let results: Vec<(f32, ExtractedElement)> = nodes
        .iter()
        .map(|node| {
            let result = visit(node);
            (
                result.score,
                ExtractedElement {
                    tag: node.tag.clone(),
                    html: result.html,
                    text: node.text.clone(),
                },
            )
        })
        .filter(|(score, _)| *score > 0.0)
        .collect();

    let winner = results
        .iter()
        .map(|(score, _)| *score)
        .fold(0.0_f32, f32::max);

    let threshold = winner * sensitivity;

    results
        .into_iter()
        .filter(|(score, _)| *score >= threshold)
        .map(|(score, element)| ScoredElement { score, element })
        .collect()
}

/// Recursively builds an owned [`HtmlNode`] tree from a live scraper [`ElementRef`].
/// Collects tag name, attributes, direct text, and recursively builds all children.
/// This is the only function that touches scraper types — everything below works
/// on [`HtmlNode`] exclusively.
fn build_tree(node: ElementRef) -> HtmlNode {
    let tag = node.value().name().to_string();

    let attrs = node
        .value()
        .attrs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let text = node
        .children()
        .filter_map(|child| child.value().as_text())
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let children = node
        .children()
        .filter_map(ElementRef::wrap)
        .map(|child| build_tree(child))
        .collect();

    HtmlNode {
        tag,
        attrs,
        text,
        children,
    }
}

/// Recursively visits a node and returns its [`NodeResult`].
///
/// Processing order:
/// 1. Skip tags → empty result immediately, subtree discarded
/// 2. Passthrough tags → fixed [`PASSTHROUGH_SCORE`], children html rebuilt
/// 3. Everything else → recurse into children first, then score this node,
///    bubble total up to parent
///
/// This is a post-order traversal — children are always processed before
/// their parent.
fn visit(node: &HtmlNode) -> NodeResult {
    if is_skip(&node.tag) {
        return NodeResult {
            score: 0.0,
            html: String::new(),
        };
    }

    if is_passthrough(&node.tag) {
        let html = rebuild_html(
            node,
            node.children
                .iter()
                .map(|child| visit(child).html)
                .collect(),
        );
        return NodeResult {
            score: PASSTHROUGH_SCORE,
            html,
        };
    }

    let child_results: Vec<NodeResult> = node.children.iter().map(visit).collect();

    let children_score: f32 = child_results.iter().map(|r| r.score).sum();
    let children_html: Vec<String> = child_results.into_iter().map(|r| r.html).collect();

    let own_score = score_node(node);
    let total_score = own_score + children_score;

    let html = rebuild_html(node, children_html);

    NodeResult {
        score: total_score,
        html,
    }
}

/// Reconstructs the outer html tag for `node` with only the surviving
/// children's html inside. Penalty and skip subtrees are absent from
/// `children_html` — they were never added by `visit`.
fn rebuild_html(node: &HtmlNode, children_html: Vec<String>) -> String {
    let attrs = node
        .attrs
        .iter()
        .map(|(k, v)| format!(" {}=\"{}\"", k, v))
        .collect::<String>();

    let inner = std::iter::once(node.text.clone())
        .chain(children_html)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    format!("<{}{}>{}</{}>", node.tag, attrs, inner, node.tag)
}

/// Scores a single node's own direct contribution.
///
/// Formula: `(direct_words - link_words) × tag_multiplier`
///
/// Only counts direct text — child text is scored when those children
/// are visited. Link words are subtracted since they signal navigation
/// rather than readable content.
fn score_node(node: &HtmlNode) -> f32 {
    let multiplier = tag_multiplier(&node.tag);

    let total_words = node.text.split_whitespace().count() as f32;
    let link_words: f32 = node
        .children
        .iter()
        .filter(|child| child.tag == "a")
        .map(|child| child.text.split_whitespace().count() as f32)
        .sum();

    let content_words = (total_words - link_words).max(0.0);

    content_words * multiplier
}

/// Returns the score multiplier for a given tag name.
/// Ranges from [`TAG_BONUS_HIGH`] for strong content signals down to
/// [`TAG_BONUS_PENALTY`] for known noise tags. Unknown tags return
/// [`TAG_BONUS_NEUTRAL`].
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

/// Returns `true` if `tag` should be excluded before traversal.
/// These tags contain code or styles, never prose.
fn is_skip(tag: &str) -> bool {
    SKIP_TAGS.contains(&tag)
}

/// Returns `true` if `tag` should bypass scoring and receive a fixed score.
fn is_passthrough(tag: &str) -> bool {
    PASSTHROUGH_TAGS.contains(&tag)
}
