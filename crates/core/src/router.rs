//! Unified routing engine: T0 (reqwest) or T1 (headless Chrome).
//!
//! Automatically picks the right path, or caller can force a mode.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::browser::BrowserPool;
use crate::distiller_fast::DistillMode;
use crate::fetcher::{FetchOptions, Fetcher};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FetchMode {
    /// T0: reqwest only — fast, lightweight
    #[default]
    T0,
    /// T1: headless Chrome — JS rendering
    T1,
    /// Auto: try T0 first, fallback to T1 if content looks empty
    Auto,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("T0 fetch error: {0}")]
    T0(#[from] crate::fetcher::FetchError),
    #[error("T1 browser error: {0}")]
    T1(#[from] crate::browser::BrowserError),
}

/// Unified result that works for both T0 and T1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResult {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub content_length: usize,
    pub mode_used: String,
}

/// The main engine that routes between T0 and T1.
pub struct Engine {
    t0: Fetcher,
    t1: Arc<BrowserPool>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            t0: Fetcher::new(),
            t1: Arc::new(BrowserPool::new()),
        }
    }

    /// Build an engine with a caller-supplied browser pool — e.g. one pinned to a
    /// specific executable (`BrowserPool::with_executable`) resolved from config.
    pub fn with_browser_pool(pool: BrowserPool) -> Self {
        Self {
            t0: Fetcher::new(),
            t1: Arc::new(pool),
        }
    }

    /// Get a reference to the browser pool (for sharing with Probe).
    pub fn browser_pool(&self) -> Arc<BrowserPool> {
        self.t1.clone()
    }

    /// Fetch a URL using the specified mode.
    pub async fn fetch(
        &self,
        url: &str,
        output: &str,
        mode: FetchMode,
    ) -> Result<EngineResult, EngineError> {
        self.fetch_with_opts(url, output, mode, false, DistillMode::default())
            .await
    }

    /// Fetch with fast distiller + distill mode.
    pub async fn fetch_fast(
        &self,
        url: &str,
        output: &str,
        mode: FetchMode,
    ) -> Result<EngineResult, EngineError> {
        self.fetch_with_opts(url, output, mode, true, DistillMode::default())
            .await
    }

    /// Fetch with full options.
    pub async fn fetch_full(
        &self,
        url: &str,
        output: &str,
        mode: FetchMode,
        fast: bool,
        distill: DistillMode,
    ) -> Result<EngineResult, EngineError> {
        self.fetch_with_opts(url, output, mode, fast, distill).await
    }

    async fn fetch_with_opts(
        &self,
        url: &str,
        output: &str,
        mode: FetchMode,
        fast: bool,
        distill: DistillMode,
    ) -> Result<EngineResult, EngineError> {
        match mode {
            FetchMode::T0 => self.fetch_t0(url, output, fast, distill).await,
            FetchMode::T1 => self.fetch_t1(url, output).await,
            FetchMode::Auto => self.fetch_auto(url, output, fast, distill).await,
        }
    }

    /// T0: reqwest + Rust distiller
    pub(crate) async fn fetch_t0(
        &self,
        url: &str,
        output: &str,
        fast: bool,
        distill: DistillMode,
    ) -> Result<EngineResult, EngineError> {
        let opts = FetchOptions {
            url: url.to_string(),
            output: output.to_string(),
            user_agent: None,
            timeout_secs: 30,
            distill_mode: distill,
        };
        let result = if fast {
            self.t0.fetch_fast(opts).await?
        } else {
            self.t0.fetch(opts).await?
        };
        Ok(EngineResult {
            url: result.url,
            title: result.title,
            content: result.content,
            content_length: result.content_length,
            mode_used: "t0".to_string(),
        })
    }

    /// T1: headless Chrome + distiller
    pub(crate) async fn fetch_t1(&self, url: &str, output: &str) -> Result<EngineResult, EngineError> {
        let result = self.t1.fetch(url, output).await?;
        Ok(EngineResult {
            url: result.url,
            title: result.title,
            content: result.content,
            content_length: result.content_length,
            mode_used: "t1".to_string(),
        })
    }

    /// Auto: T0 first, fallback to T1 if content looks empty/too short
    async fn fetch_auto(
        &self,
        url: &str,
        output: &str,
        fast: bool,
        distill: DistillMode,
    ) -> Result<EngineResult, EngineError> {
        match self.fetch_t0(url, output, fast, distill).await {
            Ok(result) if !is_sparse_content(&result.content) => Ok(result),
            Ok(_sparse) => {
                // T0 got almost nothing — likely a JS-rendered page, try T1
                tracing::info!("T0 returned sparse content for {}, falling back to T1", url);
                match self.fetch_t1(url, output).await {
                    Ok(t1_result) => Ok(t1_result),
                    Err(_) => Ok(_sparse), // T1 also failed, return T0's sparse result
                }
            }
            Err(_t0_err) => {
                // T0 failed entirely, try T1
                tracing::info!("T0 failed for {}, falling back to T1", url);
                Ok(self.fetch_t1(url, output).await?)
            }
        }
    }

    /// Fetch RAW (undistilled) HTML with the same fastest-first escalation as
    /// [`Engine::fetch`]: reqwest first, and only fall back to the browser when the
    /// static body looks like an unrendered shell. Used by the search backend,
    /// which parses the page's own HTML structure.
    pub(crate) async fn fetch_raw(&self, url: &str) -> Result<String, EngineError> {
        let opts = FetchOptions {
            url: url.to_string(),
            output: "markdown".to_string(),
            user_agent: None,
            timeout_secs: 30,
            distill_mode: DistillMode::default(),
        };
        match self.t0.fetch_raw_html(opts).await {
            Ok(html) if !is_sparse_content(&html) => Ok(html),
            Ok(sparse) => match self.t1.fetch_raw_html(url).await {
                Ok(html) => Ok(html),
                Err(_) => Ok(sparse),
            },
            Err(_) => Ok(self.t1.fetch_raw_html(url).await?),
        }
    }

    /// Start the T1 browser daemon (call once at startup).
    pub async fn start_browser(&self) -> Result<(), crate::browser::BrowserError> {
        self.t1.start().await
    }

    /// Stop the T1 browser daemon.
    pub async fn stop_browser(&self) {
        self.t1.stop().await;
    }
}

/// Decide whether a T0 result is too thin to trust, so Auto should escalate to T1.
///
/// Bare length is not enough: JS-app shells return a short-but-nonempty skeleton
/// (e.g. a 142-byte `Loading…` placeholder) that clears a naive length threshold
/// yet carries zero real content. Treat those placeholder shells as sparse too.
pub(crate) fn is_sparse_content(content: &str) -> bool {
    let trimmed = content.trim();
    let len = trimmed.chars().count();
    if len <= 100 {
        return true;
    }
    // A short body dominated by a JS placeholder is an unrendered SPA shell.
    if len < 400 {
        let lower = trimmed.to_lowercase();
        const PLACEHOLDERS: [&str; 4] = [
            "loading",
            "enable javascript",
            "javascript is required",
            "you need to enable javascript",
        ];
        if PLACEHOLDERS.iter().any(|p| lower.contains(p)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod sparse_tests {
    use super::is_sparse_content;

    #[test]
    fn empty_and_tiny_is_sparse() {
        assert!(is_sparse_content(""));
        assert!(is_sparse_content("   \n  "));
        assert!(is_sparse_content("short"));
    }

    #[test]
    fn js_placeholder_shell_is_sparse() {
        // ~142-byte SPA shell that previously slipped through the >100 byte check.
        let shell = "Loading...\n\n".to_string() + &"Loading ".repeat(15);
        assert!(shell.len() > 100, "shell must clear the old byte threshold");
        assert!(is_sparse_content(&shell));
    }

    #[test]
    fn real_content_is_not_sparse() {
        let real = "# Models & Pricing\n\n".to_string()
            + &"DeepSeek V4 pricing details, context length, thinking mode. ".repeat(10);
        assert!(!is_sparse_content(&real));
    }
}
