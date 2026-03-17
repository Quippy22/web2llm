use web2llm::{Web2llm, Web2llmConfig};

fn test_client() -> Web2llm {
    let config = Web2llmConfig {
        block_private_hosts: false,
        ..Default::default()
    };
    Web2llm::new(config).unwrap()
}

#[tokio::test]
async fn test_batch_fetch_returns_multiple_results() {
    let client = test_client();
    let urls = vec![
        "https://example.com".to_string(),
        "https://google.com".to_string(),
    ];
    let results = client.batch_fetch(urls).await;
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_batch_fetch_handles_mixed_results() {
    let client = test_client();
    let urls = vec![
        "https://example.com".to_string(),
        "https://invalid-url-that-should-fail.com".to_string(),
    ];
    let results = client.batch_fetch(urls).await;
    assert_eq!(results.len(), 2);

    // Find the success and failure
    let success = results.iter().find(|(url, _)| url == "https://example.com");
    let failure = results
        .iter()
        .find(|(url, _)| url == "https://invalid-url-that-should-fail.com");

    assert!(success.is_some() && success.unwrap().1.is_ok());
    assert!(failure.is_some() && failure.unwrap().1.is_err());
}
