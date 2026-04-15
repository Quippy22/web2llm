use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use web2llm::config::Web2llmConfig;
use web2llm::{CrawlConfig, FetchMode, Web2llm};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let config = Web2llmConfig {
        user_agent: "web2llm-benchmark".to_string(),
        timeout: Duration::from_secs(30),
        block_private_hosts: false,
        robots_check: false,
        rate_limit: 1000,
        max_concurrency: 100,
        fetch_mode: FetchMode::Static,
        ..Default::default()
    };
    Web2llm::new(config).unwrap()
}

fn benchmark_extraction_wikipedia(c: &mut Criterion) {
    let html = std::fs::read_to_string("benchmarks/fixtures/wikipedia.html")
        .expect("missing benchmark fixture");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        let client = test_client();
        (server, client)
    });

    c.bench_function("extract_wikipedia", |b| {
        b.to_async(&rt)
            .iter(|| async { client.fetch(black_box(&server.uri())).await.unwrap() })
    });
}

fn benchmark_extraction_simple(c: &mut Criterion) {
    let html = "<html><body><article>Just some simple content for a fast benchmark.</article></body></html>";
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        let client = test_client();
        (server, client)
    });

    c.bench_function("extract_simple", |b| {
        b.to_async(&rt)
            .iter(|| async { client.fetch(black_box(&server.uri())).await.unwrap() })
    });
}

fn benchmark_batch_wikipedia(c: &mut Criterion) {
    let html = std::fs::read_to_string("benchmarks/fixtures/wikipedia.html")
        .expect("missing benchmark fixture");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(html))
            .mount(&server)
            .await;

        let client = test_client();
        (server, client)
    });

    c.bench_function("batch_fetch_wikipedia_100x", |b| {
        b.to_async(&rt).iter(|| async {
            let urls = vec![server.uri(); 100];
            client.batch_fetch(black_box(urls)).await
        })
    });
}

fn benchmark_crawl_depth_one(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;

        Mock::given(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                r#"
                <html><body>
                    <article>
                        Seed content with enough text to score properly.
                        <a href="{0}/one">One</a>
                        <a href="{0}/two">Two</a>
                    </article>
                </body></html>
                "#,
                server.uri()
            )))
            .mount(&server)
            .await;

        Mock::given(path("/one"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"
                <html><body>
                    <article>First child content with enough text to score properly.</article>
                </body></html>
                "#,
            ))
            .mount(&server)
            .await;

        Mock::given(path("/two"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"
                <html><body>
                    <article>Second child content with enough text to score properly.</article>
                </body></html>
                "#,
            ))
            .mount(&server)
            .await;

        let client = test_client();
        (server, client)
    });

    c.bench_function("crawl_depth_one", |b| {
        b.to_async(&rt).iter(|| async {
            client
                .crawl(
                    black_box(&server.uri()),
                    CrawlConfig {
                        max_depth: 1,
                        preserve_domain: true,
                    },
                )
                .await
        })
    });
}

criterion_group!(
    benches,
    benchmark_extraction_wikipedia,
    benchmark_extraction_simple,
    benchmark_batch_wikipedia,
    benchmark_crawl_depth_one
);
criterion_main!(benches);
