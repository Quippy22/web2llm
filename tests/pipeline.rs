use web2llm::extract::PageElements;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_parse_returns_markdown_on_200() {
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

    let page = PageElements::parse(&server.uri()).await.unwrap();
    let md = page.into_result().unwrap().markdown;
    assert!(md.contains("main content"));
}

#[tokio::test]
async fn test_parse_errors_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let result = PageElements::parse(&server.uri()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_errors_on_empty_content() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html><body></body></html>"))
        .mount(&server)
        .await;

    let page = PageElements::parse(&server.uri()).await.unwrap();
    let result = page.into_result();
    assert!(result.is_err());
}
