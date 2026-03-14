use web2llm::fetch;

#[tokio::main]
async fn main() {
    // Convenience function to fetch a single page with default config
    match fetch("https://example.com").await {
        Ok(result) => {
            println!("Title: {}", result.title);
            println!("URL: {}", result.url);
            println!("---\n\n{}", result.markdown);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
