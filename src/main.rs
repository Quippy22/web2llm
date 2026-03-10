use scraper::Html;

use web2llm::error::Result;
use web2llm::extract::PageElements;
use web2llm::fetch::get_html;

#[tokio::main]
async fn main() -> Result<()> {
    let url: &str = "https://www.rust-lang.org/learn";
    let html = get_html(url).await?;

    let document = Html::parse_document(&html);
    let page = PageElements::parse(document);
    let md = page.to_markdown()?;

    print!("{}", md);

    Ok(())
}
