//! Operator strategy — preserve UI elements, annotate with @e references.
//!
//! Every interactive element gets a unique @eN ID that Agent can reference.
//! Also injects `data-agent-id="eN"` into the HTML for Playwright targeting.
//! Resolves all links including bare relative (item?id=123).

use lol_html::{element, rewrite_str, RewriteStrSettings};
use lol_html::html_content::ContentType::Text;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::{extract_origin, finalize, operator_noise_selectors, resolve_href, strip_tags};

/// Distill with @e element references. Returns (markdown, element_map_json).
pub fn distill(html: &str, base_url: Option<&str>) -> String {
    let rewritten = rewrite(html, base_url);
    let text = strip_tags(&rewritten);
    finalize(&text)
}

/// Distill and also return the element map as JSON for Playwright targeting.
pub fn distill_with_map(html: &str, base_url: Option<&str>) -> (String, String) {
    let counter = Arc::new(AtomicUsize::new(0));
    let rewritten = rewrite_with_counter(html, base_url, counter.clone());
    let text = strip_tags(&rewritten);
    let markdown = finalize(&text);

    // Element map is embedded in the markdown as @eN references
    // Playwright uses: page.locator('[data-agent-id="eN"]')
    let total = counter.load(Ordering::Relaxed);
    let map = serde_json::json!({
        "total_elements": total,
        "selector_pattern": "[data-agent-id=\"eN\"]",
        "usage": "Replace eN with the @e reference from the markdown"
    });

    (markdown, map.to_string())
}

fn rewrite(html: &str, base_url: Option<&str>) -> String {
    let counter = Arc::new(AtomicUsize::new(0));
    rewrite_with_counter(html, base_url, counter)
}

fn rewrite_with_counter(html: &str, base_url: Option<&str>, counter: Arc<AtomicUsize>) -> String {
    let base_origin = base_url.and_then(extract_origin);

    let mut handlers = Vec::new();

    // Minimal noise removal
    for sel in operator_noise_selectors() {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }

    // Headings
    handlers.push(element!("h1", |el| { el.before("\n\n# ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h2", |el| { el.before("\n\n## ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h3", |el| { el.before("\n\n### ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("p", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));

    // Links — @eN reference + resolve URL
    let bo = base_origin.clone();
    let bu = base_url.map(|s| s.to_string());
    let c = counter.clone();
    handlers.push(element!("a[href]", move |el| {
        let href = el.get_attribute("href").unwrap_or_default();
        let full_url = resolve_href(&href, bo.as_deref(), bu.as_deref()).unwrap_or(href);
        let id = c.fetch_add(1, Ordering::Relaxed) + 1;
        let eid = format!("e{}", id);
        el.set_attribute("data-agent-id", &eid).ok();
        el.before(&format!("@{} \x01L", eid), Text);
        el.after(&format!("\x02{}\x03", full_url), Text);
        Ok(())
    }));

    // Buttons — @eN
    let c = counter.clone();
    handlers.push(element!("button", move |el| {
        let id = c.fetch_add(1, Ordering::Relaxed) + 1;
        let eid = format!("e{}", id);
        el.set_attribute("data-agent-id", &eid).ok();
        el.before(&format!("@{} [Button: ", eid), Text);
        el.after("]", Text);
        Ok(())
    }));

    // Input fields — @eN
    let c = counter.clone();
    handlers.push(element!("input", move |el| {
        let id = c.fetch_add(1, Ordering::Relaxed) + 1;
        let eid = format!("e{}", id);
        el.set_attribute("data-agent-id", &eid).ok();
        let itype = el.get_attribute("type").unwrap_or_else(|| "text".to_string());
        let name = el.get_attribute("name").unwrap_or_else(|| "?".to_string());
        let placeholder = el.get_attribute("placeholder").unwrap_or_default();
        let label = if placeholder.is_empty() {
            format!("@{} [Input: type={} name={}]", eid, itype, name)
        } else {
            format!("@{} [Input: type={} name={} placeholder=\"{}\"]", eid, itype, name, placeholder)
        };
        el.before(&label, Text);
        Ok(())
    }));

    // Select — @eN
    let c = counter.clone();
    handlers.push(element!("select", move |el| {
        let id = c.fetch_add(1, Ordering::Relaxed) + 1;
        let eid = format!("e{}", id);
        el.set_attribute("data-agent-id", &eid).ok();
        let name = el.get_attribute("name").unwrap_or_default();
        el.before(&format!("@{} [Select: {}]", eid, name), Text);
        Ok(())
    }));

    // Textarea — @eN
    let c = counter.clone();
    handlers.push(element!("textarea", move |el| {
        let id = c.fetch_add(1, Ordering::Relaxed) + 1;
        let eid = format!("e{}", id);
        el.set_attribute("data-agent-id", &eid).ok();
        let name = el.get_attribute("name").unwrap_or_default();
        el.before(&format!("@{} [Textarea: {}]", eid, name), Text);
        Ok(())
    }));

    // Forms — action + method (no @eN, it's a container)
    handlers.push(element!("form", |el| {
        let action = el.get_attribute("action").unwrap_or_default();
        let method = el.get_attribute("method").unwrap_or_else(|| "GET".to_string());
        el.before(&format!("\n[Form: {} {}]\n", method.to_uppercase(), action), Text);
        el.after("\n[/Form]\n", Text);
        Ok(())
    }));

    // Nav sections
    handlers.push(element!("nav", |el| {
        el.before("\n[Nav] ", Text);
        el.after("\n", Text);
        Ok(())
    }));

    // Formatting
    handlers.push(element!("strong, b", |el| { el.before("**", Text); el.after("**", Text); Ok(()) }));
    handlers.push(element!("li", |el| { el.before("\n- ", Text); Ok(()) }));
    handlers.push(element!("tr", |el| { el.after("\n", Text); Ok(()) }));
    handlers.push(element!("div, section, article", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));
    handlers.push(element!("hr", |el| { el.before("\n---\n", Text); Ok(()) }));
    handlers.push(element!("br", |el| { el.before("\n", Text); Ok(()) }));
    handlers.push(element!("pre", |el| { el.before("\n\x04", Text); el.after("\x05\n", Text); Ok(()) }));

    // Images
    handlers.push(element!("img", |el| {
        let alt = el.get_attribute("alt").unwrap_or_default();
        let src = el.get_attribute("data-src")
            .or_else(|| el.get_attribute("data-original"))
            .or_else(|| el.get_attribute("src"))
            .unwrap_or_default();
        if !alt.is_empty() {
            if !src.is_empty() {
                el.before(&format!("\n[image: {}]({})\n", alt, src), Text);
            } else {
                el.before(&format!("\n[image: {}]\n", alt), Text);
            }
        } else if !src.is_empty() && !src.starts_with("data:") {
            el.before(&format!("\n[image]({})\n", src), Text);
        }
        el.remove();
        Ok(())
    }));

    rewrite_str(html, RewriteStrSettings {
        element_content_handlers: handlers,
        ..RewriteStrSettings::new()
    }).unwrap_or_else(|_| html.to_string())
}
