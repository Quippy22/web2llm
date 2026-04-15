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

#[tokio::test]
async fn test_batch_fetch_returns_multiple_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>
                    This page has enough content to be scored and converted into markdown.
                </article>
            </body></html>
            "#,
        ))
        .mount(&server)
        .await;

    let client = test_client();
    let urls = vec![server.uri(), server.uri()];
    let results = client.batch_fetch(urls).await;
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|(_, result)| result.is_ok()));
}

#[tokio::test]
async fn test_batch_fetch_handles_mixed_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>
                    This page has enough content to be scored and converted into markdown.
                </article>
            </body></html>
            "#,
        ))
        .mount(&server)
        .await;

    let client = test_client();
    let urls = vec![server.uri(), "not-a-url".to_string()];
    let results = client.batch_fetch(urls).await;
    assert_eq!(results.len(), 2);

    // Find the success and failure
    let success = results.iter().find(|(url, _)| url == &server.uri());
    let failure = results.iter().find(|(url, _)| url == "not-a-url");

    assert!(success.is_some() && success.unwrap().1.is_ok());
    assert!(failure.is_some() && failure.unwrap().1.is_err());
}
