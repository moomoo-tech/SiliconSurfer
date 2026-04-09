//! Scenario 2: Logic Probe — Agent's development tool.
//!
//! Fast smoke tests after code changes:
//! - HTTP status check
//! - DOM element existence
//! - Text/data presence
//! - DOM snapshot for regression diff

use crate::browser::BrowserPool;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeCheck {
    /// CSS selector to look for (e.g. "#app", "div.stats-chart", "h1")
    pub selector: String,
    /// Optional: expected text content (substring match)
    pub contains_text: Option<String>,
    /// Optional: expected attribute value
    pub attr: Option<String>,
    /// Optional: expected attribute value
    pub attr_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeRequest {
    pub url: String,
    /// DOM checks to run
    #[serde(default)]
    pub checks: Vec<ProbeCheck>,
    /// Check if page contains these text strings anywhere
    #[serde(default)]
    pub contains: Vec<String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// If true, return a DOM snapshot (element tree) for diff
    #[serde(default)]
    pub snapshot: bool,
    /// If true, use headless Chrome to render JS before checking
    #[serde(default)]
    pub render_js: bool,
}

fn default_timeout() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub selector: String,
    pub found: bool,
    pub count: usize,
    /// Text content of first match (truncated)
    pub text: Option<String>,
    /// Attribute value if requested
    pub attr_value: Option<String>,
    /// Did text assertion pass?
    pub text_match: Option<bool>,
    /// Did attr assertion pass?
    pub attr_match: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub url: String,
    pub status: u16,
    pub ok: bool,
    pub elapsed_ms: u64,
    /// Individual check results
    pub checks: Vec<CheckResult>,
    /// Text contains results
    pub contains: HashMap<String, bool>,
    /// All checks passed?
    pub all_passed: bool,
    /// DOM snapshot (if requested)
    pub snapshot: Option<Vec<SnapshotNode>>,
    /// Summary for LLM
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotNode {
    pub tag: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub text: Option<String>,
    pub children_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Browser error: {0}")]
    Browser(String),
}

pub struct Probe {
    client: Client,
    browser: Option<Arc<BrowserPool>>,
}

impl Default for Probe {
    fn default() -> Self {
        Self::new()
    }
}

impl Probe {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
                .cookie_store(true)
                .build()
                .expect("failed to build HTTP client"),
            browser: None,
        }
    }

    pub fn with_browser(mut self, browser: Arc<BrowserPool>) -> Self {
        self.browser = Some(browser);
        self
    }

    pub async fn check(&self, req: ProbeRequest) -> Result<ProbeResult, ProbeError> {
        // T1 fast path: run everything inside Chrome via CDP
        if req.render_js {
            return self.check_in_browser(&req).await;
        }

        let start = Instant::now();
        let (status, is_success, html) = self.fetch_with_reqwest(&req).await?;
        let elapsed_ms = start.elapsed().as_millis() as u64;

        // Run DOM checks
        let check_results: Vec<CheckResult> =
            req.checks.iter().map(|c| run_check(&html, c)).collect();

        // Run text contains checks
        let mut contains_results = HashMap::new();
        let html_lower = html.to_lowercase();
        for text in &req.contains {
            contains_results.insert(text.clone(), html_lower.contains(&text.to_lowercase()));
        }

        // DOM snapshot
        let snapshot = if req.snapshot {
            Some(build_snapshot(&html))
        } else {
            None
        };

        // All passed?
        let checks_passed = check_results.iter().all(|c| {
            let mut ok = c.found;
            if let Some(tm) = c.text_match {
                ok = ok && tm;
            }
            if let Some(am) = c.attr_match {
                ok = ok && am;
            }
            ok
        });
        let contains_passed = contains_results.values().all(|&v| v);
        let all_passed = is_success && checks_passed && contains_passed;

        // Build summary for LLM
        let summary = build_summary(
            status,
            &check_results,
            &contains_results,
            all_passed,
            elapsed_ms,
        );

        Ok(ProbeResult {
            url: req.url,
            status,
            ok: all_passed,
            elapsed_ms,
            checks: check_results,
            contains: contains_results,
            all_passed,
            snapshot,
            summary,
        })
    }

    async fn fetch_with_reqwest(
        &self,
        req: &ProbeRequest,
    ) -> Result<(u16, bool, String), ProbeError> {
        let resp = self
            .client
            .get(&req.url)
            .timeout(std::time::Duration::from_secs(req.timeout_secs))
            .send()
            .await?;
        let status = resp.status().as_u16();
        let is_success = resp.status().is_success();
        let html = resp.text().await?;
        Ok((status, is_success, html))
    }

    /// Run all checks directly inside Chrome via CDP — no HTML transfer.
    /// This is the fast path for T1 probes.
    async fn check_in_browser(&self, req: &ProbeRequest) -> Result<ProbeResult, ProbeError> {
        let start = Instant::now();

        let browser = self
            .browser
            .as_ref()
            .ok_or_else(|| ProbeError::Browser("Browser not available.".to_string()))?;

        // Build JS that runs all checks in one evaluate() call
        let checks_json = serde_json::to_string(&req.checks).unwrap_or_default();
        let contains_json = serde_json::to_string(&req.contains).unwrap_or_default();
        let want_snapshot = req.snapshot;

        let js = format!(
            r#"
            (() => {{
                const checks = {checks_json};
                const contains = {contains_json};
                const wantSnapshot = {want_snapshot};

                // Run selector checks
                const checkResults = checks.map(c => {{
                    const els = document.querySelectorAll(c.selector);
                    const found = els.length > 0;
                    const count = els.length;
                    let text = null;
                    let attrValue = null;
                    if (found) {{
                        text = els[0].textContent.trim().substring(0, 200);
                        if (c.attr) {{
                            attrValue = els[0].getAttribute(c.attr);
                        }}
                    }}
                    let textMatch = null;
                    const wantText = c.contains_text || c.containsText;
                    if (wantText !== undefined && wantText !== null) {{
                        textMatch = found && text !== null && text.toLowerCase().includes(wantText.toLowerCase());
                    }}
                    let attrMatch = null;
                    const wantAttrVal = c.attr_value || c.attrValue;
                    if (c.attr && wantAttrVal !== undefined && wantAttrVal !== null) {{
                        attrMatch = attrValue === wantAttrVal;
                    }}
                    return {{
                        selector: c.selector,
                        found, count, text, attr_value: attrValue,
                        text_match: textMatch, attr_match: attrMatch
                    }};
                }});

                // Run text contains checks
                const bodyText = document.body ? document.body.innerHTML.toLowerCase() : '';
                const containsResults = {{}};
                contains.forEach(t => {{
                    containsResults[t] = bodyText.includes(t.toLowerCase());
                }});

                // DOM snapshot
                let snapshot = null;
                if (wantSnapshot) {{
                    snapshot = [];
                    if (document.body) {{
                        for (const child of document.body.children) {{
                            snapshot.push({{
                                tag: child.tagName.toLowerCase(),
                                id: child.id || null,
                                class: child.className || null,
                                text: child.textContent ? child.textContent.trim().substring(0, 100) : null,
                                children_count: child.children.length
                            }});
                        }}
                    }}
                }}

                return {{ checkResults, containsResults, snapshot, title: document.title }};
            }})()
        "#
        );

        // Navigate and run checks in one shot
        browser
            .ensure_started()
            .await
            .map_err(|e| ProbeError::Browser(e.to_string()))?;

        let guard = browser
            .browser_guard()
            .await
            .map_err(|e| ProbeError::Browser(e.to_string()))?;
        let br = guard
            .as_ref()
            .ok_or_else(|| ProbeError::Browser("Browser not started".to_string()))?;

        let page = br
            .new_page(&req.url)
            .await
            .map_err(|e| ProbeError::Browser(e.to_string()))?;
        // Wait for DOM content loaded (not full network idle — same as Playwright domcontentloaded)
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let result: serde_json::Value = page
            .evaluate(js)
            .await
            .map_err(|e| ProbeError::Browser(e.to_string()))?
            .into_value()
            .map_err(|e| ProbeError::Browser(format!("JSON parse error: {:?}", e)))?;

        let _ = page.close().await;
        drop(guard);

        let elapsed_ms = start.elapsed().as_millis() as u64;

        // Parse JS results into our types
        let check_results: Vec<CheckResult> = result["checkResults"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| CheckResult {
                        selector: v["selector"].as_str().unwrap_or("").to_string(),
                        found: v["found"].as_bool().unwrap_or(false),
                        count: v["count"].as_u64().unwrap_or(0) as usize,
                        text: v["text"].as_str().map(|s| s.to_string()),
                        attr_value: v["attr_value"].as_str().map(|s| s.to_string()),
                        text_match: v["text_match"].as_bool(),
                        attr_match: v["attr_match"].as_bool(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut contains_results = HashMap::new();
        if let Some(obj) = result["containsResults"].as_object() {
            for (k, v) in obj {
                contains_results.insert(k.clone(), v.as_bool().unwrap_or(false));
            }
        }

        let snapshot = if want_snapshot {
            result["snapshot"].as_array().map(|arr| {
                arr.iter()
                    .map(|v| SnapshotNode {
                        tag: v["tag"].as_str().unwrap_or("").to_string(),
                        id: v["id"].as_str().map(|s| s.to_string()),
                        class: v["class"].as_str().map(|s| s.to_string()),
                        text: v["text"].as_str().map(|s| s.to_string()),
                        children_count: v["children_count"].as_u64().unwrap_or(0) as usize,
                    })
                    .collect()
            })
        } else {
            None
        };

        let checks_passed = check_results.iter().all(|c| {
            let mut ok = c.found;
            if let Some(tm) = c.text_match {
                ok = ok && tm;
            }
            if let Some(am) = c.attr_match {
                ok = ok && am;
            }
            ok
        });
        let contains_passed = contains_results.values().all(|&v| v);
        let all_passed = checks_passed && contains_passed;

        let summary = build_summary(
            200,
            &check_results,
            &contains_results,
            all_passed,
            elapsed_ms,
        );

        Ok(ProbeResult {
            url: req.url.clone(),
            status: 200,
            ok: all_passed,
            elapsed_ms,
            checks: check_results,
            contains: contains_results,
            all_passed,
            snapshot,
            summary,
        })
    }
}

/// Run a single DOM check using scraper (reliable CSS selector matching)
fn run_check(html: &str, check: &ProbeCheck) -> CheckResult {
    let doc = scraper::Html::parse_document(html);
    let sel = match scraper::Selector::parse(&check.selector) {
        Ok(s) => s,
        Err(_) => {
            return CheckResult {
                selector: check.selector.clone(),
                found: false,
                count: 0,
                text: Some(format!("Invalid selector: {}", check.selector)),
                attr_value: None,
                text_match: None,
                attr_match: None,
            };
        }
    };

    let matches: Vec<_> = doc.select(&sel).collect();
    let count = matches.len();
    let found = count > 0;

    let text = matches.first().map(|el| {
        let t: String = el.text().collect::<Vec<_>>().join(" ");
        let t = t.trim().to_string();
        if t.len() > 200 {
            format!("{}...", &t[..200])
        } else {
            t
        }
    });

    let attr_value = if let (Some(attr), Some(el)) = (&check.attr, matches.first()) {
        el.value().attr(attr).map(|v| v.to_string())
    } else {
        None
    };

    let text_match = check.contains_text.as_ref().map(|expected| {
        text.as_ref()
            .map(|t| t.to_lowercase().contains(&expected.to_lowercase()))
            .unwrap_or(false)
    });

    let attr_match = if let (Some(_), Some(expected_val)) = (&check.attr, &check.attr_value) {
        Some(
            attr_value
                .as_ref()
                .map(|v| v == expected_val)
                .unwrap_or(false),
        )
    } else {
        None
    };

    CheckResult {
        selector: check.selector.clone(),
        found,
        count,
        text,
        attr_value,
        text_match,
        attr_match,
    }
}

/// Build a lightweight DOM snapshot — top-level structure only
fn build_snapshot(html: &str) -> Vec<SnapshotNode> {
    let doc = scraper::Html::parse_document(html);
    let body_sel = scraper::Selector::parse("body").unwrap();
    let body = match doc.select(&body_sel).next() {
        Some(b) => b,
        None => return vec![],
    };

    // Only direct children of body, and their immediate structure
    let mut nodes = Vec::new();
    for child in body.children() {
        if let Some(el) = scraper::ElementRef::wrap(child) {
            let tag = el.value().name().to_string();
            let id = el.value().attr("id").map(|s| s.to_string());
            let class = el.value().attr("class").map(|s| s.to_string());
            let text_content: String = el.text().collect::<Vec<_>>().join(" ");
            let text = if text_content.trim().is_empty() {
                None
            } else {
                let t = text_content.trim();
                Some(if t.len() > 100 {
                    format!("{}...", &t[..100])
                } else {
                    t.to_string()
                })
            };
            let children_count = el
                .children()
                .filter(|c| scraper::ElementRef::wrap(*c).is_some())
                .count();

            nodes.push(SnapshotNode {
                tag,
                id,
                class,
                text,
                children_count,
            });
        }
    }
    nodes
}

fn build_summary(
    status: u16,
    checks: &[CheckResult],
    contains: &HashMap<String, bool>,
    all_passed: bool,
    elapsed_ms: u64,
) -> String {
    let mut lines = Vec::new();

    if all_passed {
        lines.push(format!("✓ ALL PASSED ({elapsed_ms}ms)"));
    } else {
        lines.push(format!("✗ FAILED ({elapsed_ms}ms)"));
    }

    lines.push(format!("  HTTP {status}"));

    for c in checks {
        let status_icon = if c.found { "✓" } else { "✗" };
        let mut detail = format!("  {status_icon} `{}` ", c.selector);
        if c.found {
            detail.push_str(&format!("found ({})", c.count));
        } else {
            detail.push_str("NOT FOUND");
        }
        if let Some(tm) = c.text_match {
            detail.push_str(if tm { " text:✓" } else { " text:✗" });
        }
        if let Some(am) = c.attr_match {
            detail.push_str(if am { " attr:✓" } else { " attr:✗" });
        }
        lines.push(detail);
    }

    for (text, found) in contains {
        let icon = if *found { "✓" } else { "✗" };
        lines.push(format!("  {icon} contains \"{text}\""));
    }

    lines.join("\n")
}
