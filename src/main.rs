mod extract;
mod fetch;

use scraper::node::Node;
use scraper::{Html, Selector};

use fetch::get_html;

#[tokio::main]
async fn main() {
    let html = get_html("https://example.com").await.unwrap();

    let document = Html::parse_document(&html);
    let selector = Selector::parse("*").unwrap();

    for element in document.select(&selector) {
        let name = element.value().name();

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

        let link = element.value().attr("href").unwrap_or("");

        println!("{}", format!("{}: {} {}", name, text, link));
    }
}
