//! Reader / LLM-Friendly strategy.
//!
//! Aggressive noise removal, clean Markdown output.
//! Conservative links: only http:// and /path (no bare relative).

use lol_html::{element, rewrite_str, RewriteStrSettings};
use lol_html::html_content::ContentType::Text;

use super::{extract_origin, finalize, reader_noise_selectors, strip_tags};

pub fn distill(html: &str, base_url: Option<&str>) -> String {
    let rewritten = rewrite(html, base_url);
    let text = strip_tags(&rewritten);
    finalize(&text)
}

pub fn to_text(html: &str) -> String {
    let rewritten = rewrite_text_only(html);
    let text = strip_tags(&rewritten);
    finalize(&text)
}

fn rewrite(html: &str, base_url: Option<&str>) -> String {
    let base_origin = base_url.and_then(extract_origin);

    let mut handlers = Vec::new();

    // Noise removal
    for sel in reader_noise_selectors() {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }

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
