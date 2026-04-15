use web2llm::{CrawlConfig, Web2llm, Web2llmConfig};
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let config = Web2llmConfig {
        block_private_hosts: false,
        robots_check: false,
        ..Default::default()
    };
    Web2llm::new(config).unwrap()
}

#[tokio::test]
async fn test_crawl_respects_depth() {
    let server = MockServer::start().await;

    Mock::given(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"
            <html><body>
                <article>
                    Seed content with enough text to score properly.
                    <a href="{0}/one">One</a>
                </article>
            </body></html>
            "#,
            server.uri()
        )))
        .mount(&server)
        .await;

    Mock::given(path("/one"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"
            <html><body>
                <article>
                    Child content with enough text to score properly.
                    <a href="{0}/two">Two</a>
                </article>
            </body></html>
            "#,
            server.uri()
        )))
        .mount(&server)
        .await;

    Mock::given(path("/two"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>Grandchild content with enough text to score properly.</article>
            </body></html>
            "#,
        ))
        .mount(&server)
        .await;

    let results = test_client()
        .crawl(
            &server.uri(),
            CrawlConfig {
                max_depth: 1,
                ..Default::default()
            },
        )
        .await;

    let seed_url = format!("{}/", server.uri());
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .any(|(url, result)| url == &seed_url && result.is_ok())
    );
    assert!(
        results
            .iter()
            .any(|(url, result)| url == &format!("{}/one", server.uri()) && result.is_ok())
    );
    assert!(
        !results
            .iter()
            .any(|(url, _)| url == &format!("{}/two", server.uri()))
    );
}

#[tokio::test]
async fn test_crawl_preserves_domain_by_default() {
    let server = MockServer::start().await;
    let external = MockServer::start().await;

    Mock::given(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"
            <html><body>
                <article>
                    Seed content with enough text to score properly.
                    <a href="{0}/local">Local</a>
                    <a href="{1}/remote">Remote</a>
                </article>
            </body></html>
            "#,
            server.uri(),
            external.uri()
        )))
        .mount(&server)
        .await;

    Mock::given(path("/local"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>Local content with enough text to score properly.</article>
            </body></html>
            "#,
        ))
        .mount(&server)
        .await;

    let results = test_client()
        .crawl(
            &server.uri(),
            CrawlConfig {
                max_depth: 1,
                ..Default::default()
            },
        )
        .await;

    let seed_url = format!("{}/", server.uri());
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .any(|(url, result)| url == &seed_url && result.is_ok())
    );
    assert!(
        results
            .iter()
            .any(|(url, result)| url == &format!("{}/local", server.uri()) && result.is_ok())
    );
    assert!(
        !results
            .iter()
            .any(|(url, _)| url == &format!("{}/remote", external.uri()))
    );
}

#[tokio::test]
async fn test_crawl_can_cross_domains_when_disabled() {
    let server = MockServer::start().await;
    let external = MockServer::start().await;

    Mock::given(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            r#"
            <html><body>
                <article>
                    Seed content with enough text to score properly.
                    <a href="{0}/local">Local</a>
                    <a href="{1}/remote">Remote</a>
                </article>
            </body></html>
            "#,
            server.uri(),
            external.uri()
        )))
        .mount(&server)
        .await;

    Mock::given(path("/local"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>Local content with enough text to score properly.</article>
            </body></html>
            "#,
        ))
        .mount(&server)
        .await;

    Mock::given(path("/remote"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
            <html><body>
                <article>Remote content with enough text to score properly.</article>
            </body></html>
            "#,
        ))
        .mount(&external)
        .await;

    let results = test_client()
        .crawl(
            &server.uri(),
            CrawlConfig {
                max_depth: 1,
                preserve_domain: false,
            },
        )
        .await;

    assert_eq!(results.len(), 3);
    assert!(
        results
            .iter()
            .any(|(url, result)| url == &format!("{}/local", server.uri()) && result.is_ok())
    );
    assert!(
        results
            .iter()
            .any(|(url, result)| url == &format!("{}/remote", external.uri()) && result.is_ok())
    );
}
