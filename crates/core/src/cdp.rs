//! CDP interaction layer — our own "hands" replacing Playwright.
//!
//! BrowserSession holds a Chrome page and provides:
//! - navigate(url)
//! - fill(selector, value) / fill_by_agent_id(@eN, value)
//! - click(selector) / click_by_agent_id(@eN)
//! - submit(selector)
//! - content() → raw HTML
//! - see(mode) → distilled content via FastDistiller
//! - eval(js) → run arbitrary JS

use crate::browser::{BrowserError, BrowserPool};
use crate::distiller_fast::{DistillMode, FastDistiller};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A browser session — one tab with state (cookies, history).
pub struct BrowserSession {
    pool: Arc<BrowserPool>,
    page: Option<chromiumoxide::Page>,
    current_url: String,
}

/// Result of an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub url: String,
    pub detail: String,
}

impl BrowserSession {
    pub async fn new(pool: Arc<BrowserPool>) -> Result<Self, BrowserError> {
        pool.ensure_started().await?;
        Ok(Self {
            pool,
            page: None,
            current_url: String::new(),
        })
    }

    /// Navigate to a URL — creates page if needed.
    pub async fn navigate(&mut self, url: &str) -> Result<ActionResult, BrowserError> {
        // Only need the lock to create a new page; drop guard before async I/O.
        let page = if let Some(ref p) = self.page {
            p.goto(url)
                .await
                .map_err(|e| BrowserError::Page(e.to_string()))?;
            p.clone()
        } else {
            let guard = self.pool.browser_guard().await?;
            let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;
            let p = browser
                .new_page(url)
                .await
                .map_err(|e| BrowserError::Page(e.to_string()))?;
            drop(guard);
            p
        };

        page.wait_for_navigation()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;

        self.current_url = url.to_string();
        self.page = Some(page);

        Ok(ActionResult {
            success: true,
            url: url.to_string(),
            detail: format!("Navigated to {}", url),
        })
    }

    /// Get current page HTML.
    pub async fn content(&self) -> Result<String, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        page.content()
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))
    }

    /// Get current URL.
    pub async fn url(&self) -> Result<String, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let url: String = page
            .evaluate("window.location.href")
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))?;
        Ok(url)
    }

    /// SEE — get distilled view of current page.
    pub async fn see(&self, mode: DistillMode) -> Result<String, BrowserError> {
        let html = self.content().await?;
        let url = self.url().await.unwrap_or_default();
        Ok(FastDistiller::distill(&html, mode, Some(&url)))
    }

    /// Click an element by CSS selector.
    pub async fn click(&self, selector: &str) -> Result<ActionResult, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let safe_selector =
            serde_json::to_string(selector).map_err(|e| BrowserError::Page(e.to_string()))?;
        let js = format!(
            r#"(() => {{
                const el = document.querySelector({safe_selector});
                if (!el) return {{ success: false, detail: 'Element not found: ' + {safe_selector} }};
                el.click();
                return {{ success: true, detail: 'Clicked ' + el.tagName }};
            }})()"#,
        );

        let result: serde_json::Value = page
            .evaluate(js)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))?;

        // Wait for navigation if it happens
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let url = self.url().await.unwrap_or_default();
        Ok(ActionResult {
            success: result["success"].as_bool().unwrap_or(false),
            url,
            detail: result["detail"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Click by @eN agent reference.
    pub async fn click_agent_ref(&self, ref_id: &str) -> Result<ActionResult, BrowserError> {
        // @e3 → data-agent-id="e3"
        let id = ref_id.trim_start_matches('@');
        self.click(&format!("[data-agent-id='{}']", id)).await
    }

    /// Fill a form field by CSS selector.
    pub async fn fill(&self, selector: &str, value: &str) -> Result<ActionResult, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let safe_selector =
            serde_json::to_string(selector).map_err(|e| BrowserError::Page(e.to_string()))?;
        let safe_value =
            serde_json::to_string(value).map_err(|e| BrowserError::Page(e.to_string()))?;
        let js = format!(
            r#"(() => {{
                const el = document.querySelector({safe_selector});
                if (!el) return {{ success: false, detail: 'Element not found: ' + {safe_selector} }};
                el.value = {safe_value};
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return {{ success: true, detail: 'Filled ' + el.name + ' = ' + {safe_value} }};
            }})()"#,
        );

        let result: serde_json::Value = page
            .evaluate(js)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))?;

        Ok(ActionResult {
            success: result["success"].as_bool().unwrap_or(false),
            url: self.current_url.clone(),
            detail: result["detail"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Fill by @eN agent reference.
    pub async fn fill_agent_ref(
        &self,
        ref_id: &str,
        value: &str,
    ) -> Result<ActionResult, BrowserError> {
        let id = ref_id.trim_start_matches('@');
        self.fill(&format!("[data-agent-id='{}']", id), value).await
    }

    /// Fill by field name attribute.
    pub async fn fill_by_name(
        &self,
        name: &str,
        value: &str,
    ) -> Result<ActionResult, BrowserError> {
        self.fill(&format!("[name='{}']", name), value).await
    }

    /// Submit a form (click submit button).
    pub async fn submit(&self, selector: &str) -> Result<ActionResult, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let safe_selector =
            serde_json::to_string(selector).map_err(|e| BrowserError::Page(e.to_string()))?;
        let js = format!(
            r#"(() => {{
                const el = document.querySelector({safe_selector});
                if (!el) {{
                    const btn = document.querySelector('button[type="submit"], input[type="submit"]');
                    if (btn) {{ btn.click(); return {{ success: true, detail: 'Clicked fallback submit' }}; }}
                    const form = document.querySelector('form');
                    if (form) {{ form.submit(); return {{ success: true, detail: 'Submitted form directly' }}; }}
                    return {{ success: false, detail: 'No submit element found' }};
                }}
                el.click();
                return {{ success: true, detail: 'Clicked ' + el.tagName }};
            }})()"#,
        );

        let result: serde_json::Value = page
            .evaluate(js)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))?;

        // Wait for form submission
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let url = self.url().await.unwrap_or_default();
        Ok(ActionResult {
            success: result["success"].as_bool().unwrap_or(false),
            url,
            detail: result["detail"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Run arbitrary JavaScript.
    pub async fn eval(&self, js: &str) -> Result<serde_json::Value, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        page.evaluate(js.to_string())
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))
    }

    /// Close the session.
    pub async fn close(mut self) {
        if let Some(page) = self.page.take() {
            let _ = page.close().await;
        }
    }
}
