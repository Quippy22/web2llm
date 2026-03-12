use crate::extract::{ExtractedElement, ScoredElement};
use scraper::{Html, Selector};

/// High scoring bonus for semantic content tags.
const TAG_BONUS_HIGH: f32 = 2.0;
/// Moderate scoring bonus for common content containers.
const TAG_BONUS_MED: f32 = 1.2;
/// Fixed score assigned to passthrough elements.
const PASSTHROUGH_SCORE: f32 = 1.0;

/// Minimum number of direct words an element must have to be considered.
const MIN_WORD_COUNT: u32 = 10;

/// Tags that are very likely to contain the main page content.
const HIGH_BONUS_TAGS: &[&str] = &["article", "main", "section"];
/// Tags that may contain content but are less reliable signals.
const MED_BONUS_TAGS: &[&str] = &["div", "p", "blockquote"];
/// Tags that are almost never content — navigation, layout, chrome.
const PENALTY_TAGS: &[&str] = &["nav", "footer", "header", "aside", "menu"];
/// Tags excluded before scoring — contain code or styles, never prose.
const SKIP_TAGS: &[&str] = &["script", "style", "noscript", "template"];
/// Tags that are always included regardless of score — structural content.
const PASSTHROUGH_TAGS: &[&str] = &[
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "table",
    "pre",
    "code",
    "blockquote",
];

/// Scores a slice of extracted elements and returns only those with a
/// positive score. Elements below the word threshold or with penalized
/// tags are excluded entirely (score = 0.0).
pub(crate) fn score(elements: &[ExtractedElement]) -> Vec<ScoredElement> {
    elements
        .iter()
        .filter_map(|e| {
            classify(e).map(|score| ScoredElement {
                score,
                element: e.clone(),
            })
        })
        .collect()
}

fn classify(element: &ExtractedElement) -> Option<f32> {
    let tag = element.tag.as_str();

    if is_skip_tag(tag) {
        return None;
    }

    if PENALTY_TAGS.contains(&tag) {
        return None;
    }

    if PASSTHROUGH_TAGS.contains(&tag) {
        return Some(PASSTHROUGH_SCORE);
    }

    calculate_score(element)
}

/// Calculates a content score for a single element.
///
/// The formula is:
/// `word_count * text_to_html_ratio * tag_bonus * (1 - link_density)`
///
/// - `word_count` rewards elements with more direct text
/// - `text_to_html_ratio` penalizes elements with lots of structural noise
/// - `tag_bonus` rewards semantic tags and penalizes navigation tags
/// - `link_density` penalizes elements where most text is inside `<a>` tags
fn calculate_score(element: &ExtractedElement) -> Option<f32> {
    let word_count = {
        let wc = element.text.split_whitespace().count() as f32;
        if wc < MIN_WORD_COUNT as f32 {
            return None;
        } else {
            wc
        }
    };
    let text_to_html_ratio = {
        let text_len = element.text.len() as f32;
        let html_len = element.html.len() as f32;
        if html_len == 0.0 {
            return None;
        } else {
            text_len / html_len
        }
    };
    let tag_bonus = calculate_tag_bonus(&element.tag);
    let link_density_penalty = link_density(element);
    let score = word_count * text_to_html_ratio * tag_bonus * (1.0 - link_density_penalty);

    if score > 0.0 { Some(score) } else { None }
}

/// Returns a score multiplier based on the element's tag name.
/// Semantic content tags are boosted, unknown tags get a neutral 1.0.
/// Penalty and skip tags never reach this function — handled in `classify`.
fn calculate_tag_bonus(tag: &str) -> f32 {
    if HIGH_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_HIGH
    } else if MED_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_MED
    } else {
        1.0
    }
}

/// Calculates the ratio of words inside `<a>` tags to total words.
/// A high ratio suggests the element is mostly navigation links
/// rather than readable content.
/// Returns a value between 0.0 (no links) and 1.0 (all text is links).
fn link_density(element: &ExtractedElement) -> f32 {
    let document = Html::parse_fragment(&element.html);
    let a_selector = Selector::parse("a").unwrap();
    let link_words: usize = document
        .select(&a_selector)
        .map(|a| a.text().collect::<String>())
        .map(|t| t.split_whitespace().count())
        .sum();
    let total_words = element.text.split_whitespace().count();
    if total_words == 0 {
        return 0.0;
    }
    link_words as f32 / total_words as f32
}

/// Returns `true` if `tag` should be excluded before scoring.
/// These tags contain code or styles, never prose.
/// Called from `mod.rs` during element collection, before `classify`.
pub(crate) fn is_skip_tag(tag: &str) -> bool {
    SKIP_TAGS.contains(&tag)
}
