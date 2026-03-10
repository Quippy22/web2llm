mod extract;
mod fetch;

use scraper::Html;

use extract::PageElements;
use fetch::get_html;

#[tokio::main]
async fn main() {
    let url: &str = "https://www.rust-lang.org/learn";
    let html = get_html(url).await.unwrap();

    let document = Html::parse_document(&html);
    let page = PageElements::parse(document);
    let md = page.to_markdown();

    print!("{}", md);
}
