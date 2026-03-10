use scraper::Html;
use web2llm::extract::PageElements;

#[test]
fn test_nav_scores_lower_than_article() {
    let html = r#"
        <html><body>
            <nav>Home About Contact Blog Portfolio Links More Links Even More Links</nav>
            <article>
                This is a long article with plenty of meaningful content and real sentences.
                It talks about interesting topics and has enough words to pass the threshold.
                The scorer should strongly prefer this over the navigation element above.
            </article>
        </body></html>
    "#;

    let document = Html::parse_document(html);
    let page = PageElements::parse(document);
    let markdown = page.to_markdown();

    assert!(markdown.contains("meaningful content"));
    assert!(!markdown.contains("Home About Contact"));
}

#[test]
fn test_short_elements_are_excluded() {
    let html = r#"
        <html><body>
            <p>Too short</p>
            <article>
                This article has enough words to pass the minimum word count threshold
                and should be included in the scored results returned by the scorer.
            </article>
        </body></html>
    "#;

    let document = Html::parse_document(html);
    let page = PageElements::parse(document);
    let markdown = page.to_markdown();

    assert!(markdown.contains("enough words"));
    assert!(!markdown.contains("Too short"));
}
