use web2llm::error::Result;
use web2llm::extract::PageElements;
use web2llm::output::PageResult;

#[tokio::main]
async fn main() -> Result<()> {
    let url: &str = "https://www.rust-lang.org/learn";
    let page = PageElements::parse(url).await?;
    let result: PageResult = page.into_result()?;

    print!("{}", result.markdown);
    Ok(())
}
