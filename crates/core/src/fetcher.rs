use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::distiller::Distiller;
use crate::distiller_fast::FastDistiller;

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

pub struct Fetcher {
    client: Client,
    distiller: Distiller,
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

        let (title, content) = if fast {
            let title = FastDistiller::extract_title(&raw_html);
            let content = match opts.output.as_str() {
                "text" => FastDistiller::to_text(&raw_html),
                _ => FastDistiller::to_markdown_with_base(&raw_html, Some(&opts.url)),
            };
            (title, content)
        } else {
            let title = self.distiller.extract_title(&raw_html);
            let content = match opts.output.as_str() {
                "text" => self.distiller.to_text(&raw_html),
                _ => self.distiller.to_markdown_with_base(&raw_html, Some(&opts.url)),
            };
            (title, content)
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
