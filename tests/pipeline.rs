use std::time::Duration;
use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    Web2llm::new(Web2llmConfig::new(
        "web2llm-test".to_string(),
        Duration::from_secs(5),
    ))
}

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
