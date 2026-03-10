use scraper::{Html, Selector};

use crate::extract::{ExtractedElement, ScoredElement};

const TAG_BONUS_HIGH: f32 = 2.0;
const TAG_BONUS_MED: f32 = 1.2;
const TAG_PENALITY: f32 = 0.0;
const MIN_WORD_COUNT: u32 = 10;

const HIGH_BONUS_TAGS: &[&str] = &["article", "main", "section"];
const MED_BONUS_TAGS: &[&str] = &["div", "p", "blockquote"];
const PENALTY_TAGS: &[&str] = &["nav", "footer", "header", "aside", "menu"];

pub fn score(elements: &[ExtractedElement]) -> Vec<ScoredElement> {
    elements
        .iter()
        .map(|e| ScoredElement {
            score: calculate_score(e),
            element: e.clone(),
        })
        .filter(|s| s.score > 0.0)
        .collect()
}

fn calculate_score(element: &ExtractedElement) -> f32 {
    let word_count = {
        let wc = element.text.split(" ").count() as f32;
        if wc < MIN_WORD_COUNT as f32 { 0.0 } else { wc }
    };
    let text_to_html_ration = {
        let text_len = element.text.len() as f32;
        let html_len = element.html.len() as f32;
        if html_len == 0.0 {
            0.0
        } else {
            text_len / html_len
        }
    };
    let tag_bonus = calculate_tag_bonus(&element.tag);
    let link_density_penalty = link_density(element);

    word_count * text_to_html_ration * tag_bonus * (1.0 - link_density_penalty)
}

fn calculate_tag_bonus(tag: &str) -> f32 {
    if HIGH_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_HIGH
    } else if MED_BONUS_TAGS.contains(&tag) {
        TAG_BONUS_MED
    } else if PENALTY_TAGS.contains(&tag) {
        TAG_PENALITY
    } else {
        1.0
    }
}

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
