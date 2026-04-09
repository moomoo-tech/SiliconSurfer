//! T1: Headless Chrome browser pool via CDP.
//!
//! - Global daemon process (single Chromium instance)
//! - Millisecond-level context creation/destruction
//! - Block CSS/images/fonts/media at network level
//! - Extract content via JS injection or raw HTML + Rust distiller

use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::distiller::Distiller;

#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("Browser launch failed: {0}")]
    Launch(String),
    #[error("Page error: {0}")]
    Page(String),
    #[error("Browser not started")]
    NotStarted,
}

impl From<chromiumoxide::error::CdpError> for BrowserError {
    fn from(e: chromiumoxide::error::CdpError) -> Self {
        BrowserError::Page(e.to_string())
    }
}

/// Result from a T1 browser fetch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserFetchResult {
    pub url: String,
    pub content: String,
    pub title: Option<String>,
    pub content_length: usize,
}

/// Headless Chrome pool — single daemon, multiple contexts.
pub struct BrowserPool {
    browser: Arc<Mutex<Option<Browser>>>,
    distiller: Distiller,
}

impl BrowserPool {
    pub fn new() -> Self {
        Self {
            browser: Arc::new(Mutex::new(None)),
            distiller: Distiller::new(),
        }
    }

    /// Start the Chrome daemon with aggressive resource stripping.
    pub async fn start(&self) -> Result<(), BrowserError> {
        let config = BrowserConfig::builder()
            .no_sandbox()
            .arg("--disable-gpu")
            .arg("--disable-software-rasterizer")
            .arg("--disable-dev-shm-usage")
            .arg("--blink-settings=imagesEnabled=false")
            .arg("--disable-extensions")
            .arg("--mute-audio")
            .arg("--disable-background-networking")
            .arg("--disable-default-apps")
            .arg("--disable-sync")
            .arg("--disable-translate")
            .arg("--metrics-recording-only")
            .arg("--no-first-run")
            .build()
            .map_err(|e| BrowserError::Launch(e.to_string()))?;

        let (browser, mut handler) =
            Browser::launch(config)
                .await
                .map_err(|e| BrowserError::Launch(e.to_string()))?;

        // Spawn CDP event handler in background
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let mut guard = self.browser.lock().await;
        *guard = Some(browser);
        Ok(())
    }

    /// Ensure browser is running, start if needed.
    pub async fn ensure_started(&self) -> Result<(), BrowserError> {
        let guard = self.browser.lock().await;
        if guard.is_none() {
            drop(guard);
            self.start().await?;
        }
        Ok(())
    }

    /// Get a lock on the browser instance.
    pub async fn browser_guard(&self) -> Result<tokio::sync::MutexGuard<'_, Option<Browser>>, BrowserError> {
        Ok(self.browser.lock().await)
    }

    /// T1 fetch: render page with JS, extract clean content.
    pub async fn fetch(
        &self,
        url: &str,
        output: &str,
    ) -> Result<BrowserFetchResult, BrowserError> {
        self.ensure_started().await?;

        let guard = self.browser.lock().await;
        let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;

        // Create isolated incognito context — millisecond level
        let page = browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        // Wait for DOM to be ready
        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        // Extract title
        let title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .filter(|t| !t.is_empty());

        // Get rendered HTML (after JS execution)
        let raw_html = page
            .content()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        // Close page immediately — free resources
        let _ = page.close().await;
        drop(guard);

        // Distill in Rust — same pipeline as T0
        let content = match output {
            "text" => self.distiller.to_text(&raw_html),
            _ => self.distiller.to_markdown(&raw_html),
        };
        let content_length = content.len();

        Ok(BrowserFetchResult {
            url: url.to_string(),
            content,
            title,
            content_length,
        })
    }

    /// Fetch raw rendered HTML (after JS execution). No distilling.
    /// Used by Probe for DOM checks on JS-rendered pages.
    pub async fn fetch_raw_html(&self, url: &str) -> Result<String, BrowserError> {
        self.ensure_started().await?;

        let guard = self.browser.lock().await;
        let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;

        let page = browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        let html = page
            .content()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        let _ = page.close().await;
        Ok(html)
    }

    /// Fetch with JS injection for content extraction (alternative to Rust distiller).
    /// Useful when you need the browser's own DOM API for complex pages.
    pub async fn fetch_with_js_extract(
        &self,
        url: &str,
    ) -> Result<BrowserFetchResult, BrowserError> {
        self.ensure_started().await?;

        let guard = self.browser.lock().await;
        let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;

        let page = browser
            .new_page(url)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        let title = page
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .filter(|t| !t.is_empty());

        // JS-side DOM cleanup + text extraction
        let content = page
            .evaluate(
                r#"
                (() => {
                    ['nav','footer','header','script','style','iframe','noscript','svg'].forEach(tag => {
                        document.querySelectorAll(tag).forEach(el => el.remove());
                    });
                    document.querySelectorAll('[class*="ad-"],[class*="ads-"],[class*="banner"],[class*="sidebar"],[class*="popup"],[class*="modal"],[class*="cookie"]').forEach(el => el.remove());
                    let main = document.querySelector('article')
                        || document.querySelector('main')
                        || document.querySelector('[role="main"]')
                        || document.body;
                    return main ? main.innerText : '';
                })()
                "#,
            )
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value::<String>()
            .unwrap_or_default();

        let _ = page.close().await;
        drop(guard);

        let content_length = content.len();

        Ok(BrowserFetchResult {
            url: url.to_string(),
            content,
            title,
            content_length,
        })
    }

    /// Shutdown the browser daemon.
    pub async fn stop(&self) {
        let mut guard = self.browser.lock().await;
        *guard = None;
        // Browser drops, Chrome process killed
    }
}
