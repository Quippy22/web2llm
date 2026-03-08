mod extract;
mod fetch;

use scraper::Html;

use extract::PageElements;
use fetch::get_html;

#[tokio::main]
async fn main() {
    let html = get_html("https://example.com").await.unwrap();

    let document = Html::parse_document(&html);
    let page = PageElements::parse(document);

    for elem in page.elements.iter() {
        println!("{}", format!("{}: {} {}", elem.tag, elem.text, elem.link));
    }
}
