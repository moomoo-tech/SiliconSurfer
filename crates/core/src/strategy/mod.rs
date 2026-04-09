//! Strategy pattern for distillation modes.
//!
//! Each mode is a separate module with its own lol_html handler setup.
//! Shared utilities live here.

pub mod data;
pub mod developer;
pub mod operator;
pub mod reader;
pub mod spider;

use std::collections::HashSet;

/// Resolve an href to a full URL.
pub fn resolve_href(
    href: &str,
    base_origin: Option<&str>,
    base_url: Option<&str>,
) -> Option<String> {
    if href.starts_with("http") {
        Some(href.to_string())
    } else if href.starts_with("javascript:")
        || href.starts_with("mailto:")
        || href == "#"
        || href.is_empty()
    {
        None
    } else if href.starts_with('/') {
        base_origin.map(|o| format!("{}{}", o, href))
    } else {
        // Bare relative: item?id=123
        base_url.map(|base| {
            let base_dir = base.rfind('/').map(|i| &base[..i + 1]).unwrap_or(base);
            format!("{}{}", base_dir, href)
        })
    }
}

/// Extract origin from URL: "https://example.com/path" → "https://example.com"
pub fn extract_origin(url: &str) -> Option<String> {
    let scheme_end = url.find("://")?;
    let after = &url[scheme_end + 3..];
    let host_end = after.find('/').unwrap_or(after.len());
    Some(url[..scheme_end + 3 + host_end].to_string())
}

/// Strip all HTML tags, preserving injected control chars.
pub fn strip_tags(html: &str) -> String {
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

/// Collapse whitespace — zero-allocation state machine.
pub fn collapse_whitespace(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last_was_space = true;
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
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Resolve control-char markers + decode HTML entities in one O(n) pass.
pub fn resolve_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        match chars[i] {
            // \x01L...text...\x02url\x03 → [text](url)
            '\x01' if i + 1 < len && chars[i + 1] == 'L' => {
                i += 2;
                let text_start = i;
                while i < len && chars[i] != '\x02' {
                    i += 1;
                }
                let link_text: String = chars[text_start..i].iter().collect();
                let link_text = link_text.trim();
                i += 1;
                let url_start = i;
                while i < len && chars[i] != '\x03' {
                    i += 1;
                }
                let url: String = chars[url_start..i].iter().collect();
                i += 1;
                if !link_text.is_empty() {
                    out.push('[');
                    out.push_str(link_text);
                    out.push_str("](");
                    out.push_str(&url);
                    out.push_str(") ");
                }
            }
            '\x04' => {
                out.push_str("```\n");
                i += 1;
            }
            '\x05' => {
                out.push_str("\n```");
                i += 1;
            }
            '&' => {
                let rest: String = chars[i..std::cmp::min(i + 12, len)].iter().collect();
                if let Some(semi) = rest.find(';')
                    && let Some(decoded) = decode_entity(&rest[..semi + 1])
                {
                    out.push_str(&decoded);
                    i += semi + 1;
                    continue;
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

/// Decode a single HTML entity.
pub fn decode_entity(entity: &str) -> Option<String> {
    let decoded = match entity {
        "&amp;" => "&",
        "&lt;" => "<",
        "&gt;" => ">",
        "&quot;" => "\"",
        "&apos;" => "'",
        "&nbsp;" => " ",
        "&ndash;" => "\u{2013}",
        "&mdash;" => "\u{2014}",
        "&copy;" => "\u{00A9}",
        "&reg;" => "\u{00AE}",
        "&trade;" => "\u{2122}",
        "&hellip;" => "\u{2026}",
        "&bull;" => "\u{2022}",
        "&middot;" => "\u{00B7}",
        "&larr;" => "\u{2190}",
        "&rarr;" => "\u{2192}",
        _ => {
            if entity.starts_with("&#") && entity.ends_with(';') {
                let inner = &entity[2..entity.len() - 1];
                let code = if inner.starts_with('x') || inner.starts_with('X') {
                    u32::from_str_radix(&inner[1..], 16).ok()
                } else {
                    inner.parse::<u32>().ok()
                };
                return code.and_then(char::from_u32).map(|c| c.to_string());
            }
            return None;
        }
    };
    Some(decoded.to_string())
}

/// Final pass: resolve markers, dedup lines, collapse whitespace.
/// Handles code fences (```) by preserving whitespace inside them.
pub fn finalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut seen = HashSet::with_capacity(256);
    let mut blank_count = 0u32;
    let mut in_pre = false;

    let resolved = resolve_markers(text);

    for line in resolved.lines() {
        let trimmed = line.trim();
        if trimmed == "```" {
            in_pre = !in_pre;
            blank_count = 0;
            result.push_str("```\n");
            continue;
        }
        if in_pre {
            result.push_str(line);
            result.push('\n');
            continue;
        }
        let clean = collapse_whitespace(line);
        if clean.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
            continue;
        }
        blank_count = 0;
        if clean.len() > 10 && !seen.insert(hash_fast(&clean)) {
            continue;
        }
        result.push_str(&clean);
        result.push('\n');
    }
    result.trim().to_string()
}

fn hash_fast(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Common noise selectors for Reader mode.
pub fn reader_noise_selectors() -> &'static [&'static str] {
    &[
        // Core noise tags
        "script",
        "style",
        "nav",
        "footer",
        "header",
        "iframe",
        "noscript",
        "svg",
        "form",
        "button",
        "input",
        "select",
        "textarea",
        "head",
        "picture",
        "video",
        "audio",
        "canvas",
        // NOTE: "img" removed from noise — handled separately to extract alt text
        // Ad / tracking
        "[class*='ad-']",
        "[class*='ads-']",
        "[class*='cookie-']",
        "[class*='cookie_']",
        ".popup",
        ".modal",
        "[class*='-popup']",
        "[class*='-modal']",
        ".social-share",
        ".share-buttons",
        ".sharing",
        ".newsletter",
        ".subscribe",
        "[class*='-banner'][class*='ad']",
        "[role='navigation']",
        "[role='complementary']",
        "[role='search']",
        "[aria-hidden='true']",
        // Code block UI noise (line numbers, copy buttons)
        ".line-numbers",
        "[class*='numbering']",
        "[class*='line-number']",
        ".copy-button",
        "[class*='copy-btn']",
        "[class*='copy-code']",
        "[class*='toolbar']",
        // Chinese tech blog noise (腾讯云/CSDN/掘金/博客园)
        ".recommend-box",
        "[class*='recommend']",
        ".toc",
        "[class*='catalog']",
        "[class*='sidebar-toc']",
        ".author-info",
        ".profile-box",
        "[class*='author-card']",
        "[class*='article-tag']",
        ".tag-list",
        "[class*='related-article']",
        "[class*='hot-article']",
        "[class*='comment-box']",
        "[class*='comment-list']",
    ]
}

/// Minimal noise selectors for Operator mode (keep UI elements).
pub fn operator_noise_selectors() -> &'static [&'static str] {
    &[
        "script",
        "style",
        "head",
        "noscript",
        "svg",
        "picture",
        "video",
        "audio",
        "canvas",
        // NOTE: "img" handled separately (extract alt text)
        // Code block UI noise
        ".line-numbers",
        "[class*='numbering']",
        "[class*='line-number']",
        ".copy-button",
        "[class*='copy-btn']",
        "[class*='copy-code']",
    ]
}
