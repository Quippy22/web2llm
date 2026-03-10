use web2llm::error::Result;
use web2llm::extract::PageElements;

#[tokio::main]
async fn main() -> Result<()> {
    let url: &str = "https://www.rust-lang.org/learn";
    let page = PageElements::parse(url).await?;
    let md = page.to_markdown()?;

    print!("{}", md);
    Ok(())
}
