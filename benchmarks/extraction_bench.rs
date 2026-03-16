use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;
use web2llm::{Web2llm, Web2llmConfig};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_client() -> Web2llm {
    Web2llm::new(
        Web2llmConfig::new(
            "web2llm-benchmark".to_string(),
            Duration::from_secs(30),
            false,
            0.1,
            1000,
            100,
        )
        .with_robots_check(false),
    )
    .unwrap()
}

fn benchmark_extraction(c: &mut Criterion) {
    let html = std::fs::read_to_string("benchmarks/fixtures/wikipedia.html")
        .expect("missing benchmark fixture — run: curl https://en.wikipedia.org/wiki/Web_scraping -o benchmarks/fixtures/wikipedia.html");

    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&html))
            .mount(&server)
            .await;
        (server, test_client())
    });

    c.bench_function("benchmark extraction wikipedia", |b| {
        b.to_async(&rt)
            .iter(|| async { client.fetch(black_box(&server.uri())).await.unwrap() })
    });
}

fn benchmark_extraction_simple(c: &mut Criterion) {
    let html = r#"<html><body>
        <article>
            This is a simple page with just one article element and enough content to score well.
            It should be much faster than the Wikipedia page since the tree is tiny.
        </article>
    </body></html>"#
        .to_string();

    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&html))
            .mount(&server)
            .await;
        (server, test_client())
    });

    c.bench_function("benchmark extraction simple", |b| {
        b.to_async(&rt).iter(|| async {
            client
                .fetch(std::hint::black_box(&server.uri()))
                .await
                .unwrap()
        })
    });
}

fn benchmark_batch_wikipedia(c: &mut Criterion) {
    let html =
        std::fs::read_to_string("benchmarks/fixtures/wikipedia.html").expect("missing benchmark fixture");
    let rt = tokio::runtime::Runtime::new().unwrap();

    let (server, client) = rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&html))
            .mount(&server)
            .await;
        (server, test_client())
    });

    let urls: Vec<String> = (0..110).map(|_| server.uri()).collect();

    c.bench_function("benchmark batch fetch 110 wikipedia", |b| {
        b.to_async(&rt)
            .iter(|| async { client.batch_fetch(black_box(urls.clone())).await })
    });
}

criterion_group!(
    benchmarks,
    benchmark_extraction,
    benchmark_extraction_simple,
    benchmark_batch_wikipedia
);
criterion_main!(benchmarks);
