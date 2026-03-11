use web2llm::error::Result;
use web2llm::fetch;

#[tokio::main]
async fn main() -> Result<()> {
    let url: &str = "https://www.rust-lang.org/learn";
    let result = fetch(url).await?;

    print!("{}", result.markdown);
    Ok(())
}
