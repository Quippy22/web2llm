use web2llm::fetch::get_html;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_html_returns_body_on_200() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html><body>hello</body></html>"))
        .mount(&server)
        .await;

    let result = get_html(&server.uri()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "<html><body>hello</body></html>");
}

#[tokio::test]
async fn test_get_html_errors_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let result = get_html(&server.uri()).await;
    assert!(result.is_err());
}
