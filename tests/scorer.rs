use std::time::Duration;
use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    Web2llm::new(Web2llmConfig::new(
        "web2llm-test".to_string(),
        Duration::from_secs(5),
        false,
    ))
}

#[tokio::test]
async fn test_nav_scores_lower_than_article() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <nav>Home About Contact Blog Portfolio Links More Links Even More Links</nav>
                <article>
                    This is a long article with plenty of meaningful content and real sentences.
                    It talks about interesting topics and has enough words to pass the threshold.
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

#[tokio::test]
async fn test_short_elements_are_excluded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <p>Too short</p>
                <article>
                    This article has enough words to pass the minimum word count threshold
                    and should be included in the scored results returned by the scorer.
                </article>
            </body></html>
        "#,
        ))
        .mount(&server)
        .await;
    let result = test_client().fetch(&server.uri()).await.unwrap();
    assert!(result.markdown.contains("enough words"));
    assert!(!result.markdown.contains("Too short"));
}
