use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use web2llm::config::Web2llmConfig;
use web2llm::{FetchMode, Web2llm};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    let config = Web2llmConfig::new(
        "web2llm-benchmark".to_string(),
        Duration::from_secs(30),
        false,
        0.1,
        false,
        1000,
        100,
        FetchMode::Static,
    );
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

    c.bench_function("batch_fetch_wikipedia_10x", |b| {
        b.to_async(&rt).iter(|| async {
            let urls = vec![server.uri(); 10];
            client.batch_fetch(black_box(urls)).await
        })
    });
}

criterion_group!(
    benches,
    benchmark_extraction_wikipedia,
    benchmark_extraction_simple,
    benchmark_batch_wikipedia
);
criterion_main!(benches);
