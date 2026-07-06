//! Public web API — the two primitives a downstream consumer (tars/concer) links:
//!
//!   * [`fetch`]  — URL → clean Markdown/text + metadata, with fastest-first
//!     escalation (reqwest static → Chromium only when the static body is an
//!     unrendered JS shell) hidden *inside*. The caller never picks the tier.
//!   * [`search`] — query → `Vec<SearchResult>`, default backend scrapes
//!     DuckDuckGo's HTML endpoint through the same fetch/escalation/extract stack.
//!
//! Both surface a single typed [`WebError`] (incl. [`WebError::NoBrowser`]) and a
//! bounded transient-retry. The escalation router itself is [`crate::router::Engine`],
//! reused as-is; this module adds the clean public entry, typed errors, and retry.

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::browser::BrowserError;
use crate::distiller_fast::DistillMode;
use crate::fetcher::FetchError;
use crate::router::{is_sparse_content, Engine, EngineError, EngineResult};
use crate::{browser::BrowserPool, profiles};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Which tier actually served the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Served by the static reqwest path (no browser).
    Static,
    /// Served by the headless-Chromium path after escalation.
    Browser,
}

/// A fetched, distilled page plus provenance metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Final URL (as fetched).
    pub url: String,
    pub title: Option<String>,
    /// Clean Markdown (or text, per [`FetchOpts::output`]).
    pub content: String,
    pub content_length: usize,
    /// Which tier produced this result.
    pub tier: Tier,
}

impl Page {
    fn from_engine(r: EngineResult, tier: Tier) -> Self {
        Page {
            url: r.url,
            title: r.title,
            content: r.content,
            content_length: r.content_length,
            tier,
        }
    }
}

/// Options for [`fetch`]. `Default` = markdown, fast distiller, 3 attempts.
#[derive(Debug, Clone)]
pub struct FetchOpts {
    /// Output shape: `"markdown"` (default) or `"text"`.
    pub output: String,
    /// Use the fast lol_html streaming distiller.
    pub fast: bool,
    /// Distill mode (reader/operator/spider/…).
    pub distill: DistillMode,
    /// Max attempts per tier for transient failures (>= 1). 1 disables retry.
    pub max_attempts: u32,
    /// Base backoff between retries (scaled by attempt number).
    pub backoff: Duration,
}

impl Default for FetchOpts {
    fn default() -> Self {
        Self {
            output: "markdown".to_string(),
            fast: true,
            distill: DistillMode::default(),
            max_attempts: 3,
            backoff: Duration::from_millis(200),
        }
    }
}

// ---------------------------------------------------------------------------
// Typed error taxonomy
// ---------------------------------------------------------------------------

/// The single typed error for the public web API.
///
/// Callers branch on the variant — notably [`WebError::NoBrowser`], which means a
/// page needed Chromium but none could be launched, so the caller can fall back to
/// a static result or tell the user to install a browser.
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    /// URL failed to parse or isn't http/https. Permanent.
    #[error("invalid URL: {url}")]
    InvalidUrl { url: String },

    /// Server returned a 4xx client error. Permanent (retrying won't help).
    #[error("HTTP {status} for {url}")]
    Http { url: String, status: u16 },

    /// Static (reqwest) transport failure — carried typed, not stringified.
    #[error("static fetch failed: {0}")]
    Fetch(#[source] FetchError),

    /// Browser-tier failure other than "no browser" (navigation, crash, page error).
    #[error("browser fetch failed: {0}")]
    Browser(#[source] BrowserError),

    /// A page needed Chromium but none could be found or launched. Permanent.
    #[error("no usable Chromium-family browser: {hint}")]
    NoBrowser { hint: String },

    /// The search engine returned a page but it carried no results for the query.
    #[error("search returned no results for {query:?}")]
    EmptyResults { query: String },

    /// A search backend's HTML was fetched but no result rows could be parsed —
    /// e.g. the site changed its markup. Not a raw string dump: names the backend
    /// and the concrete reason.
    #[error("could not parse {backend} search results: {detail}")]
    SearchParse { backend: String, detail: String },

    /// A selected search backend is not implemented yet (e.g. the Brave API seam).
    #[error("search backend not available: {0}")]
    UnsupportedBackend(String),
}

impl WebError {
    /// Whether retrying could plausibly succeed. Only transport-level timeouts,
    /// connection resets, 5xx, and recoverable browser faults (crash/timeout) are
    /// transient; 4xx, invalid URL, NoBrowser, and parse/empty are permanent.
    pub fn is_transient(&self) -> bool {
        match self {
            WebError::Fetch(e) => e.is_transient(),
            WebError::Browser(e) => e.is_transient(),
            _ => false,
        }
    }

    /// Convenience inverse of [`WebError::is_transient`].
    pub fn is_permanent(&self) -> bool {
        !self.is_transient()
    }
}

/// Map an [`EngineError`] into the public taxonomy, lifting `NoBrowser` and 4xx
/// out into their own dedicated variants.
fn map_engine_err(url: &str, e: EngineError) -> WebError {
    match e {
        EngineError::T0(FetchError::Status(status)) if (400..500).contains(&status) => {
            WebError::Http {
                url: url.to_string(),
                status,
            }
        }
        EngineError::T0(fe) => WebError::Fetch(fe),
        EngineError::T1(be) => map_browser_err(be),
    }
}

fn map_browser_err(be: BrowserError) -> WebError {
    match be {
        BrowserError::NoBrowser { hint } => WebError::NoBrowser { hint },
        other => WebError::Browser(other),
    }
}

// ---------------------------------------------------------------------------
// Bounded transient retry
// ---------------------------------------------------------------------------

/// Run `op` up to `max_attempts` times, retrying only while `is_transient` holds.
/// Backoff scales linearly with the attempt number; `Duration::ZERO` disables the
/// sleep (used in tests). The final attempt's result is returned as-is.
pub(crate) async fn with_retry<T, E, F, Fut>(
    max_attempts: u32,
    backoff: Duration,
    is_transient: impl Fn(&E) -> bool,
    mut op: F,
) -> Result<T, E>
where
    F: FnMut(u32) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let attempts = max_attempts.max(1);
    // All but the last attempt may retry on a transient error.
    for attempt in 1..attempts {
        match op(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) if is_transient(&e) => {
                if !backoff.is_zero() {
                    tokio::time::sleep(backoff.saturating_mul(attempt)).await;
                }
            }
            Err(e) => return Err(e),
        }
    }
    // Final attempt: whatever it yields is the answer.
    op(attempts).await
}

// ---------------------------------------------------------------------------
// fetch()
// ---------------------------------------------------------------------------

static SHARED_ENGINE: OnceLock<Engine> = OnceLock::new();

fn shared_engine() -> &'static Engine {
    SHARED_ENGINE.get_or_init(Engine::new)
}

/// Fetch a URL → clean [`Page`], escalating reqwest → Chromium internally.
///
/// Static pages resolve via reqwest with no browser; only pages flagged
/// [`profiles::requires_t1`] or whose static body is an unrendered JS shell escalate
/// to Chromium. Transient failures retry with backoff. The caller does NOT choose
/// the tier.
pub async fn fetch(url: &str, opts: FetchOpts) -> Result<Page, WebError> {
    fetch_via(shared_engine(), url, &opts).await
}

impl Engine {
    /// Public clean fetch entry on an explicit engine (see the free [`fetch`]).
    pub async fn fetch_page(&self, url: &str, opts: &FetchOpts) -> Result<Page, WebError> {
        fetch_via(self, url, opts).await
    }
}

async fn fetch_via(engine: &Engine, url: &str, opts: &FetchOpts) -> Result<Page, WebError> {
    validate_url(url)?;

    // Profile says this domain always needs JS → go straight to the browser tier.
    if profiles::requires_t1(url) {
        return fetch_browser(engine, url, opts).await;
    }

    // Fastest-first: try the static path.
    match fetch_static(engine, url, opts).await {
        Ok(page) if !is_sparse_content(&page.content) => Ok(page),
        Ok(sparse) => {
            // Static returned an unrendered shell — escalate. If the browser is
            // unavailable or fails, fall back to the (sparse) static result.
            tracing::info!("static body sparse for {url}; escalating to browser");
            match fetch_browser(engine, url, opts).await {
                Ok(page) => Ok(page),
                Err(_) => Ok(sparse),
            }
        }
        // A permanent static error (4xx, invalid) — don't waste a browser launch.
        Err(e) if e.is_permanent() => Err(e),
        // A transient static failure that exhausted its retries — try the browser.
        Err(_) => fetch_browser(engine, url, opts).await,
    }
}

async fn fetch_static(engine: &Engine, url: &str, opts: &FetchOpts) -> Result<Page, WebError> {
    with_retry(
        opts.max_attempts,
        opts.backoff,
        WebError::is_transient,
        move |_attempt| async move {
            engine
                .fetch_t0(url, &opts.output, opts.fast, opts.distill)
                .await
                .map(|r| Page::from_engine(r, Tier::Static))
                .map_err(|e| map_engine_err(url, e))
        },
    )
    .await
}

async fn fetch_browser(engine: &Engine, url: &str, opts: &FetchOpts) -> Result<Page, WebError> {
    let attempts = opts.max_attempts.max(1);
    let pool = engine.browser_pool();

    // Retryable attempts, with a crash-triggered restart in between.
    for attempt in 1..attempts {
        match engine.fetch_t1(url, &opts.output).await {
            Ok(r) => return Ok(Page::from_engine(r, Tier::Browser)),
            Err(EngineError::T1(be)) if be.is_transient() => {
                if be.needs_restart() {
                    // Process crashed — relaunch before retrying. If it can no longer
                    // launch, surface that typed NoBrowser immediately.
                    tracing::warn!("browser crashed on {url}; restarting");
                    pool.restart().await.map_err(map_browser_err)?;
                }
                if !opts.backoff.is_zero() {
                    tokio::time::sleep(opts.backoff.saturating_mul(attempt)).await;
                }
            }
            Err(e) => return Err(map_engine_err(url, e)),
        }
    }

    // Final attempt.
    engine
        .fetch_t1(url, &opts.output)
        .await
        .map(|r| Page::from_engine(r, Tier::Browser))
        .map_err(|e| map_engine_err(url, e))
}

fn validate_url(url: &str) -> Result<(), WebError> {
    match reqwest::Url::parse(url) {
        Ok(u) if matches!(u.scheme(), "http" | "https") => Ok(()),
        _ => Err(WebError::InvalidUrl {
            url: url.to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// search()
// ---------------------------------------------------------------------------

/// One search hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Pluggable search backend.
///
/// `DdgScrape` is the default (no API key) and is the only one implemented now.
/// `BraveApi` is a seam for phase 2: a config-provided key hits the Brave Search
/// API for clean JSON — a consumer switches to it without any API change here.
#[derive(Debug, Clone, Default)]
pub enum SearchBackend {
    /// Scrape DuckDuckGo's HTML endpoint. Static + scrape-tolerant → resolves via
    /// the fast reqwest path, no Chromium.
    #[default]
    DdgScrape,
    /// TODO(phase-2): Brave Search API. Not implemented yet.
    BraveApi { api_key: String },
}

/// Options for [`search`].
#[derive(Debug, Clone, Default)]
pub struct SearchOpts {
    pub backend: SearchBackend,
    /// Retry/backoff knobs, reused from the fetch path.
    pub fetch: FetchOpts,
}

/// Search the web → `Vec<SearchResult>`.
///
/// Default backend scrapes DuckDuckGo's HTML endpoint through the same
/// fetch/escalation/extract machinery as [`fetch`].
pub async fn search(query: &str, opts: SearchOpts) -> Result<Vec<SearchResult>, WebError> {
    search_via(shared_engine(), query, &opts).await
}

async fn search_via(
    engine: &Engine,
    query: &str,
    opts: &SearchOpts,
) -> Result<Vec<SearchResult>, WebError> {
    match &opts.backend {
        SearchBackend::DdgScrape => ddg_search(engine, query, opts).await,
        SearchBackend::BraveApi { .. } => Err(WebError::UnsupportedBackend(
            "BraveApi backend is a phase-2 seam and not implemented yet".to_string(),
        )),
    }
}

async fn ddg_search(
    engine: &Engine,
    query: &str,
    opts: &SearchOpts,
) -> Result<Vec<SearchResult>, WebError> {
    if query.trim().is_empty() {
        return Err(WebError::EmptyResults {
            query: query.to_string(),
        });
    }
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencode(query));

    let html = with_retry(
        opts.fetch.max_attempts,
        opts.fetch.backoff,
        WebError::is_transient,
        move |_attempt| {
            let url = url.clone();
            async move { engine.fetch_raw(&url).await.map_err(|e| map_engine_err(&url, e)) }
        },
    )
    .await?;

    let results = parse_ddg_html(&html);
    if !results.is_empty() {
        return Ok(results);
    }

    // No rows parsed: distinguish "engine reports zero hits" from "markup changed".
    let lower = html.to_lowercase();
    if lower.contains("no results") || lower.contains("no-results") {
        Err(WebError::EmptyResults {
            query: query.to_string(),
        })
    } else {
        Err(WebError::SearchParse {
            backend: "ddg".to_string(),
            detail: "no result rows matched the expected selectors (DDG HTML may have changed)"
                .to_string(),
        })
    }
}

/// Parse DuckDuckGo HTML-endpoint markup into result rows.
fn parse_ddg_html(html: &str) -> Vec<SearchResult> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let (row_sel, a_sel, snip_sel) = match (
        Selector::parse("div.result"),
        Selector::parse("a.result__a"),
        Selector::parse(".result__snippet"),
    ) {
        (Ok(r), Ok(a), Ok(s)) => (r, a, s),
        _ => return Vec::new(),
    };

    let mut out = Vec::new();
    for row in doc.select(&row_sel) {
        let Some(anchor) = row.select(&a_sel).next() else {
            continue;
        };
        let title = anchor.text().collect::<String>().trim().to_string();
        let href = anchor.value().attr("href").unwrap_or_default();
        let url = decode_ddg_href(href);
        if title.is_empty() || url.is_empty() {
            continue;
        }
        let snippet = row
            .select(&snip_sel)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        out.push(SearchResult {
            title,
            url,
            snippet,
        });
    }
    out
}

/// DDG wraps targets as `//duckduckgo.com/l/?uddg=<pct-encoded>&rut=…`; unwrap and
/// decode. Some rows already carry a direct URL.
fn decode_ddg_href(href: &str) -> String {
    if let Some(idx) = href.find("uddg=") {
        let rest = &href[idx + "uddg=".len()..];
        let enc = rest.split('&').next().unwrap_or(rest);
        return percent_decode(enc);
    }
    if href.starts_with("http") {
        return href.to_string();
    }
    if let Some(stripped) = href.strip_prefix("//") {
        return format!("https://{stripped}");
    }
    href.to_string()
}

/// Minimal `application/x-www-form-urlencoded`-style decoder (`%XX` and `+`),
/// avoiding a new crate dependency.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => match (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                (Some(h), Some(l)) => {
                    out.push(h * 16 + l);
                    i += 3;
                }
                _ => {
                    out.push(bytes[i]);
                    i += 1;
                }
            },
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Percent-encode a query for the `?q=` parameter (spaces → `+`).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Convenience: an engine pinned to a browser executable (e.g. from `CHROME_PATH`
/// config), for callers that don't want the process-wide shared engine.
pub fn engine_with_browser(executable: impl Into<std::path::PathBuf>) -> Engine {
    Engine::with_browser_pool(BrowserPool::with_executable(executable))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Debug)]
    struct FakeErr {
        transient: bool,
    }

    // --- transient retry ---------------------------------------------------

    #[tokio::test]
    async fn with_retry_retries_transient_then_succeeds() {
        let calls = Cell::new(0u32);
        let res: Result<u32, FakeErr> = with_retry(
            3,
            Duration::ZERO,
            |e: &FakeErr| e.transient,
            |_| {
                calls.set(calls.get() + 1);
                let n = calls.get();
                async move {
                    if n < 3 {
                        Err(FakeErr { transient: true })
                    } else {
                        Ok(n)
                    }
                }
            },
        )
        .await;
        assert_eq!(res.unwrap(), 3);
        assert_eq!(calls.get(), 3, "should have retried exactly to the 3rd attempt");
    }

    #[tokio::test]
    async fn with_retry_does_not_retry_permanent() {
        let calls = Cell::new(0u32);
        let res: Result<u32, FakeErr> = with_retry(
            3,
            Duration::ZERO,
            |e: &FakeErr| e.transient,
            |_| {
                calls.set(calls.get() + 1);
                async { Err(FakeErr { transient: false }) }
            },
        )
        .await;
        assert!(res.is_err());
        assert_eq!(calls.get(), 1, "permanent error must not retry");
    }

    // --- static path uses reqwest, no browser ------------------------------

    /// Serve one HTTP/1.1 response with `body` from a throwaway localhost socket.
    async fn serve_once(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = sock.read(&mut buf).await;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.flush().await;
            }
        });
        format!("http://{addr}/")
    }

    #[tokio::test]
    async fn static_url_uses_reqwest_path() {
        let body = "<html><head><title>Hi</title></head><body><article>\
            <h1>Real Article</h1>\
            <p>This is a genuine article body with plenty of readable prose so the \
            distiller keeps it and the router does not consider it a sparse shell. \
            It talks about pricing, context length, and other real content.</p>\
            <p>A second paragraph with even more substantive text to be safe.</p>\
            </article></body></html>";
        let url = serve_once(body).await;

        // Fresh engine so this test is independent of process-wide state.
        let engine = Engine::new();
        let opts = FetchOpts {
            max_attempts: 1,
            backoff: Duration::ZERO,
            ..Default::default()
        };
        let page = engine.fetch_page(&url, &opts).await.expect("fetch ok");
        assert_eq!(page.tier, Tier::Static, "static page must resolve via reqwest");
        assert!(page.content.contains("Real Article"));
    }

    // --- NoBrowser: forced-unavailable browser -----------------------------

    #[tokio::test]
    async fn missing_browser_start_is_typed_no_browser() {
        let pool = BrowserPool::with_executable("/nonexistent/definitely-not-a-browser");
        let err = pool.start().await.expect_err("must fail: no such executable");
        assert!(
            matches!(err, BrowserError::NoBrowser { .. }),
            "expected typed NoBrowser, got {err:?}"
        );
    }

    #[tokio::test]
    async fn sparse_static_falls_back_when_browser_unavailable() {
        // A short JS-shell body → router judges it sparse and tries to escalate.
        let body = "<html><body>Loading...</body></html>";
        let url = serve_once(body).await;

        // Engine whose browser tier can never launch.
        let engine = Engine::with_browser_pool(BrowserPool::with_executable(
            "/nonexistent/definitely-not-a-browser",
        ));
        let opts = FetchOpts {
            max_attempts: 1,
            backoff: Duration::ZERO,
            ..Default::default()
        };
        // Escalation hits NoBrowser → falls back to the static (sparse) result,
        // typed and non-panicking.
        let page = engine.fetch_page(&url, &opts).await.expect("falls back to static");
        assert_eq!(page.tier, Tier::Static);
    }

    #[test]
    fn no_browser_error_maps_to_web_no_browser() {
        let mapped = map_browser_err(BrowserError::NoBrowser {
            hint: "none".to_string(),
        });
        assert!(matches!(mapped, WebError::NoBrowser { .. }));
        assert!(mapped.is_permanent());
    }

    // --- search: DDG fixture parses into results ---------------------------

    const DDG_FIXTURE: &str = r#"
    <html><body>
      <div class="result results_links results_links_deep web-result">
        <div class="links_main">
          <h2 class="result__title">
            <a rel="nofollow" class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fone&amp;rut=abc">First Result</a>
          </h2>
          <a class="result__snippet" href="//x">The first snippet text.</a>
        </div>
      </div>
      <div class="result results_links results_links_deep web-result">
        <div class="links_main">
          <h2 class="result__title">
            <a rel="nofollow" class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Frust-lang.org%2Fdocs&amp;rut=def">Second Result</a>
          </h2>
          <a class="result__snippet">Second snippet.</a>
        </div>
      </div>
    </body></html>
    "#;

    #[test]
    fn ddg_fixture_parses_into_results() {
        let results = parse_ddg_html(DDG_FIXTURE);
        assert_eq!(results.len(), 2, "should parse both rows");
        assert_eq!(results[0].title, "First Result");
        assert_eq!(results[0].url, "https://example.com/one");
        assert_eq!(results[0].snippet, "The first snippet text.");
        assert_eq!(results[1].title, "Second Result");
        assert_eq!(results[1].url, "https://rust-lang.org/docs");
    }

    #[test]
    fn percent_decode_handles_encoded_url() {
        assert_eq!(
            percent_decode("https%3A%2F%2Fa.com%2Fb+c"),
            "https://a.com/b c"
        );
    }

    #[tokio::test]
    async fn brave_backend_is_typed_unsupported() {
        let opts = SearchOpts {
            backend: SearchBackend::BraveApi {
                api_key: "k".to_string(),
            },
            ..Default::default()
        };
        let err = search("anything", opts).await.expect_err("brave not implemented");
        assert!(matches!(err, WebError::UnsupportedBackend(_)));
    }

    #[tokio::test]
    async fn empty_query_is_typed_empty_results() {
        let err = search("   ", SearchOpts::default())
            .await
            .expect_err("empty query");
        assert!(matches!(err, WebError::EmptyResults { .. }));
    }

    #[test]
    fn invalid_url_is_typed() {
        assert!(matches!(
            validate_url("not a url"),
            Err(WebError::InvalidUrl { .. })
        ));
        assert!(matches!(
            validate_url("ftp://example.com"),
            Err(WebError::InvalidUrl { .. })
        ));
        assert!(validate_url("https://example.com").is_ok());
    }
}
