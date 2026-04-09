//! Operator strategy — preserve UI elements, annotate interactive nodes.
//!
//! Keeps forms, buttons, inputs, nav. Annotates with [Button:], [Form:], [Input:], [Nav].
//! Resolves all links including bare relative (item?id=123).

use lol_html::{element, rewrite_str, RewriteStrSettings};
use lol_html::html_content::ContentType::Text;

use super::{extract_origin, finalize, operator_noise_selectors, resolve_href, strip_tags};

pub fn distill(html: &str, base_url: Option<&str>) -> String {
    let rewritten = rewrite(html, base_url);
    let text = strip_tags(&rewritten);
    finalize(&text)
}

fn rewrite(html: &str, base_url: Option<&str>) -> String {
    let base_origin = base_url.and_then(extract_origin);

    let mut handlers = Vec::new();

    // Minimal noise removal — keep UI elements
    for sel in operator_noise_selectors() {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }

    // Headings
    handlers.push(element!("h1", |el| { el.before("\n\n# ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h2", |el| { el.before("\n\n## ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h3", |el| { el.before("\n\n### ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("p", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));

    // Links — resolve ALL relative URLs (including bare relative)
    let bo = base_origin.clone();
    let bu = base_url.map(|s| s.to_string());
    handlers.push(element!("a[href]", move |el| {
        let href = el.get_attribute("href").unwrap_or_default();
        let full_url = resolve_href(&href, bo.as_deref(), bu.as_deref()).unwrap_or(href);
        el.before("\x01L", Text);
        el.after(&format!("\x02{}\x03", full_url), Text);
        Ok(())
    }));

    // Buttons — annotate
    handlers.push(element!("button", |el| {
        el.before("[Button: ", Text);
        el.after("]", Text);
        Ok(())
    }));

    // Input fields — show type and name
    handlers.push(element!("input", |el| {
        let itype = el.get_attribute("type").unwrap_or_else(|| "text".to_string());
        let name = el.get_attribute("name").unwrap_or_else(|| "?".to_string());
        let placeholder = el.get_attribute("placeholder").unwrap_or_default();
        let label = if placeholder.is_empty() {
            format!("[Input: type={} name={}]", itype, name)
        } else {
            format!("[Input: type={} name={} placeholder=\"{}\"]", itype, name, placeholder)
        };
        el.before(&label, Text);
        Ok(())
    }));

    // Forms — show action + method
    handlers.push(element!("form", |el| {
        let action = el.get_attribute("action").unwrap_or_default();
        let method = el.get_attribute("method").unwrap_or_else(|| "GET".to_string());
        el.before(&format!("\n[Form: {} {}]\n", method.to_uppercase(), action), Text);
        el.after("\n[/Form]\n", Text);
        Ok(())
    }));

    // Select
    handlers.push(element!("select", |el| {
        let name = el.get_attribute("name").unwrap_or_default();
        el.before(&format!("[Select: {}]", name), Text);
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

    // Images — preserve alt text + src
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
