use scraper::Html;
use web2llm::extract::PageElements;

#[test]
fn test_parse_extracts_body_content() {
    let html = r#"
        <html>
            <head><title>Test</title></head>
            <body>
                <nav>Home About Contact</nav>
                <article>
                    This is the main content of the page with enough words to pass the threshold.
                    It has multiple sentences and should score well.
                </article>
                <footer>Copyright 2024</footer>
            </body>
        </html>
    "#;

    let document = Html::parse_document(html);
    let page = PageElements::parse(document);
    let markdown = page.to_markdown();

    assert!(markdown.contains("main content"));
    assert!(!markdown.contains("Home About Contact"));
    assert!(!markdown.contains("Copyright"));
}
