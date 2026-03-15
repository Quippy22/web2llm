use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let mut config = Web2llmConfig::default();
    config.block_private_hosts = false;
    Web2llm::new(config).unwrap()
}

/// A full end-to-end test verifying that a 200 OK response with content
/// is successfully fetched, scored, and converted to Markdown.
#[tokio::test]
async fn test_fetch_returns_markdown_on_200() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>
                    This is the main content of the page with enough words to pass the threshold.
                    It has multiple sentences and should score well above the minimum word count.
                </article>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("main content"));
}

/// Verifies that a 404 Not Found response correctly returns an error
/// from the HTTP stage.
#[tokio::test]
async fn test_fetch_errors_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await;
    assert!(result.is_err());
}

/// Verifies that a page with no scoreable content (e.g., an empty body)
/// returns the expected `EmptyContent` error.
#[tokio::test]
async fn test_fetch_errors_on_empty_content() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html><body></body></html>"))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await;
    assert!(result.is_err());
}
