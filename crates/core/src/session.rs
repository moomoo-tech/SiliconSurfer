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
            // New page — inject all patches BEFORE any navigation
            let p = browser.new_page("about:blank").await
                .map_err(|e| BrowserError::Page(e.to_string()))?;
            Self::inject_patches(&p).await;
            p.goto(url).await.map_err(|e| BrowserError::Page(e.to_string()))?;
            p
        };

        // Wait for initial DOM load
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Re-inject tab fix + dialog handler for dynamically loaded content
        Self::inject_tab_fix(&page).await;
        Self::inject_dialog_handler(&page).await;

        self.current_url = page.evaluate("window.location.href").await
            .ok().and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_else(|| url.to_string());
        self.page = Some(page);

        Ok(ActResult { success: true, url: self.current_url.clone(), detail: format!("Navigated to {}", url) })
    }

    /// Inject all pre-navigation patches: stealth + tab fix + dialog handler + shadow DOM.
    async fn inject_patches(page: &chromiumoxide::Page) {
        Self::inject_stealth(page).await;
        Self::inject_tab_fix(page).await;
        Self::inject_dialog_handler(page).await;
    }

    /// Bug 9 fix: Rewrite target="_blank" to prevent new tab black hole.
    async fn inject_tab_fix(page: &chromiumoxide::Page) {
        let _ = page.evaluate(r#"
            // Rewrite all existing target="_blank" links
            document.querySelectorAll('a[target="_blank"]').forEach(a => {
                a.setAttribute('target', '_self');
            });
            // Intercept future clicks that try to open new tabs
            document.addEventListener('click', (e) => {
                const a = e.target.closest('a[target="_blank"]');
                if (a) { a.setAttribute('target', '_self'); }
            }, true);
            // Override window.open to navigate in current tab
            window.open = (url) => { if (url) window.location.href = url; };
        "#.to_string()).await;
    }

    /// Bug 10 fix: Auto-accept all JS dialogs (alert/confirm/prompt).
    async fn inject_dialog_handler(page: &chromiumoxide::Page) {
        // Override native dialog functions to prevent V8 freeze
        let _ = page.evaluate(r#"
            window.alert = () => {};
            window.confirm = () => true;
            window.prompt = () => '';
            // Also prevent beforeunload dialogs
            window.addEventListener('beforeunload', (e) => { delete e.returnValue; });
        "#.to_string()).await;
    }

    /// Bug 6 fix: Inject anti-bot stealth patches before page loads.
    async fn inject_stealth(page: &chromiumoxide::Page) {
        // Erase webdriver flag
        let _ = page.evaluate(r#"
            Object.defineProperty(navigator, 'webdriver', {get: () => undefined});
        "#.to_string()).await;

        // Fake plugins (headless Chrome has 0 plugins)
        let _ = page.evaluate(r#"
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5].map(() => ({
                    name: 'Chrome PDF Plugin',
                    description: 'Portable Document Format',
                    filename: 'internal-pdf-viewer',
                    length: 1
                }))
            });
        "#.to_string()).await;

        // Fake languages
        let _ = page.evaluate(r#"
            Object.defineProperty(navigator, 'languages', {get: () => ['en-US', 'en']});
        "#.to_string()).await;

        // Fix permissions API (headless reports 'denied' for everything)
        let _ = page.evaluate(r#"
            const originalQuery = window.navigator.permissions.query;
            window.navigator.permissions.query = (parameters) =>
                parameters.name === 'notifications'
                    ? Promise.resolve({state: Notification.permission})
                    : originalQuery(parameters);
        "#.to_string()).await;

        // Chrome runtime (headless doesn't have it)
        let _ = page.evaluate(r#"
            window.chrome = { runtime: {} };
        "#.to_string()).await;
    }

    /// OBSERVE — see the current page with our distiller.
    /// If mode=operator, also builds locator_map.
    pub async fn observe(&mut self, mode: DistillMode) -> Result<ObserveResult, BrowserError> {
        // Get HTML and URL from page, with ghost text removed + iframes flattened
        let (html, url) = {
            let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;

            // Bug 12 fix: Wait for DOM quiescence (not networkidle)
            // Wait until DOM stops changing for 500ms
            let _ = page.evaluate(r#"
                new Promise(resolve => {
                    let timer;
                    const observer = new MutationObserver(() => {
                        clearTimeout(timer);
                        timer = setTimeout(() => { observer.disconnect(); resolve(); }, 500);
                    });
                    observer.observe(document.body || document, {childList: true, subtree: true});
                    // Fallback: resolve after 3s max
                    setTimeout(() => { observer.disconnect(); resolve(); }, 3000);
                    // If already stable, resolve after 500ms
                    timer = setTimeout(() => { observer.disconnect(); resolve(); }, 500);
                })
            "#.to_string()).await;

            // Bug 11 fix: Shadow Piercer — tag real nodes + flatten shadow content
            let _ = page.evaluate(r#"
                (() => {
                    let counter = 0;
                    function walkAndTag(node) {
                        const children = node.shadowRoot
                            ? node.shadowRoot.childNodes
                            : node.childNodes;
                        for (const child of children) {
                            if (child.nodeType !== Node.ELEMENT_NODE) continue;
                            const tag = child.tagName.toLowerCase();
                            // Tag interactive elements with data-agent-id on the REAL node
                            const isInteractive = ['input','button','a','select','textarea'].includes(tag)
                                || child.getAttribute('role') === 'button'
                                || child.onclick != null;
                            if (isInteractive) {
                                counter++;
                                child.setAttribute('data-agent-id', 'e' + counter);
                            }
                            // Flatten shadow content into light DOM for serialization
                            if (child.shadowRoot) {
                                const flat = document.createElement('div');
                                flat.setAttribute('data-shadow-host', tag);
                                flat.innerHTML = child.shadowRoot.innerHTML;
                                child.appendChild(flat);
                            }
                            walkAndTag(child);
                        }
                    }
                    walkAndTag(document.body || document);
                })()
            "#.to_string()).await;

            // Bug 5 fix: Remove invisible elements BEFORE extracting HTML
            let _ = page.evaluate(r#"
                (() => {
                    const remove = [];
                    document.querySelectorAll('*').forEach(el => {
                        if (el.tagName === 'SCRIPT' || el.tagName === 'STYLE') return;
                        const style = window.getComputedStyle(el);
                        if (style.display === 'none' || style.visibility === 'hidden' ||
                            style.opacity === '0' || el.getAttribute('aria-hidden') === 'true') {
                            remove.push(el);
                        }
                    });
                    remove.forEach(el => el.remove());
                })()
            "#.to_string()).await;

            let mut html = page.content().await.map_err(|e| BrowserError::Page(e.to_string()))?;

            // Flatten same-origin iframe contents into main HTML
            let iframe_js = r#"(() => {
                const results = [];
                document.querySelectorAll('iframe').forEach((iframe, i) => {
                    try {
                        const doc = iframe.contentDocument || iframe.contentWindow?.document;
                        if (doc && doc.body) {
                            results.push({src: iframe.src || '', html: doc.body.innerHTML});
                        } else {
                            results.push({src: iframe.src || '', html: null, cross_origin: true});
                        }
                    } catch(e) {
                        results.push({src: iframe.src || '', html: null, cross_origin: true});
                    }
                });
                return results;
            })()"#;

            if let Ok(result) = page.evaluate(iframe_js.to_string()).await {
                if let Ok(frames) = result.into_value::<Vec<serde_json::Value>>() {
                    for frame in frames {
                        let src = frame["src"].as_str().unwrap_or("");
                        if let Some(frame_html) = frame["html"].as_str() {
                            let replacement = format!(
                                "<div data-iframe-src=\"{}\">{}</div>", src, frame_html
                            );
                            if let Some(pos) = html.find("<iframe") {
                                let end = html[pos..].find("</iframe>")
                                    .map(|e| pos + e + "</iframe>".len())
                                    .or_else(|| html[pos..].find("/>").map(|e| pos + e + "/>".len()));
                                if let Some(end) = end {
                                    html.replace_range(pos..end, &replacement);
                                }
                            }
                        } else if frame["cross_origin"].as_bool() == Some(true) {
                            // Mark cross-origin iframes so Agent knows
                            let marker = format!("[iframe: {}]", src);
                            html = html.replacen("<iframe", &format!("<!-- {} --><iframe", marker), 1);
                        }
                    }
                }
            }

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
    /// All CDP calls wrapped in timeout to prevent hanging on broken WebSocket (Bug 8).
    pub async fn act(&mut self, action: &str, target: &str, value: &str) -> Result<ActResult, BrowserError> {
        let page = self.page.as_ref().ok_or(BrowserError::NotStarted)?;
        let cdp_timeout = std::time::Duration::from_secs(10);

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

        // Bug 8 fix: Timeout all CDP calls to prevent hanging on broken WebSocket
        let result: serde_json::Value = tokio::time::timeout(cdp_timeout, page.evaluate(js))
            .await
            .map_err(|_| BrowserError::Page("CDP timeout — browser may be unresponsive".to_string()))?
            .map_err(|e| BrowserError::Page(e.to_string()))?
            .into_value()
            .map_err(|e| BrowserError::Page(format!("{:?}", e)))?;

        // Wait for potential navigation
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        // Update current URL
        self.current_url = page.evaluate("window.location.href").await
            .ok().and_then(|v| v.into_value::<String>().ok())
            .unwrap_or(self.current_url.clone());

        // Invalidate locator map — DOM may have changed after action.
        // Agent MUST call observe() again before next act().
        self.locator_map.clear();

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
