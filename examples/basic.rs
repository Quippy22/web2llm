use web2llm::fetch;

#[tokio::main]
async fn main() {
    // Fetch a page with default settings
    let result = fetch("https://example.com".to_string()).await.unwrap();

    // Print the cleaned Markdown
    println!("{}", result.markdown());
}
