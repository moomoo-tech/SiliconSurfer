use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::distiller::Distiller;
use crate::distiller_fast::{DistillMode, FastDistiller};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOptions {
    pub url: String,
    /// Output format: "markdown" or "text"
    #[serde(default = "default_output")]
    pub output: String,
    /// Custom user agent
    pub user_agent: Option<String>,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Distill mode (llm_friendly, reader, operator, spider, developer, data)
    #[serde(default)]
    pub distill_mode: DistillMode,
}

fn default_output() -> String {
    "markdown".to_string()
}

fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub url: String,
    pub content: String,
    pub title: Option<String>,
    pub status: u16,
    pub content_length: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Non-success status: {0}")]
    Status(u16),
}

impl FetchError {
    /// Whether retrying the same request could plausibly succeed.
    ///
    /// Transient: connection resets, request timeouts, and 5xx server errors.
    /// Permanent: 4xx client errors and anything that isn't a timeout/connect
    /// failure (invalid URL, TLS, DNS-not-found — reqwest cannot cleanly separate
    /// DNS-not-found from a resettable connect error, so we err toward NOT retrying).
    pub fn is_transient(&self) -> bool {
        match self {
            FetchError::Status(code) => (500..600).contains(code),
            FetchError::Request(e) => e.is_timeout() || e.is_connect(),
        }
    }
}

pub struct Fetcher {
    client: Client,
    distiller: Distiller,
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .cookie_store(true)
            .build()
            .expect("failed to build HTTP client");

        Self {
            client,
            distiller: Distiller::new(),
        }
    }

    /// T0: Lightweight HTTP fetch + distill (default scraper distiller)
    pub async fn fetch(&self, opts: FetchOptions) -> Result<FetchResult, FetchError> {
        self.fetch_inner(opts, false).await
    }

    /// T0 with fast lol_html streaming distiller
    pub async fn fetch_fast(&self, opts: FetchOptions) -> Result<FetchResult, FetchError> {
        self.fetch_inner(opts, true).await
    }

    /// Fetch the raw response body WITHOUT distilling.
    ///
    /// The search backend needs the page's original HTML structure (result rows,
    /// anchors) which the distiller would strip, so it takes this path while still
    /// reusing the shared reqwest client, timeout, and typed [`FetchError`].
    pub async fn fetch_raw_html(&self, opts: FetchOptions) -> Result<String, FetchError> {
        let mut req = self.client.get(&opts.url);
        if let Some(ua) = &opts.user_agent {
            req = req.header("User-Agent", ua);
        }
        req = req.timeout(std::time::Duration::from_secs(opts.timeout_secs));
        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Err(FetchError::Status(resp.status().as_u16()));
        }
        Ok(resp.text().await?)
    }

    async fn fetch_inner(&self, opts: FetchOptions, fast: bool) -> Result<FetchResult, FetchError> {
        let mut req = self.client.get(&opts.url);

        if let Some(ua) = &opts.user_agent {
            req = req.header("User-Agent", ua);
        }

        req = req.timeout(std::time::Duration::from_secs(opts.timeout_secs));

        let resp = req.send().await?;
        let status = resp.status().as_u16();

        if !resp.status().is_success() {
            return Err(FetchError::Status(status));
        }

        let raw_html = resp.text().await?;

        let title = if fast {
            FastDistiller::extract_title(&raw_html)
        } else {
            self.distiller.extract_title(&raw_html)
        };

        let content = match opts.output.as_str() {
            "text" => {
                if fast {
                    FastDistiller::to_text(&raw_html)
                } else {
                    self.distiller.to_text(&raw_html)
                }
            }
            // Distill mode decides the output *shape*. The operator/spider/developer/data
            // shapes only exist in the streaming strategy engine, so they always route there
            // regardless of `fast` — never silently downgrade a requested mode to reader.
            // Only reader/markdown has two engine implementations, and `fast` picks between
            // them (AST scraper vs lol_html stream).
            _ => match opts.distill_mode {
                DistillMode::LlmFriendly | DistillMode::Reader => {
                    if fast {
                        FastDistiller::distill(&raw_html, opts.distill_mode, Some(&opts.url))
                    } else {
                        self.distiller
                            .to_markdown_with_base(&raw_html, Some(&opts.url))
                    }
                }
                _ => FastDistiller::distill(&raw_html, opts.distill_mode, Some(&opts.url)),
            },
        };
        let content_length = content.len();

        Ok(FetchResult {
            url: opts.url.clone(),
            content,
            title,
            status,
            content_length,
        })
    }
}
