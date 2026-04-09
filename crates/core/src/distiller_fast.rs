//! Fast streaming distiller using lol_html.
//!
//! Two-pass processing:
//! 1. strip_noise + inject_markdown (single lol_html rewrite)
//! 2. extract_text + decode_entities + dedup (single string scan)

use lol_html::{element, rewrite_str, RewriteStrSettings};

pub struct FastDistiller;

impl FastDistiller {
    pub fn extract_title(html: &str) -> Option<String> {
        let start = html.find("<title")?.checked_add(html[html.find("<title")?..].find('>')?)?;
        let start = start + 1;
        let end = html[start..].find("</title>").map(|i| i + start)?;
        let title = html[start..end].trim();
        if title.is_empty() { None } else { Some(decode_entities_fast(title)) }
    }

    pub fn to_markdown(html: &str) -> String {
        Self::to_markdown_with_base(html, None)
    }

    pub fn to_markdown_with_base(html: &str, base_url: Option<&str>) -> String {
        // Pass 1: lol_html — strip noise + inject markdown markers (single rewrite)
        let rewritten = rewrite_combined(html, base_url, true);
        // Pass 2: pure string — strip tags, decode entities, resolve links, dedup
        let text = strip_tags(&rewritten);
        finalize(&text)
    }

    pub fn to_text(html: &str) -> String {
        let rewritten = rewrite_combined(html, None, false);
        let text = strip_tags(&rewritten);
        finalize(&text)
    }
}

/// Single lol_html pass: noise removal + markdown injection combined.
fn rewrite_combined(html: &str, base_url: Option<&str>, markdown: bool) -> String {
    use lol_html::html_content::ContentType::Text;

    let base_origin = base_url.and_then(extract_origin);

    let noise_selectors = [
        "script", "style", "nav", "footer", "header", "iframe",
        "noscript", "svg", "form", "button", "input", "select", "textarea",
        "head", "img", "picture", "video", "audio", "canvas",
        "[class*='ad-']", "[class*='ads-']",
        "[class*='cookie-']", "[class*='cookie_']",
        ".popup", ".modal", "[class*='-popup']", "[class*='-modal']",
        ".social-share", ".share-buttons", ".sharing",
        ".newsletter", ".subscribe",
        "[class*='-banner'][class*='ad']",
        "[role='navigation']", "[role='complementary']",
        "[role='search']", "[aria-hidden='true']",
    ];

    // Build noise removal handlers
    let mut handlers = Vec::new();
    for sel in &noise_selectors {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }

    if markdown {
        handlers.push(element!("h1", |el| { el.before("\n\n# ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("h2", |el| { el.before("\n\n## ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("h3", |el| { el.before("\n\n### ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("h4, h5, h6", |el| { el.before("\n\n#### ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("p", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));
        handlers.push(element!("a[href]", |el| {
            let href = el.get_attribute("href").unwrap_or_default();
            let full_url = if href.starts_with("http") {
                Some(href)
            } else if href.starts_with('/') {
                base_origin.as_ref().map(|o| format!("{}{}", o, href))
            } else {
                None
            };
            if let Some(url) = full_url {
                el.before("\x01L", Text);
                el.after(&format!("\x02{}\x03", url), Text);
            }
            Ok(())
        }));
        handlers.push(element!("strong, b", |el| { el.before("**", Text); el.after("**", Text); Ok(()) }));
        handlers.push(element!("em, i", |el| { el.before("_", Text); el.after("_", Text); Ok(()) }));
        handlers.push(element!("code", |el| { el.before("`", Text); el.after("`", Text); Ok(()) }));
        handlers.push(element!("pre", |el| { el.before("\n\x04", Text); el.after("\x05\n", Text); Ok(()) }));
        handlers.push(element!("li", |el| { el.before("\n- ", Text); Ok(()) }));
        handlers.push(element!("tr", |el| { el.after("\n", Text); Ok(()) }));
        handlers.push(element!("div, section, article, blockquote", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));
        handlers.push(element!("hr", |el| { el.before("\n\n---\n\n", Text); Ok(()) }));
        handlers.push(element!("br", |el| { el.before("\n", Text); Ok(()) }));
    } else {
        handlers.push(element!("p, div, tr, li, h1, h2, h3, h4, h5, h6, br", |el| { el.before("\n", Text); Ok(()) }));
        handlers.push(element!("pre", |el| { el.before("\n\x04", Text); el.after("\x05\n", Text); Ok(()) }));
    }

    rewrite_str(html, RewriteStrSettings {
        element_content_handlers: handlers,
        ..RewriteStrSettings::new()
    }).unwrap_or_else(|_| html.to_string())
}

/// Strip all HTML tags, preserving injected markers.
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Collapse whitespace in a single line — state machine, zero allocation.
fn collapse_whitespace(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last_was_space = true; // true = skip leading spaces
    for c in line.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    // Trim trailing space
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Final pass: decode entities, resolve links, dedup — all in one scan.
fn finalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut seen = std::collections::HashSet::with_capacity(256);
    let mut blank_count = 0u32;
    let mut in_pre = false;

    // First resolve control-char markers and decode entities
    let resolved = resolve_markers(text);

    for line in resolved.lines() {
        let trimmed = line.trim();

        // Pre block boundaries
        if trimmed == "```" && !in_pre {
            in_pre = true;
            blank_count = 0;
            result.push_str("```\n");
            continue;
        }
        if trimmed == "```" && in_pre {
            in_pre = false;
            result.push_str("```\n");
            continue;
        }

        if in_pre {
            // Preserve whitespace inside pre
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Collapse whitespace — state machine, zero allocation
        let clean = collapse_whitespace(line);

        if clean.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
            continue;
        }

        blank_count = 0;

        // Dedup (skip short lines like "1." or "- ")
        if clean.len() > 10 && !seen.insert(hash_fast(&clean)) {
            continue;
        }

        result.push_str(&clean);
        result.push('\n');
    }

    result.trim().to_string()
}

/// Resolve control-char markers into markdown + decode HTML entities.
/// Single pass O(n).
fn resolve_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        match chars[i] {
            // \x01L = link start, collect text until \x02, then url until \x03
            '\x01' if i + 1 < len && chars[i + 1] == 'L' => {
                i += 2; // skip \x01L
                let text_start = i;
                // Find \x02
                while i < len && chars[i] != '\x02' { i += 1; }
                let link_text: String = chars[text_start..i].iter().collect();
                let link_text = link_text.trim();
                i += 1; // skip \x02
                // Find \x03
                let url_start = i;
                while i < len && chars[i] != '\x03' { i += 1; }
                let url: String = chars[url_start..i].iter().collect();
                i += 1; // skip \x03

                if link_text.is_empty() {
                    // Skip empty links
                } else {
                    out.push('[');
                    out.push_str(link_text);
                    out.push_str("](");
                    out.push_str(&url);
                    out.push_str(") ");
                }
            }
            // \x04 = pre start
            '\x04' => {
                out.push_str("```\n");
                i += 1;
            }
            // \x05 = pre end
            '\x05' => {
                out.push_str("\n```");
                i += 1;
            }
            // HTML entity: &...;
            '&' => {
                // Try to decode entity
                let rest: String = chars[i..std::cmp::min(i + 12, len)].iter().collect();
                if let Some(semi) = rest.find(';') {
                    let entity = &rest[..semi + 1];
                    if let Some(decoded) = decode_entity(entity) {
                        out.push_str(&decoded);
                        i += semi + 1;
                        continue;
                    }
                }
                out.push('&');
                i += 1;
            }
            ch => {
                out.push(ch);
                i += 1;
            }
        }
    }

    out
}

/// Decode a single HTML entity. Returns None if not recognized.
fn decode_entity(entity: &str) -> Option<String> {
    // Named entities
    let decoded = match entity {
        "&amp;" => "&",
        "&lt;" => "<",
        "&gt;" => ">",
        "&quot;" => "\"",
        "&apos;" => "'",
        "&nbsp;" => " ",
        "&ndash;" => "\u{2013}",
        "&mdash;" => "\u{2014}",
        "&laquo;" => "\u{00AB}",
        "&raquo;" => "\u{00BB}",
        "&copy;" => "\u{00A9}",
        "&reg;" => "\u{00AE}",
        "&trade;" => "\u{2122}",
        "&hellip;" => "\u{2026}",
        "&bull;" => "\u{2022}",
        "&middot;" => "\u{00B7}",
        "&larr;" => "\u{2190}",
        "&rarr;" => "\u{2192}",
        "&uarr;" => "\u{2191}",
        "&darr;" => "\u{2193}",
        _ => {
            // Numeric: &#NNN; or &#xHHH;
            if entity.starts_with("&#") && entity.ends_with(';') {
                let inner = &entity[2..entity.len() - 1];
                let code = if inner.starts_with('x') || inner.starts_with('X') {
                    u32::from_str_radix(&inner[1..], 16).ok()
                } else {
                    inner.parse::<u32>().ok()
                };
                if let Some(ch) = code.and_then(char::from_u32) {
                    return Some(ch.to_string());
                }
            }
            return None;
        }
    };
    Some(decoded.to_string())
}

/// Fast string hash for dedup (avoid cloning into HashSet<String>)
fn hash_fast(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn extract_origin(url: &str) -> Option<String> {
    let scheme_end = url.find("://")?;
    let after_scheme = &url[scheme_end + 3..];
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    Some(url[..scheme_end + 3 + host_end].to_string())
}

fn decode_entities_fast(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '&' {
            let rest: String = chars[i..std::cmp::min(i + 12, len)].iter().collect();
            if let Some(semi) = rest.find(';') {
                let entity = &rest[..semi + 1];
                if let Some(decoded) = decode_entity(entity) {
                    out.push_str(&decoded);
                    i += semi + 1;
                    continue;
                }
            }
            out.push('&');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title() {
        let html = "<html><head><title>Hello World</title></head><body></body></html>";
        assert_eq!(FastDistiller::extract_title(html), Some("Hello World".to_string()));
    }

    #[test]
    fn test_noise_removal() {
        let html = "<html><body><nav>Menu</nav><p>Content here</p><footer>Foot</footer></body></html>";
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("Content here"), "got: {md}");
        assert!(!md.contains("Menu"), "got: {md}");
        assert!(!md.contains("Foot"), "got: {md}");
    }

    #[test]
    fn test_headings() {
        let html = "<h1>Title</h1><p>Text</p>";
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("# Title"), "got: {md}");
        assert!(md.contains("Text"), "got: {md}");
    }

    #[test]
    fn test_links() {
        let html = r#"<p><a href="https://example.com">Click</a></p>"#;
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("[Click](https://example.com)"), "got: {md}");
    }

    #[test]
    fn test_relative_links() {
        let html = r#"<p><a href="/foo/bar">Link</a></p>"#;
        let md = FastDistiller::to_markdown_with_base(html, Some("https://example.com/page"));
        assert!(md.contains("[Link](https://example.com/foo/bar)"), "got: {md}");
    }

    #[test]
    fn test_relative_links_no_base() {
        let html = r#"<p><a href="/foo/bar">Link</a></p>"#;
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("Link"), "got: {md}");
        assert!(!md.contains("/foo/bar"), "got: {md}");
    }

    #[test]
    fn test_empty_links_skipped() {
        let html = r#"<a href="https://example.com"></a><a href="https://real.com">Real</a>"#;
        let md = FastDistiller::to_markdown(html);
        assert!(!md.contains("[]("), "got: {md}");
        assert!(md.contains("[Real](https://real.com)"), "got: {md}");
    }

    #[test]
    fn test_html_entities_decoded() {
        let html = "<p>Hello&#160;world &amp; foo&#91;bar&#93;</p>";
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("& foo[bar]"), "got: {md}");
        assert!(!md.contains("&#"), "got: {md}");
    }

    #[test]
    fn test_title_not_in_body() {
        let html = "<html><head><title>My Title</title></head><body><p>Content</p></body></html>";
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("Content"), "got: {md}");
        assert!(!md.contains("My Title"), "got: {md}");
    }

    #[test]
    fn test_pre_preserves_whitespace() {
        let html = r#"<pre><code>fn main() {
    println!("hello");
    let x = 42;
}</code></pre>"#;
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("```"), "got: {md}");
        assert!(md.contains("    println!"), "got: {md}");
        assert!(md.contains("    let x"), "got: {md}");
    }

    #[test]
    fn test_pre_not_collapsed() {
        let html = "<pre>line1\nline2\nline3</pre>";
        let md = FastDistiller::to_markdown(html);
        assert!(md.contains("line1") && md.contains("line2") && md.contains("line3"), "got: {md}");
    }

    #[test]
    fn test_dedup() {
        let html = "<p>Same line here</p><p>Same line here</p><p>Unique</p>";
        let md = FastDistiller::to_markdown(html);
        assert_eq!(md.matches("Same line here").count(), 1, "got: {md}");
        assert!(md.contains("Unique"), "got: {md}");
    }

    #[test]
    fn test_text_mode() {
        let html = "<html><body><h1>Title</h1><p>Hello <b>world</b></p></body></html>";
        let text = FastDistiller::to_text(html);
        assert!(text.contains("Title"), "got: {text}");
        assert!(text.contains("Hello"), "got: {text}");
        assert!(text.contains("world"), "got: {text}");
        assert!(!text.contains("<"), "got: {text}");
    }
}
