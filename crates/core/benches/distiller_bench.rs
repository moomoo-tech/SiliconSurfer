use criterion::{Criterion, black_box, criterion_group, criterion_main};

use agent_browser_core::distiller::Distiller;
use agent_browser_core::distiller_fast::{DistillMode, FastDistiller};

// Test HTML samples of varying sizes
fn small_html() -> &'static str {
    r#"<html><head><title>Test</title></head><body>
    <article><h1>Hello World</h1><p>This is a <a href="https://example.com">test page</a> with some content.</p>
    <ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul></article>
    <nav>Home | About | Contact</nav><footer>Copyright 2026</footer></body></html>"#
}

fn medium_html() -> String {
    // ~50KB of realistic HTML
    let mut html = String::from("<html><head><title>Medium Page</title></head><body><main>");
    for i in 0..200 {
        html.push_str(&format!(
            r#"<div class="item"><h2>Item {i}</h2>
            <p>Description for item {i} with <a href="https://example.com/{i}">a link</a> and <strong>bold text</strong>.</p>
            <pre><code>fn example_{i}() {{ println!("hello {i}"); }}</code></pre>
            <table><tr><th>Key</th><th>Value</th></tr><tr><td>price</td><td>${i}.99</td></tr></table></div>"#
        ));
    }
    html.push_str("</main><nav>Nav</nav><footer>Footer</footer></body></html>");
    html
}

fn large_html() -> String {
    // ~500KB of HTML
    let mut html = String::from("<html><head><title>Large Page</title></head><body><article>");
    for i in 0..2000 {
        html.push_str(&format!(
            r#"<section><h3>Section {i}</h3><p>Paragraph with <a href="/page/{i}">relative link</a> and
            <em>emphasis</em>. More text to simulate real content with various &#160; entities &#38; special chars.</p></section>"#
        ));
    }
    html.push_str(
        "</article><script>var x = 1;</script><style>.foo{color:red}</style></body></html>",
    );
    html
}

fn bench_scraper(c: &mut Criterion) {
    let d = Distiller::new();
    let small = small_html();
    let medium = medium_html();
    let large = large_html();

    let mut group = c.benchmark_group("scraper");

    group.bench_function("small_500B", |b| b.iter(|| d.to_markdown(black_box(small))));
    group.bench_function("medium_50KB", |b| {
        b.iter(|| d.to_markdown(black_box(&medium)))
    });
    group.bench_function("large_500KB", |b| {
        b.iter(|| d.to_markdown(black_box(&large)))
    });

    group.finish();
}

fn bench_lol_html(c: &mut Criterion) {
    let small = small_html();
    let medium = medium_html();
    let large = large_html();

    let mut group = c.benchmark_group("lol_html");

    group.bench_function("small_500B", |b| {
        b.iter(|| FastDistiller::to_markdown(black_box(small)))
    });
    group.bench_function("medium_50KB", |b| {
        b.iter(|| FastDistiller::to_markdown(black_box(&medium)))
    });
    group.bench_function("large_500KB", |b| {
        b.iter(|| FastDistiller::to_markdown(black_box(&large)))
    });

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    let d = Distiller::new();
    let medium = medium_html();

    let mut group = c.benchmark_group("comparison_50KB");

    group.bench_function("scraper", |b| b.iter(|| d.to_markdown(black_box(&medium))));
    group.bench_function("lol_html", |b| {
        b.iter(|| FastDistiller::to_markdown(black_box(&medium)))
    });

    group.finish();
}

fn bench_all_modes(c: &mut Criterion) {
    let medium = medium_html();

    let mut group = c.benchmark_group("modes_50KB");

    group.bench_function("reader", |b| {
        b.iter(|| {
            FastDistiller::distill(
                black_box(&medium),
                DistillMode::Reader,
                Some("https://example.com"),
            )
        })
    });
    group.bench_function("operator", |b| {
        b.iter(|| {
            FastDistiller::distill(
                black_box(&medium),
                DistillMode::Operator,
                Some("https://example.com"),
            )
        })
    });
    group.bench_function("spider", |b| {
        b.iter(|| {
            FastDistiller::distill(
                black_box(&medium),
                DistillMode::Spider,
                Some("https://example.com"),
            )
        })
    });
    group.bench_function("developer", |b| {
        b.iter(|| FastDistiller::distill(black_box(&medium), DistillMode::Developer, None))
    });
    group.bench_function("data", |b| {
        b.iter(|| FastDistiller::distill(black_box(&medium), DistillMode::Data, None))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scraper,
    bench_lol_html,
    bench_comparison,
    bench_all_modes
);
criterion_main!(benches);
