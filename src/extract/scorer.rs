use scraper::ElementRef;

/// Multiplier for high-signal semantic tags — article, main, section.
const TAG_BONUS_HIGH: f32 = 2.0;
/// Multiplier for common content containers — div, p, span.
/// Set to neutral (1.0) to prevent layout-heavy nesting from inflating scores.
const TAG_BONUS_MED: f32 = 1.0;
/// Neutral multiplier for unknown tags — no opinion.
const TAG_BONUS_NEUTRAL: f32 = 1.0;
/// Reduced multiplier for tags unlikely to be primary content.
const TAG_BONUS_LOW: f32 = 0.7;
/// Heavily reduced multiplier for tags rarely containing prose — forms, lists, etc.
const TAG_BONUS_POOR: f32 = 0.5;
/// Penalty multiplier for known noise tags — nav, footer, header etc.
/// Applied to the entire subtree via compounding to wipe out navigation noise.
const TAG_BONUS_PENALTY: f32 = 0.05;
/// Fixed score assigned to passthrough elements — bypasses the formula entirely.
const PASSTHROUGH_SCORE: f32 = 10.0;

/// Tags that are very likely to contain the main page content.
const HIGH_BONUS_TAGS: &[&str] = &["article", "main", "section"];
/// Tags that may contain content but are less reliable signals.
const MED_BONUS_TAGS: &[&str] = &[
    "div", "p", "span", "table", "thead", "tbody", "tfoot", "tr", "th", "td",
];
/// Tags that occasionally contain content but are weak signals.
const LOW_BONUS_TAGS: &[&str] = &["figure", "figcaption", "details"];
/// Tags rarely contain prose — forms, controls, labels, lists.
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

/// Absolute minimum score required for a branch to be considered content.
/// This prevents pages containing only noise (nav/footer) from returning
/// those noise blocks as "main content".
const MIN_SCORE_THRESHOLD: f32 = 5.0;

/// A scored content block ready for Markdown conversion.
/// `score` is the cumulative score of the subtree.
/// `html` is the cleaned html with skip and penalty subtrees removed.
pub(crate) struct ScoredElement {
    pub(crate) score: f32,
    pub(crate) html: String,
}

/// Scores the body element and returns a filtered vec of [`ScoredElement`].
///
/// Visits each top-level child of the body to compute scores and rebuild clean html,
/// then filters by a dynamic threshold derived from the highest scoring branch.
///
/// `sensitivity` controls how aggressively secondary content is filtered.
/// A value of `0.1` keeps everything within 10x of the best scoring branch.
/// A value of `0.5` keeps only branches close to the best.
pub(crate) fn score(body: ElementRef, sensitivity: f32) -> Vec<ScoredElement> {
    let results: Vec<(f32, ElementRef)> = body
        .children()
        .filter_map(ElementRef::wrap)
        .map(|el| (compute_score(el), el))
        .filter(|(s, _)| *s > 0.0)
        .collect();

    let winner = results.iter().map(|(s, _)| *s).fold(0.0_f32, f32::max);
    if winner < MIN_SCORE_THRESHOLD {
        return Vec::new();
    }

    let threshold = winner * sensitivity;

    results
        .into_iter()
        .filter(|(s, _)| *s >= threshold)
        .map(|(s, el)| {
            let mut html = String::with_capacity(8192);
            // Use a much more lenient threshold for deep pruning to avoid
            // removing fragmented content in large containers.
            // This multiplier (0.01) allows small prose blocks to survive
            // while still killing deeply nested noise tags.
            let prune_threshold = threshold * 0.01;
            rebuild_html(el, &mut html, prune_threshold, false);
            ScoredElement { score: s, html }
        })
        .collect()
}

/// Recursively calculates the score for a subtree rooted at `node`.
///
/// Scores are computed as: `(own_words + children_scores) * multiplier`.
/// By applying the multiplier to the sum, penalties (like `nav`) correctly
/// propagate down to all children, effectively "wiping out" noise subtrees.
fn compute_score(node: ElementRef) -> f32 {
    let tag = node.value().name();

    if is_skip(tag) {
        return 0.0;
    }

    if is_passthrough(tag) {
        return PASSTHROUGH_SCORE;
    }

    let multiplier = tag_multiplier(tag);
    let own_words = get_direct_text_word_count(node);

    let children_score: f32 = node
        .children()
        .filter_map(ElementRef::wrap)
        .map(compute_score)
        .sum();

    (own_words + children_score) * multiplier
}

/// Counts words in the direct text nodes of `node`.
///
/// Optimized to avoid allocations and multiple passes over the string by
/// counting words directly from the character stream of each text child.
fn get_direct_text_word_count(node: ElementRef) -> f32 {
    let mut total_count = 0;
    for child in node.children() {
        if let Some(text) = child.value().as_text() {
            let mut in_word = false;
            for c in text.chars() {
                if c.is_whitespace() {
                    in_word = false;
                } else if !in_word {
                    total_count += 1;
                    in_word = true;
                }
            }
        }
    }
    total_count as f32
}

/// Recursively appends cleaned HTML to `out`.
///
/// Reconstruction logic:
/// 1. Prunes `PENALTY_TAGS` subtrees whose cumulative score is below `threshold`.
/// 2. Flattens nested `<table>` tags to `<div>` to prevent "pipe table mess".
/// 3. Strips all attributes except `href` and `src` for token efficiency.
/// 4. Preserves inline content (`a`, `code`, `span`) regardless of score to prevent redaction.
fn rebuild_html(node: ElementRef, out: &mut String, threshold: f32, inside_table: bool) {
    let tag = node.value().name();

    if is_skip(tag) {
        return;
    }

    // Only prune explicitly recognized "noise" tags (nav, footer, aside, etc.)
    // if their cumulative score is below threshold.
    // Generic containers (div, section) and content tags (p, a, code, span)
    // are ALWAYS preserved to prevent "inline content stripping" where
    // short but essential technical terms or hyperlinked words disappear.
    if PENALTY_TAGS.contains(&tag) && compute_score(node) < threshold {
        return;
    }

    // Markdown tables do not support nesting. If we are already inside a table,
    // flatten nested <table> tags to generic <div> containers while preserving
    // tr/td tags to maintain structure without creating a "pipe table mess".
    let tag_name = if inside_table && tag == "table" {
        "div"
    } else {
        tag
    };
    let next_inside_table = inside_table || tag == "table";

    out.push('<');
    out.push_str(tag_name);

    // Attribute stripping: Keep only essential attributes for Markdown conversion.
    // This reduces the token footprint and prevents htmd from adding noise.
    for (k, v) in node.value().attrs() {
        if k == "href" || k == "src" {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            out.push_str(v);
            out.push('"');
        }
    }
    out.push('>');

    for child in node.children() {
        if let Some(text) = child.value().as_text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                out.push(' ');
                out.push_str(trimmed);
            }
        } else if let Some(el) = ElementRef::wrap(child) {
            out.push('\n');
            rebuild_html(el, out, threshold, next_inside_table);
        }
    }

    out.push_str("</");
    out.push_str(tag_name);
    out.push('>');
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
