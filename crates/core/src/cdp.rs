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

use chromiumoxide_cdp::cdp::browser_protocol::network::CookieParam;

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
        // Create blank page under lock (no network I/O), then navigate after lock is dropped.
        let page = if let Some(ref p) = self.page {
            p.goto(url)
                .await
                .map_err(|e| BrowserError::Page(e.to_string()))?;
            p.clone()
        } else {
            let p = {
                let guard = self.pool.browser_guard().await?;
                let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;
                browser
                    .new_page("about:blank")
                    .await
                    .map_err(|e| BrowserError::Page(e.to_string()))?
                // guard drops here — lock released
            };
            p.goto(url)
                .await
                .map_err(|e| BrowserError::Page(e.to_string()))?;
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

        // Click may trigger navigation which destroys the execution context.
        // "Execution context was destroyed" means the click succeeded AND caused a page transition.
        let result = Self::evaluate_tolerant(page, &js).await?;

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

        let result = Self::evaluate_tolerant(page, &js).await?;

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

    /// Evaluate JS, tolerating "execution context destroyed" errors.
    /// When a click/submit triggers navigation, V8 destroys the context before
    /// returning the result. This is expected — it means the action succeeded.
    async fn evaluate_tolerant(
        page: &chromiumoxide::Page,
        js: &str,
    ) -> Result<serde_json::Value, BrowserError> {
        match page.evaluate(js.to_string()).await {
            Ok(val) => Ok(val
                .into_value()
                .unwrap_or(serde_json::json!({"success": true, "detail": "Executed"}))),
            Err(e) => {
                let err = e.to_string();
                if err.contains("context was destroyed")
                    || err.contains("Target closed")
                    || err.contains("Session closed")
                    || err.contains("Cannot find context")
                {
                    // Navigation killed the context — action succeeded
                    Ok(serde_json::json!({"success": true, "detail": "Clicked and navigated"}))
                } else {
                    Err(BrowserError::Page(err))
                }
            }
        }
    }

    /// Set a cookie on the current page.
    /// If `url` is provided, it sets domain/path automatically from the URL.
    /// Otherwise, `domain` must be specified.
    pub async fn set_cookie(
        &self,
        name: &str,
        value: &str,
        domain: &str,
        path: Option<&str>,
    ) -> Result<(), BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let mut cookie = CookieParam::new(name, value);
        cookie.domain = Some(domain.to_string());
        cookie.path = Some(path.unwrap_or("/").to_string());
        page.set_cookie(cookie)
            .await
            .map_err(|e| BrowserError::Page(e.to_string()))?;
        Ok(())
    }

    /// Set multiple cookies from a JSON-style list.
    /// Each entry: {name, value, domain, path?, secure?, httpOnly?}
    pub async fn set_cookies_from_json(
        &self,
        cookies_json: &[serde_json::Value],
    ) -> Result<usize, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let mut params = Vec::with_capacity(cookies_json.len());
        for c in cookies_json {
            let name = c["name"].as_str().unwrap_or_default();
            let value = c["value"].as_str().unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let mut cookie = CookieParam::new(name, value);
            cookie.domain = c["domain"].as_str().map(|s| s.to_string());
            cookie.path = Some(c["path"].as_str().unwrap_or("/").to_string());
            cookie.secure = c["secure"].as_bool();
            cookie.http_only = c["httpOnly"].as_bool();
            params.push(cookie);
        }
        let count = params.len();
        if !params.is_empty() {
            page.set_cookies(params)
                .await
                .map_err(|e| BrowserError::Page(e.to_string()))?;
        }
        Ok(count)
    }

    /// Close the session.
    pub async fn close(mut self) {
        if let Some(page) = self.page.take() {
            let _ = page.close().await;
        }
    }
}
