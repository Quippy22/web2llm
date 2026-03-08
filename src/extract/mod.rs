use scraper::{Html, Selector, node::Node};

pub struct ExtractedElement {
    pub tag: String,
    pub html: String,
    pub text: String,
    pub link: String,
}

pub struct ScoredElement {
    pub element: ExtractedElement,
    pub score: f32,
}

pub struct PageElements {
    pub elements: Vec<ExtractedElement>,
}

impl PageElements {
    pub fn parse(html: Html) -> Self {
        // Parses the body, builds elements vec!
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

            let link = element.value().attr("href").unwrap_or("").to_string();

            let extracted = ExtractedElement {
                tag,
                html,
                text,
                link,
            };
            elements.push(extracted);
        }

        Self { elements }
    }

    pub fn score(&self) -> Vec<ScoredElement> {
        //let scores = scorer::score(&self.elements);
        todo!()
    }

    pub fn get_links(&self) -> Vec<String> {
        // TODO: collects links from elements
        todo!()
    }

    pub fn to_markdown(&self) -> String {
        // TODO: converts the best scored elements to html
        todo!()
    }
}
