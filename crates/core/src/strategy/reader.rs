//! Reader / LLM-Friendly strategy.
//!
//! Aggressive noise removal, clean Markdown output.
//! Conservative links: only http:// and /path (no bare relative).

use lol_html::{element, rewrite_str, RewriteStrSettings};
use lol_html::html_content::ContentType::Text;

use crate::profiles;

use super::{extract_origin, finalize, reader_noise_selectors, strip_tags};

pub fn distill(html: &str, base_url: Option<&str>) -> String {
    // First: apply site-specific profile noise removal (dynamic selectors)
    let cleaned = if let Some(url) = base_url {
        apply_profile_noise(html, url)
    } else {
        html.to_string()
    };
    // Then: standard Reader rewrite (static selectors)
    let rewritten = rewrite(&cleaned, base_url);
    let text = strip_tags(&rewritten);
    finalize(&text)
}

/// Apply site-specific noise removal from profiles.toml.
/// Uses scraper (DOM) for dynamic selectors since lol_html element! needs &'static str.
fn apply_profile_noise(html: &str, url: &str) -> String {
    let extra = profiles::extra_noise_for_url(url);
    if extra.is_empty() {
        return html.to_string();
    }

    // Parse selectors, collect matching elements' HTML, then remove them
    let doc = scraper::Html::parse_document(html);
    let mut noise_fragments: Vec<String> = Vec::new();

    for sel_str in &extra {
        if let Ok(sel) = scraper::Selector::parse(sel_str) {
            for el in doc.select(&sel) {
                noise_fragments.push(el.html());
            }
        }
    }

    if noise_fragments.is_empty() {
        return html.to_string();
    }

    // Remove noise fragments from HTML
    let mut result = html.to_string();
    for frag in &noise_fragments {
        if frag.len() > 10 { // Skip tiny fragments that might cause false matches
            result = result.replacen(frag, "", 1);
        }
    }
    result
}

/// Get combined noise selectors: base + site-specific profile.
fn get_noise_selectors(base_url: Option<&str>) -> Vec<String> {
    let mut selectors: Vec<String> = reader_noise_selectors().iter().map(|s| s.to_string()).collect();
    if let Some(url) = base_url {
        for extra in profiles::extra_noise_for_url(url) {
            selectors.push(extra.to_string());
        }
    }
    selectors
}

pub fn to_text(html: &str) -> String {
    let rewritten = rewrite_text_only(html);
    let text = strip_tags(&rewritten);
    finalize(&text)
}

fn rewrite(html: &str, base_url: Option<&str>) -> String {
    let base_origin = base_url.and_then(extract_origin);

    let mut handlers = Vec::new();

    // Noise removal — base selectors + site-specific profile
    for sel in reader_noise_selectors() {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }
    // Site-specific noise already removed in apply_profile_noise() above.

    // Headings
    handlers.push(element!("h1", |el| { el.before("\n\n# ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h2", |el| { el.before("\n\n## ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h3", |el| { el.before("\n\n### ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h4, h5, h6", |el| { el.before("\n\n#### ", Text); el.after("\n\n", Text); Ok(()) }));

    // Paragraphs
    handlers.push(element!("p", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));

    // Links — conservative: only http + /path
    let bo = base_origin.clone();
    handlers.push(element!("a[href]", move |el| {
        let href = el.get_attribute("href").unwrap_or_default();
        let full_url = if href.starts_with("http") {
            Some(href)
        } else if href.starts_with('/') {
            bo.as_ref().map(|o| format!("{}{}", o, href))
        } else {
            None
        };
        if let Some(url) = full_url {
            el.before("\x01L", Text);
            el.after(&format!("\x02{}\x03", url), Text);
        }
        Ok(())
    }));

    // Formatting
    handlers.push(element!("strong, b", |el| { el.before("**", Text); el.after("**", Text); Ok(()) }));
    handlers.push(element!("em, i", |el| { el.before("_", Text); el.after("_", Text); Ok(()) }));
    handlers.push(element!("code", |el| { el.before("`", Text); el.after("`", Text); Ok(()) }));
    handlers.push(element!("pre", |el| { el.before("\n\x04", Text); el.after("\x05\n", Text); Ok(()) }));
    handlers.push(element!("li", |el| { el.before("\n- ", Text); Ok(()) }));
    handlers.push(element!("tr", |el| { el.after("\n", Text); Ok(()) }));
    handlers.push(element!("div, section, article, blockquote", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));
    handlers.push(element!("hr", |el| { el.before("\n\n---\n\n", Text); Ok(()) }));
    handlers.push(element!("br", |el| { el.before("\n", Text); Ok(()) }));

    // Images — extract alt text + src as markdown image placeholder
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

fn rewrite_text_only(html: &str) -> String {
    let mut handlers = Vec::new();
    for sel in reader_noise_selectors() {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }
    handlers.push(element!("p, div, tr, li, h1, h2, h3, h4, h5, h6, br", |el| { el.before("\n", Text); Ok(()) }));
    handlers.push(element!("pre", |el| { el.before("\n\x04", Text); el.after("\x05\n", Text); Ok(()) }));

    rewrite_str(html, RewriteStrSettings {
        element_content_handlers: handlers,
        ..RewriteStrSettings::new()
    }).unwrap_or_else(|_| html.to_string())
}
