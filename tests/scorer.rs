use web2llm::extract::PageElements;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

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

    let page = PageElements::parse(&server.uri()).await.unwrap();
    let md = page.to_markdown().unwrap();
    assert!(md.contains("meaningful content"));
    assert!(!md.contains("Home About Contact"));
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

    let page = PageElements::parse(&server.uri()).await.unwrap();
    let md = page.to_markdown().unwrap();
    assert!(md.contains("enough words"));
    assert!(!md.contains("Too short"));
}
