use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let mut config = Web2llmConfig::default();
    config.block_private_hosts = false;
    Web2llm::new(config).unwrap()
}

/// Verifies that multiple URLs can be fetched concurrently.
#[tokio::test]
async fn test_batch_fetch_returns_multiple_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<html><body><article>Content for all</article></body></html>"#),
        )
        .mount(&server)
        .await;

    let urls = vec![server.uri(), server.uri(), server.uri()];
    let results = test_client().batch_fetch(&urls).await;

    assert_eq!(results.len(), 3);
    for (_url, result) in results {
        assert!(result.is_ok());
        assert!(result.unwrap().markdown.contains("Content for all"));
    }
}

/// Verifies that batch_fetch handles individual failures correctly
/// without failing the entire batch.
#[tokio::test]
async fn test_batch_fetch_handles_mixed_results() {
    let server = MockServer::start().await;

    // First URL: Success
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"<html><body><article>Success content</article></body></html>"#),
        )
        .mount(&server)
        .await;

    let urls = vec![
        server.uri(),
        "https://invalid-url-that-fails.com".to_string(),
    ];
    let results = test_client().batch_fetch(&urls).await;

    assert_eq!(results.len(), 2);

    // Check success
    let success = results.iter().find(|(u, _)| u == &server.uri()).unwrap();
    assert!(success.1.is_ok());

    // Check failure
    let failure = results
        .iter()
        .find(|(u, _)| u.contains("invalid-url"))
        .unwrap();
    assert!(failure.1.is_err());
}
