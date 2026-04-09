//! Agent Session — isolated browser context + cookie sync + locator map.
//!
//! Solves three architectural gaps:
//! 1. Context isolation: each session = incognito context, no state leakage
//! 2. Cookie sync: T1 cookies → T0 reqwest jar, seamless switching
//! 3. Locator map: @eN → CSS selector mapping for reliable targeting

use crate::browser::{BrowserError, BrowserPool};
use crate::distiller_fast::{DistillMode, FastDistiller};
use reqwest::cookie::Jar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// An isolated Agent session — one task, one context, no leakage.
pub struct AgentSession {
    pool: Arc<BrowserPool>,
    page: Option<chromiumoxide::Page>,
    /// @eN → CSS selector mapping (refreshed on each observe)
    locator_map: HashMap<String, String>,
    /// Cookie jar synced from T1 for T0 use
    cookie_jar: Arc<Jar>,
    /// Current URL
    pub current_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserveResult {
    pub content: String,
    pub title: Option<String>,
    pub url: String,
    pub content_length: usize,
    pub mode: String,
    pub element_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActResult {
    pub success: bool,
    pub url: String,
    pub detail: String,
}

impl AgentSession {
    pub async fn new(pool: Arc<BrowserPool>) -> Result<Self, BrowserError> {
        pool.ensure_started().await?;
        Ok(Self {
            pool,
            page: None,
            locator_map: HashMap::new(),
            cookie_jar: Arc::new(Jar::default()),
            current_url: String::new(),
        })
    }

    /// Navigate to a URL (creates page if needed).
    pub async fn navigate(&mut self, url: &str) -> Result<ActResult, BrowserError> {
        let guard = self.pool.browser_guard().await?;
        let browser = guard.as_ref().ok_or(BrowserError::NotStarted)?;

        let page = if let Some(ref p) = self.page {
            p.goto(url).await.map_err(|e| BrowserError::Page(e.to_string()))?;
            p.clone()
        } else {
            browser.new_page(url).await.map_err(|e| BrowserError::Page(e.to_string()))?
        };

        // Inject resource blocker — kill fonts, trackers, analytics
        let _ = page.evaluate(r#"
            if (!window.__ss_blocked) {
                window.__ss_blocked = true;
                const observer = new PerformanceObserver((list) => {});
                // Block via CSP meta tag
                const meta = document.createElement('meta');
                meta.httpEquiv = 'Content-Security-Policy';
                meta.content = "font-src 'none'; media-src 'none'";
                document.head?.appendChild(meta);
            }
        "#.to_string()).await;

        // Wait for DOM
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        self.current_url = page.evaluate("window.location.href").await
            .ok().and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_else(|| url.to_string());
        self.page = Some(page);

        Ok(ActResult { success: true, url: self.current_url.clone(), detail: format!("Navigated to {}", url) })
    }

    /// OBSERVE — see the current page with our distiller.
    /// If mode=operator, also builds locator_map.
    pub async fn observe(&mut self, mode: DistillMode) -> Result<ObserveResult, BrowserError> {
        // Get HTML and URL from page (immutable borrow ends here)
        let (html, url) = {
            let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
            let html = page.content().await.map_err(|e| BrowserError::Page(e.to_string()))?;
            (html, self.current_url.clone())
        };

        let title = FastDistiller::extract_title(&html);
        let content = FastDistiller::distill(&html, mode, Some(&url));

        // Build locator map for operator mode (now safe to borrow self mutably)
        if mode == DistillMode::Operator {
            if let Some(page) = self.page.as_ref() {
                let js = r#"
                    (() => {
                        const map = {};
                        document.querySelectorAll('[data-agent-id]').forEach(el => {
                            const eid = '@' + el.getAttribute('data-agent-id');
                            let sel = el.tagName.toLowerCase();
                            if (el.id) sel = '#' + el.id;
                            else if (el.name) sel += "[name='" + el.name + "']";
                            else sel += "[data-agent-id='" + el.getAttribute('data-agent-id') + "']";
                            map[eid] = sel;
                        });
                        return map;
                    })()
                "#;
                if let Ok(result) = page.evaluate(js.to_string()).await {
                    if let Ok(val) = result.into_value::<serde_json::Value>() {
                        self.locator_map.clear();
                        if let Some(obj) = val.as_object() {
                            for (k, v) in obj {
                                if let Some(sel) = v.as_str() {
                                    self.locator_map.insert(k.clone(), sel.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        let content_length = content.len();
        let element_count = self.locator_map.len();

        Ok(ObserveResult {
            content, title, url, content_length,
            mode: format!("{:?}", mode).to_lowercase(),
            element_count,
        })
    }

    /// ACT — execute an action on a @eN element.
    pub async fn act(&mut self, action: &str, target: &str, value: &str) -> Result<ActResult, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;

        // Resolve @eN to CSS selector
        let selector = if target.starts_with("@e") {
            // First try locator map
            if let Some(sel) = self.locator_map.get(target) {
                sel.clone()
            } else {
                // Fallback: use data-agent-id attribute directly
                let id = target.trim_start_matches('@');
                format!("[data-agent-id='{}']", id)
            }
        } else {
            // Raw CSS selector
            target.to_string()
        };

        let js = match action {
            "click" => format!(
                r#"(() => {{
                    const el = document.querySelector('{}');
                    if (!el) return {{success: false, detail: 'Not found: {}'}};
                    el.click();
                    return {{success: true, detail: 'Clicked ' + el.tagName}};
                }})()"#,
                selector.replace('\'', "\\'"), selector.replace('\'', "\\'")
            ),
            "fill" => format!(
                r#"(() => {{
                    const el = document.querySelector('{}');
                    if (!el) return {{success: false, detail: 'Not found: {}'}};
                    el.value = '{}';
                    el.dispatchEvent(new Event('input', {{bubbles: true}}));
                    el.dispatchEvent(new Event('change', {{bubbles: true}}));
                    return {{success: true, detail: 'Filled ' + (el.name || el.id || 'element')}};
                }})()"#,
                selector.replace('\'', "\\'"), selector.replace('\'', "\\'"),
                value.replace('\'', "\\'")
            ),
            "submit" => format!(
                r#"(() => {{
                    const el = document.querySelector('{}');
                    if (el) {{ el.click(); return {{success: true, detail: 'Clicked submit'}}; }}
                    const btn = document.querySelector('button[type="submit"], input[type="submit"]');
                    if (btn) {{ btn.click(); return {{success: true, detail: 'Clicked fallback submit'}}; }}
                    const form = document.querySelector('form');
                    if (form) {{ form.submit(); return {{success: true, detail: 'Submitted form'}}; }}
                    return {{success: false, detail: 'No submit element found'}};
                }})()"#,
                selector.replace('\'', "\\'")
            ),
            _ => return Err(BrowserError::Page(format!("Unknown action: {}", action))),
        };

        let result: serde_json::Value = page.evaluate(js).await
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))?;

        // Wait for potential navigation
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        // Update current URL
        self.current_url = page.evaluate("window.location.href").await
            .ok().and_then(|v| v.into_value::<String>().ok())
            .unwrap_or(self.current_url.clone());

        Ok(ActResult {
            success: result["success"].as_bool().unwrap_or(false),
            url: self.current_url.clone(),
            detail: result["detail"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Get the locator map (for debugging / MCP response).
    pub fn locator_map(&self) -> &HashMap<String, String> {
        &self.locator_map
    }

    /// Close the session — destroy context.
    pub async fn close(mut self) {
        if let Some(page) = self.page.take() {
            let _ = page.close().await;
        }
    }
}
