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

    /// A configured search backend is missing its API key. Permanent — the
    /// consumer must inject a key into the `SearchConfig`/`SearchOpts`.
    #[error("search backend {0} requires an API key, but none was provided")]
    MissingApiKey(String),

    /// A configured search backend is otherwise misconfigured (e.g. Google CSE
    /// without a `cx` engine id). Permanent.
    #[error("search backend {backend} misconfigured: {detail}")]
    BackendConfig { backend: String, detail: String },

    /// A selected search backend is not implemented yet (e.g. the Baidu scrape seam).
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

/// The HTTP request one search dialect wants executed — the "build-request" half
/// of the dialect seam (mirrors tars' `CliDialect::invocation`).
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// Fully-formed request URL (query + any API key/params already encoded in).
    pub url: String,
    /// Extra request headers (e.g. `X-Subscription-Token`, `Accept: application/json`).
    pub headers: Vec<(String, String)>,
    /// Whether the executor may escalate to the browser when the static body looks
    /// like an unrendered shell. API dialects set `false`; HTML-scrape dialects `true`.
    pub allow_browser: bool,
}

/// Per-search-engine behavior seam — "each search engine is a dialect."
///
/// Mirrors tars' `CliDialect`: a common trait with a **build-request** step
/// ([`SearchDialect::build_request`]) and a **parse-response** step
/// ([`SearchDialect::parse_response`]); the shared executor drives any dialect over
/// [`Engine::fetch_search`]. Adding an engine = one small impl.
pub trait SearchDialect: Send + Sync {
    /// Name for diagnostics / typed errors.
    fn name(&self) -> &'static str;

    /// Assemble the HTTP request for `query`. Returns a typed error when the
    /// dialect can't be honored (e.g. a missing API key).
    fn build_request(&self, query: &str, opts: &SearchOpts) -> Result<SearchRequest, WebError>;

    /// Map the raw response body → results. `Ok(vec![])` means the engine
    /// legitimately returned zero hits; `Err(SearchParse)` means the payload was
    /// present but unreadable (markup/schema changed) — carrying context, never a
    /// bare token.
    fn parse_response(&self, raw: &str) -> Result<Vec<SearchResult>, WebError>;
}

// --- Google Custom Search JSON API (recommended) ---------------------------

/// Google Programmable Search / Custom Search JSON API. 100 queries/day free.
#[derive(Debug, Clone)]
pub struct GoogleCseDialect {
    pub api_key: String,
    /// Programmable Search Engine id (`cx`).
    pub cx: String,
}

impl SearchDialect for GoogleCseDialect {
    fn name(&self) -> &'static str {
        "google_cse"
    }

    fn build_request(&self, query: &str, _opts: &SearchOpts) -> Result<SearchRequest, WebError> {
        if self.api_key.trim().is_empty() {
            return Err(WebError::MissingApiKey(self.name().to_string()));
        }
        if self.cx.trim().is_empty() {
            return Err(WebError::BackendConfig {
                backend: self.name().to_string(),
                detail: "missing `cx` programmable-search-engine id".to_string(),
            });
        }
        let url = format!(
            "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}",
            urlencode(&self.api_key),
            urlencode(&self.cx),
            urlencode(query),
        );
        Ok(SearchRequest {
            url,
            headers: vec![("Accept".to_string(), "application/json".to_string())],
            allow_browser: false,
        })
    }

    fn parse_response(&self, raw: &str) -> Result<Vec<SearchResult>, WebError> {
        let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| WebError::SearchParse {
            backend: self.name().to_string(),
            detail: format!("invalid JSON: {e}"),
        })?;
        // A structured API error (bad key / quota) — surface its message typed.
        if let Some(msg) = v
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            return Err(WebError::BackendConfig {
                backend: self.name().to_string(),
                detail: msg.to_string(),
            });
        }
        let Some(items) = v.get("items").and_then(|i| i.as_array()) else {
            return Ok(Vec::new()); // valid response, zero results
        };
        Ok(items
            .iter()
            .filter_map(|it| {
                let title = it.get("title")?.as_str()?.to_string();
                let url = it.get("link")?.as_str()?.to_string();
                let snippet = it
                    .get("snippet")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string();
                Some(SearchResult {
                    title,
                    url,
                    snippet,
                })
            })
            .collect())
    }
}

// --- Brave Search API ------------------------------------------------------

/// Brave Search API (`/res/v1/web/search`), key via the `X-Subscription-Token` header.
#[derive(Debug, Clone)]
pub struct BraveApiDialect {
    pub api_key: String,
}

impl SearchDialect for BraveApiDialect {
    fn name(&self) -> &'static str {
        "brave"
    }

    fn build_request(&self, query: &str, _opts: &SearchOpts) -> Result<SearchRequest, WebError> {
        if self.api_key.trim().is_empty() {
            return Err(WebError::MissingApiKey(self.name().to_string()));
        }
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}",
            urlencode(query)
        );
        Ok(SearchRequest {
            url,
            headers: vec![
                ("Accept".to_string(), "application/json".to_string()),
                ("Accept-Encoding".to_string(), "gzip".to_string()),
                ("X-Subscription-Token".to_string(), self.api_key.clone()),
            ],
            allow_browser: false,
        })
    }

    fn parse_response(&self, raw: &str) -> Result<Vec<SearchResult>, WebError> {
        let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| WebError::SearchParse {
            backend: self.name().to_string(),
            detail: format!("invalid JSON: {e}"),
        })?;
        let Some(results) = v
            .get("web")
            .and_then(|w| w.get("results"))
            .and_then(|r| r.as_array())
        else {
            return Ok(Vec::new());
        };
        Ok(results
            .iter()
            .filter_map(|it| {
                let title = it.get("title")?.as_str()?.to_string();
                let url = it.get("url")?.as_str()?.to_string();
                let snippet = it
                    .get("description")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string();
                Some(SearchResult {
                    title,
                    url,
                    snippet,
                })
            })
            .collect())
    }
}

// --- DuckDuckGo HTML scrape (no key) ---------------------------------------

/// DuckDuckGo HTML endpoint scrape. No API key; resolves via the fast reqwest path.
#[derive(Debug, Clone, Default)]
pub struct DdgScrapeDialect;

impl SearchDialect for DdgScrapeDialect {
    fn name(&self) -> &'static str {
        "ddg"
    }

    fn build_request(&self, query: &str, _opts: &SearchOpts) -> Result<SearchRequest, WebError> {
        Ok(SearchRequest {
            url: format!("https://html.duckduckgo.com/html/?q={}", urlencode(query)),
            headers: Vec::new(),
            allow_browser: true,
        })
    }

    fn parse_response(&self, raw: &str) -> Result<Vec<SearchResult>, WebError> {
        let rows = parse_ddg_html(raw);
        if !rows.is_empty() {
            return Ok(rows);
        }
        // No rows: tell "engine says zero hits" from "markup changed".
        let lower = raw.to_lowercase();
        if lower.contains("no results") || lower.contains("no-results") {
            Ok(Vec::new())
        } else {
            Err(WebError::SearchParse {
                backend: self.name().to_string(),
                detail: "no result rows matched the expected selectors (DDG HTML may have changed)"
                    .to_string(),
            })
        }
    }
}

// TODO(phase-2): BaiduScrape — a Chinese-market HTML scrape dialect. Add a
// `BackendKind::Baidu` + a `BaiduScrapeDialect` impl here when ready.

// --- Backend selector (enum of configured dialects) ------------------------

/// A configured search backend: which dialect + its resolved parameters. Behavior
/// lives in the [`SearchDialect`] impls; this enum is the data the consumer selects
/// (usually via [`SearchConfig::build`]).
#[derive(Debug, Clone, Default)]
pub enum SearchBackend {
    /// Scrape DuckDuckGo's HTML endpoint (no key). Default.
    #[default]
    DdgScrape,
    /// Google Custom Search JSON API — recommended.
    GoogleCse { api_key: String, cx: String },
    /// Brave Search API.
    BraveApi { api_key: String },
}

impl SearchBackend {
    /// Construct the behavior object (dialect) for this backend.
    pub fn dialect(&self) -> Box<dyn SearchDialect> {
        match self {
            SearchBackend::DdgScrape => Box::new(DdgScrapeDialect),
            SearchBackend::GoogleCse { api_key, cx } => Box::new(GoogleCseDialect {
                api_key: api_key.clone(),
                cx: cx.clone(),
            }),
            SearchBackend::BraveApi { api_key } => Box::new(BraveApiDialect {
                api_key: api_key.clone(),
            }),
        }
    }
}

/// Options for [`search`].
#[derive(Debug, Clone, Default)]
pub struct SearchOpts {
    pub backend: SearchBackend,
    /// Retry/backoff knobs, reused from the fetch path.
    pub fetch: FetchOpts,
}

// --- SearchConfig: the serde schema the consumer (tars) deserializes -------

/// Serde schema for a `[web_search]` TOML section — **owned by sisurf**.
///
/// sisurf owns *which* backends exist and *what fields* each needs; the CONSUMER
/// (tars) deserializes the file into this, then injects the resolved API key into
/// the relevant sub-config (sisurf never reads env vars or files itself — it's a
/// library). Turn a key-injected config into a runnable [`SearchBackend`] with
/// [`SearchConfig::build`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Which backend to use.
    #[serde(default)]
    pub backend: BackendKind,
    /// Google CSE settings (required when `backend = "google_cse"`).
    #[serde(default)]
    pub google_cse: Option<GoogleCseConfig>,
    /// Brave settings (required when `backend = "brave"`).
    #[serde(default)]
    pub brave: Option<BraveConfig>,
}

/// Which search backend a [`SearchConfig`] selects.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    /// DuckDuckGo HTML scrape (no key). Default.
    #[default]
    Ddg,
    /// Google Custom Search JSON API.
    GoogleCse,
    /// Brave Search API.
    Brave,
    // TODO(phase-2): Baidu (Chinese HTML scrape).
}

/// Google CSE sub-config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoogleCseConfig {
    /// Programmable Search Engine id (`cx`) — committed config, not a secret.
    pub cx: String,
    /// Resolved API key — the CONSUMER injects this; sisurf never reads env.
    #[serde(default)]
    pub api_key: String,
}

/// Brave sub-config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BraveConfig {
    /// Resolved API key — the CONSUMER injects this.
    #[serde(default)]
    pub api_key: String,
}

impl SearchConfig {
    /// Build a runnable [`SearchBackend`] from the (key-injected) config.
    ///
    /// Fails typed when the selected backend's sub-config or key is missing, so the
    /// consumer surfaces a clear error instead of silently falling back.
    pub fn build(&self) -> Result<SearchBackend, WebError> {
        match self.backend {
            BackendKind::Ddg => Ok(SearchBackend::DdgScrape),
            BackendKind::GoogleCse => {
                let g = self
                    .google_cse
                    .as_ref()
                    .ok_or_else(|| WebError::BackendConfig {
                        backend: "google_cse".to_string(),
                        detail: "missing [web_search.google_cse] section".to_string(),
                    })?;
                if g.api_key.trim().is_empty() {
                    return Err(WebError::MissingApiKey("google_cse".to_string()));
                }
                if g.cx.trim().is_empty() {
                    return Err(WebError::BackendConfig {
                        backend: "google_cse".to_string(),
                        detail: "missing `cx` programmable-search-engine id".to_string(),
                    });
                }
                Ok(SearchBackend::GoogleCse {
                    api_key: g.api_key.clone(),
                    cx: g.cx.clone(),
                })
            }
            BackendKind::Brave => {
                let b = self.brave.as_ref().ok_or_else(|| WebError::BackendConfig {
                    backend: "brave".to_string(),
                    detail: "missing [web_search.brave] section".to_string(),
                })?;
                if b.api_key.trim().is_empty() {
                    return Err(WebError::MissingApiKey("brave".to_string()));
                }
                Ok(SearchBackend::BraveApi {
                    api_key: b.api_key.clone(),
                })
            }
        }
    }
}

/// Search the web → `Vec<SearchResult>` via the configured dialect.
///
/// The default backend scrapes DuckDuckGo's HTML endpoint; API backends
/// (Google CSE / Brave) go through the same executor + retry.
pub async fn search(query: &str, opts: SearchOpts) -> Result<Vec<SearchResult>, WebError> {
    run_dialect(shared_engine(), query, &opts).await
}

/// Shared executor: build the request from the dialect, fetch it (with transient
/// retry) via [`Engine::fetch_search`], then parse. Drives any [`SearchDialect`].
async fn run_dialect(
    engine: &Engine,
    query: &str,
    opts: &SearchOpts,
) -> Result<Vec<SearchResult>, WebError> {
    if query.trim().is_empty() {
        return Err(WebError::EmptyResults {
            query: query.to_string(),
        });
    }
    let dialect = opts.backend.dialect();
    let req = dialect.build_request(query, opts)?;

    let raw = with_retry(
        opts.fetch.max_attempts,
        opts.fetch.backoff,
        WebError::is_transient,
        move |_attempt| {
            let req = req.clone();
            async move {
                engine
                    .fetch_search(&req)
                    .await
                    .map_err(|e| map_engine_err(&req.url, e))
            }
        },
    )
    .await?;

    let results = dialect.parse_response(&raw)?;
    if results.is_empty() {
        return Err(WebError::EmptyResults {
            query: query.to_string(),
        });
    }
    Ok(results)
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

    // --- Google CSE dialect: build-request + parse-response ----------------

    const GOOGLE_CSE_FIXTURE: &str = r#"
    {
      "kind": "customsearch#search",
      "items": [
        { "title": "Rust Programming Language", "link": "https://www.rust-lang.org/", "snippet": "A language empowering everyone." },
        { "title": "Rust - Wikipedia", "link": "https://en.wikipedia.org/wiki/Rust", "snippet": "Rust is a systems language." }
      ]
    }"#;

    #[test]
    fn google_cse_fixture_parses_into_results() {
        let d = GoogleCseDialect {
            api_key: "k".to_string(),
            cx: "cx1".to_string(),
        };
        let results = d.parse_response(GOOGLE_CSE_FIXTURE).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(results[1].snippet, "Rust is a systems language.");
    }

    #[test]
    fn google_cse_build_request_encodes_key_cx_query() {
        let d = GoogleCseDialect {
            api_key: "KEY".to_string(),
            cx: "CX".to_string(),
        };
        let req = d.build_request("hello world", &SearchOpts::default()).unwrap();
        assert!(req.url.contains("key=KEY"));
        assert!(req.url.contains("cx=CX"));
        assert!(req.url.contains("q=hello+world"));
        assert!(!req.allow_browser, "API dialect must not escalate to browser");
    }

    #[test]
    fn google_cse_missing_key_is_typed() {
        let d = GoogleCseDialect {
            api_key: "  ".to_string(),
            cx: "cx".to_string(),
        };
        assert!(matches!(
            d.build_request("q", &SearchOpts::default()),
            Err(WebError::MissingApiKey(_))
        ));
    }

    // --- Brave dialect -----------------------------------------------------

    const BRAVE_FIXTURE: &str = r#"
    {
      "web": {
        "results": [
          { "title": "Example Domain", "url": "https://example.com/", "description": "Illustrative example." }
        ]
      }
    }"#;

    #[test]
    fn brave_fixture_parses_and_sets_auth_header() {
        let d = BraveApiDialect {
            api_key: "secret".to_string(),
        };
        let req = d.build_request("q", &SearchOpts::default()).unwrap();
        assert!(req
            .headers
            .iter()
            .any(|(k, v)| k == "X-Subscription-Token" && v == "secret"));
        let results = d.parse_response(BRAVE_FIXTURE).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com/");
        assert_eq!(results[0].snippet, "Illustrative example.");
    }

    // --- SearchConfig -> SearchBackend builder -----------------------------

    #[test]
    fn config_default_builds_ddg() {
        let cfg: SearchConfig = toml::from_str("").unwrap();
        assert!(matches!(cfg.build(), Ok(SearchBackend::DdgScrape)));
    }

    #[test]
    fn config_google_cse_from_toml_builds() {
        let cfg: SearchConfig = toml::from_str(
            r#"
            backend = "google_cse"
            [google_cse]
            cx = "my-cx"
            api_key = "injected-key"
            "#,
        )
        .unwrap();
        match cfg.build().unwrap() {
            SearchBackend::GoogleCse { api_key, cx } => {
                assert_eq!(api_key, "injected-key");
                assert_eq!(cx, "my-cx");
            }
            other => panic!("expected GoogleCse, got {other:?}"),
        }
    }

    #[test]
    fn config_google_cse_missing_key_is_typed() {
        // Consumer selected google_cse but injected no key.
        let cfg: SearchConfig = toml::from_str(
            r#"
            backend = "google_cse"
            [google_cse]
            cx = "my-cx"
            "#,
        )
        .unwrap();
        assert!(matches!(cfg.build(), Err(WebError::MissingApiKey(_))));
    }

    #[test]
    fn config_brave_missing_section_is_typed() {
        let cfg: SearchConfig = toml::from_str(r#"backend = "brave""#).unwrap();
        assert!(matches!(cfg.build(), Err(WebError::BackendConfig { .. })));
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

    // --- WebError transient/permanent taxonomy -----------------------------

    #[test]
    fn web_error_transient_classification() {
        // Transient: 5xx and recoverable browser faults bubble up as transient.
        assert!(WebError::Fetch(FetchError::Status(503)).is_transient());
        assert!(WebError::Browser(BrowserError::Crashed("x".into())).is_transient());
        assert!(WebError::Browser(BrowserError::Timeout("x".into())).is_transient());

        // Permanent: 4xx, invalid URL, NoBrowser, and search-level errors never retry.
        assert!(WebError::Fetch(FetchError::Status(404)).is_permanent());
        assert!(WebError::Http {
            url: "u".into(),
            status: 404,
        }
        .is_permanent());
        assert!(WebError::InvalidUrl { url: "u".into() }.is_permanent());
        assert!(WebError::NoBrowser { hint: "h".into() }.is_permanent());
        assert!(WebError::Browser(BrowserError::NoBrowser { hint: "h".into() }).is_permanent());
        assert!(WebError::MissingApiKey("google_cse".into()).is_permanent());
        assert!(WebError::SearchParse {
            backend: "ddg".into(),
            detail: "d".into(),
        }
        .is_permanent());
        assert!(WebError::EmptyResults { query: "q".into() }.is_permanent());
    }

    // --- map_engine_err lifts 4xx / NoBrowser into dedicated variants -------

    #[test]
    fn map_engine_err_lifts_4xx_to_http() {
        let mapped = map_engine_err("https://x/", EngineError::T0(FetchError::Status(404)));
        assert!(matches!(mapped, WebError::Http { status: 404, .. }));
        assert!(mapped.is_permanent());
    }

    #[test]
    fn map_engine_err_keeps_5xx_as_transient_fetch() {
        let mapped = map_engine_err("https://x/", EngineError::T0(FetchError::Status(503)));
        assert!(matches!(mapped, WebError::Fetch(_)));
        assert!(mapped.is_transient(), "5xx must stay retryable through the map");
    }

    #[test]
    fn map_engine_err_lifts_t1_no_browser() {
        let mapped = map_engine_err(
            "https://x/",
            EngineError::T1(BrowserError::NoBrowser { hint: "none".into() }),
        );
        assert!(matches!(mapped, WebError::NoBrowser { .. }));
    }

    // --- DDG: real-empty (Ok) vs markup-changed (Err) ----------------------

    #[test]
    fn ddg_no_results_page_is_ok_empty() {
        // DDG's own "no results" marker → legitimately zero hits, not a parse error.
        let html = r#"<html><body><div class="no-results">No results found.</div></body></html>"#;
        let out = DdgScrapeDialect.parse_response(html).unwrap();
        assert!(out.is_empty(), "explicit no-results page must be Ok(vec![])");
    }

    #[test]
    fn ddg_unexpected_markup_is_search_parse_error() {
        // No result rows AND no "no results" marker → the selectors went stale.
        let html = r#"<html><body><div class="totally-different-layout">hi</div></body></html>"#;
        let err = DdgScrapeDialect
            .parse_response(html)
            .expect_err("stale markup must be a typed parse error, not silent empty");
        match err {
            WebError::SearchParse { backend, .. } => assert_eq!(backend, "ddg"),
            other => panic!("expected SearchParse, got {other:?}"),
        }
    }

    // --- Malformed / edge payloads → typed error, never panic --------------

    #[test]
    fn google_cse_malformed_json_is_search_parse() {
        let d = GoogleCseDialect {
            api_key: "k".into(),
            cx: "cx".into(),
        };
        let err = d.parse_response("{ this is not json").expect_err("malformed");
        assert!(matches!(err, WebError::SearchParse { .. }));
    }

    #[test]
    fn google_cse_api_error_body_is_backend_config() {
        // A structured Google API error (bad key/quota) → typed BackendConfig, not a
        // silent empty result set.
        let body = r#"{"error":{"code":403,"message":"API key not valid"}}"#;
        let d = GoogleCseDialect {
            api_key: "k".into(),
            cx: "cx".into(),
        };
        match d.parse_response(body).expect_err("api error") {
            WebError::BackendConfig { backend, detail } => {
                assert_eq!(backend, "google_cse");
                assert!(detail.contains("API key not valid"));
            }
            other => panic!("expected BackendConfig, got {other:?}"),
        }
    }

    #[test]
    fn google_cse_valid_but_no_items_is_ok_empty() {
        let d = GoogleCseDialect {
            api_key: "k".into(),
            cx: "cx".into(),
        };
        let out = d
            .parse_response(r#"{"kind":"customsearch#search","searchInformation":{}}"#)
            .unwrap();
        assert!(out.is_empty(), "valid response with no items → Ok(vec![])");
    }

    #[test]
    fn google_cse_build_request_missing_cx_is_backend_config() {
        let d = GoogleCseDialect {
            api_key: "KEY".into(),
            cx: "  ".into(),
        };
        assert!(matches!(
            d.build_request("q", &SearchOpts::default()),
            Err(WebError::BackendConfig { .. })
        ));
    }

    #[test]
    fn brave_malformed_json_is_search_parse() {
        let d = BraveApiDialect {
            api_key: "k".into(),
        };
        assert!(matches!(
            d.parse_response("<html>not json</html>"),
            Err(WebError::SearchParse { .. })
        ));
    }

    #[test]
    fn brave_no_web_results_is_ok_empty() {
        let d = BraveApiDialect {
            api_key: "k".into(),
        };
        let out = d.parse_response(r#"{"query":{"original":"q"}}"#).unwrap();
        assert!(out.is_empty(), "no web.results → Ok(vec![])");
    }

    #[test]
    fn brave_missing_key_build_request_is_typed() {
        let d = BraveApiDialect {
            api_key: String::new(),
        };
        assert!(matches!(
            d.build_request("q", &SearchOpts::default()),
            Err(WebError::MissingApiKey(_))
        ));
    }

    // --- SearchConfig::build remaining branches ----------------------------

    #[test]
    fn config_brave_with_key_builds() {
        let cfg: SearchConfig = toml::from_str(
            r#"
            backend = "brave"
            [brave]
            api_key = "brave-key"
            "#,
        )
        .unwrap();
        match cfg.build().unwrap() {
            SearchBackend::BraveApi { api_key } => assert_eq!(api_key, "brave-key"),
            other => panic!("expected BraveApi, got {other:?}"),
        }
    }

    #[test]
    fn config_google_cse_empty_cx_is_backend_config() {
        // Key injected, but the `cx` engine id is present-and-blank → build() must
        // fail typed BackendConfig (not silently proceed with an empty cx).
        let cfg: SearchConfig = toml::from_str(
            r#"
            backend = "google_cse"
            [google_cse]
            cx = ""
            api_key = "injected-key"
            "#,
        )
        .unwrap();
        match cfg.build() {
            Err(WebError::BackendConfig { backend, .. }) => assert_eq!(backend, "google_cse"),
            other => panic!("expected BackendConfig for empty cx, got {other:?}"),
        }
    }

    #[test]
    fn config_google_cse_missing_section_is_backend_config() {
        let cfg: SearchConfig = toml::from_str(r#"backend = "google_cse""#).unwrap();
        assert!(matches!(cfg.build(), Err(WebError::BackendConfig { .. })));
    }

    #[test]
    fn config_brave_missing_key_is_missing_api_key() {
        let cfg: SearchConfig = toml::from_str(
            r#"
            backend = "brave"
            [brave]
            api_key = ""
            "#,
        )
        .unwrap();
        assert!(matches!(cfg.build(), Err(WebError::MissingApiKey(_))));
    }

    // --- requires_t1 routes straight to the browser tier -------------------

    #[tokio::test]
    async fn requires_t1_url_goes_straight_to_browser() {
        // Depends on the committed force_t1 profile (xueqiu). If profiles didn't load
        // in this cwd, the premise isn't met — skip rather than assert a false thing.
        let t1_url = "https://xueqiu.com/S/SH600000";
        if !profiles::requires_t1(t1_url) {
            eprintln!("skip: xueqiu force_t1 profile not loaded in this cwd");
            return;
        }
        // Browser tier can never launch → a requires_t1 URL must surface NoBrowser
        // (proving it bypassed the static path and went straight to the browser).
        let engine = Engine::with_browser_pool(BrowserPool::with_executable(
            "/nonexistent/definitely-not-a-browser",
        ));
        let opts = FetchOpts {
            max_attempts: 1,
            backoff: Duration::ZERO,
            ..Default::default()
        };
        let err = engine
            .fetch_page(t1_url, &opts)
            .await
            .expect_err("no browser available for a force_t1 URL");
        assert!(
            matches!(err, WebError::NoBrowser { .. }),
            "force_t1 URL with no browser must be typed NoBrowser, got {err:?}"
        );
    }
}
