use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let config = Web2llmConfig {
        block_private_hosts: false,
        ..Default::default()
    };
    Web2llm::new(config).unwrap()
}

/// Verifies that an article scores higher than a nav element,
/// and only the article is included in the output.
#[tokio::test]
async fn test_article_scores_higher_than_nav() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <nav>
                    <ul><li>Home</li><li>About</li><li>Contact</li></ul>
                </nav>
                <article>
                    This is the main content of the page. It has more words than the navigation bar.
                    Therefore, it should receive a higher score and be the only part included.
                </article>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;

    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("main content"));
    assert!(!result.markdown.contains("Home"));
}

/// Verifies that if multiple content sections score highly, they are
/// both included in the final Markdown.
#[tokio::test]
async fn test_two_content_sections_both_included() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <section id="one">
                    This is the first section of high-quality content that should be kept.
                </section>
                <section id="two">
                    This is the second section of high-quality content that should also be kept.
                </section>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;

    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("first section"));
    assert!(result.markdown.contains("second section"));
}

/// Verifies that headers and footers are excluded when there's better
/// content available in an article.
#[tokio::test]
async fn test_footer_excluded_when_article_present() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <header><h1>Site Title</h1></header>
                <article>
                    This is the actual article content that we want to extract from the page.
                    It should be kept because it scores higher than the boilerplate.
                </article>
                <footer>Copyright 2024. All rights reserved. No part of this site...</footer>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;

    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("actual article content"));
    assert!(!result.markdown.contains("Copyright"));
}

/// Verifies that a single short article is still returned even if there's
/// nothing to compare it against, as long as it's not noise.
#[tokio::test]
async fn test_single_short_article_is_returned() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>
                    Just some short but meaningful text content.
                </article>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;

    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("meaningful text"));
}

/// Verifies that if all content scores poorly (below the absolute threshold),
/// an EmptyContent error is returned.
#[tokio::test]
async fn test_only_noise_returns_empty_content() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <nav>Short</nav>
                <footer>Short</footer>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;

    let result = test_client().fetch(&server.uri()).await;
    assert!(result.is_err());
}
