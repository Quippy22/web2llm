use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let mut config = Web2llmConfig::default();
    config.block_private_hosts = false;
    Web2llm::new(config)
}

/// Article with rich content should score above nav with same word count but all links.
#[tokio::test]
async fn test_article_scores_higher_than_nav() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <nav>Home About Contact Blog Portfolio Links More Links Even More Links</nav>
                <article>
                    This is a long article with plenty of meaningful content and real sentences.
                    It talks about interesting topics and has enough words to score well.
                    The scorer should strongly prefer this over the navigation element above.
                </article>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("meaningful content"));
    assert!(!result.markdown.contains("Home About Contact"));
}

/// Footer with penalty multiplier should lose to article even with similar word count.
#[tokio::test]
async fn test_footer_excluded_when_article_present() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>
                    This is a long article with plenty of meaningful content and real sentences.
                    It talks about interesting topics and has enough words to score highly.
                    The scorer should strongly prefer this over the footer element below.
                </article>
                <footer>
                    Privacy Policy Terms of Use Contact Us About Us Careers Press
                    Cookie Settings Accessibility Sitemap Help Center Support
                </footer>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("meaningful content"));
    assert!(!result.markdown.contains("Privacy Policy"));
}

/// A very short page with only one content block should still return that block.
#[tokio::test]
async fn test_single_short_article_is_returned() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>Short but only content on the page so it wins by default.</article>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("wins by default"));
}

/// Two content sections — both should appear since secondary is within 10x of winner.
#[tokio::test]
async fn test_two_content_sections_both_included() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <section>
                    This is the primary content section with lots of meaningful words and sentences.
                    It should score as the winner and set the threshold for everything else.
                </section>
                <section>
                    This is a secondary content section also with meaningful words and sentences.
                    It should survive the threshold since it is close in score to the winner.
                </section>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("primary content section"));
    assert!(result.markdown.contains("secondary content section"));
}

/// Truly empty body should return EmptyContent error.
#[tokio::test]
async fn test_only_noise_returns_empty_content() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html><body></body></html>"))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await;
    assert!(result.is_err());
}
