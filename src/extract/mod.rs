mod scorer;
use htmd::convert;
use scraper::{Html, Selector, node::Node};

/// A single element extracted from the page body.
/// Holds only direct text (not inherited from children)
/// and the full inner HTML for downstream conversion.
#[derive(Clone)]
pub(crate) struct ExtractedElement {
    pub(crate) tag: String,  // tag name e.g. "article", "div", "p"
    pub(crate) html: String, // full inner HTML including all children
    pub(crate) text: String, // direct text nodes only, used for scoring
}

/// An element paired with its content score.
/// Higher score means more likely to be the main content.
pub(crate) struct ScoredElement {
    pub(crate) element: ExtractedElement,
    pub(crate) score: f32,
}

/// The main extraction type. Parses a page's body into scoreable
/// elements and converts the best candidates to Markdown.
pub struct PageElements {
    elements: Vec<ExtractedElement>,
}

impl PageElements {
    /// Walks every element inside `<body>`, collecting tag name,
    /// inner HTML, and direct text nodes into a flat vec.
    /// Head is ignored entirely — only body content is relevant.
    pub fn parse(html: Html) -> Self {
        let selector = Selector::parse("body *").unwrap();
        let mut elements: Vec<ExtractedElement> = Vec::new();

        for element in html.select(&selector) {
            let tag = element.value().name().to_string();
            let html = element.inner_html();

            // Only collect direct text nodes, not inherited from children.
            // This ensures scoring reflects the element's own content density.
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

            elements.push(ExtractedElement { tag, html, text });
        }

        Self { elements }
    }

    /// Scores all elements and returns them sorted by score descending.
    /// Elements scoring 0.0 (below word threshold or penalized tags) are excluded.
    fn score(&self) -> Vec<ScoredElement> {
        let mut scored = scorer::score(&self.elements);
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scored
    }

    /// Converts all positively scored elements to Markdown and joins them.
    /// Each element's full inner HTML is passed to htmd, preserving
    /// nested structure, links, and formatting.
    pub fn to_markdown(&self) -> String {
        self.score()
            .iter()
            .map(|s| convert(&s.element.html).unwrap())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
