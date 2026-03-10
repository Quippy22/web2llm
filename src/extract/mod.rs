mod scorer;

use htmd::convert;
use scraper::{Html, Selector, node::Node};

#[derive(Clone)]
pub(crate) struct ExtractedElement {
    pub(crate) tag: String,
    pub(crate) html: String,
    pub(crate) text: String,
}

pub(crate) struct ScoredElement {
    pub(crate) element: ExtractedElement,
    pub(crate) score: f32,
}

pub struct PageElements {
    elements: Vec<ExtractedElement>,
}

impl PageElements {
    pub fn parse(html: Html) -> Self {
        // Parses the body, builds elements vec
        // Has a depth of 1, but it returns the elements with all their children
        let selector = Selector::parse("body *").unwrap();
        let mut elements: Vec<ExtractedElement> = Vec::new();

        for element in html.select(&selector) {
            let tag = element.value().name().to_string();
            let html = element.inner_html();
            let text: String = element
                .children()
                .filter_map(|child| {
                    if let Node::Text(t) = child.value() {
                        let trimmed = t.trim();
                        if !trimmed.is_empty() {
                            Some(trimmed.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            let extracted = ExtractedElement { tag, html, text };
            elements.push(extracted);
        }

        Self { elements }
    }

    fn score(&self) -> Vec<ScoredElement> {
        let mut scored = scorer::score(&self.elements);
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored
    }

    pub fn to_markdown(&self) -> String {
        // converts the best scored elements to html
        self.score()
            .iter()
            .map(|s| convert(&s.element.html).unwrap())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
