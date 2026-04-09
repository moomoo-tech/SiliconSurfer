//! Fast streaming distiller with multi-mode vision.
//!
//! 5 modes for different Agent tasks:
//! - Reader: extreme noise removal, LLM-friendly markdown
//! - Operator: preserve UI elements, annotate actionable nodes
//! - Spider: extract link topology only
//! - Developer: DOM skeleton with id/class/role attributes
//! - Data: structured table/list extraction as JSON

use lol_html::{element, rewrite_str, RewriteStrSettings};
use serde::{Deserialize, Serialize};

/// Distiller output mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DistillMode {
    /// Default for LLM: extreme noise removal + token compression + base_url links
    /// Uses lol_html stream engine. Conservative link policy (http + /path only).
    #[default]
    LlmFriendly,
    /// Same as LlmFriendly but with less noise removal (keeps more structure)
    Reader,
    /// Preserve UI elements, annotate actionable nodes for interaction
    Operator,
    /// Extract link topology only — JSON array of links
    Spider,
    /// DOM skeleton with attributes — for writing scripts/selectors
    Developer,
    /// Structured table/list extraction as JSON
    Data,
}

pub struct FastDistiller;

impl FastDistiller {
    pub fn extract_title(html: &str) -> Option<String> {
        let start = html.find("<title")?.checked_add(html[html.find("<title")?..].find('>')?)?;
        let start = start + 1;
        let end = html[start..].find("</title>").map(|i| i + start)?;
        let title = html[start..end].trim();
        if title.is_empty() { None } else { Some(decode_entities_fast(title)) }
    }

    /// Distill HTML with the specified mode.
    pub fn distill(html: &str, mode: DistillMode, base_url: Option<&str>) -> String {
        match mode {
            DistillMode::LlmFriendly | DistillMode::Reader => Self::mode_reader(html, base_url),
            DistillMode::Operator => Self::mode_operator(html, base_url),
            DistillMode::Spider => Self::mode_spider(html, base_url),
            DistillMode::Developer => Self::mode_developer(html),
            DistillMode::Data => Self::mode_data(html, base_url),
        }
    }

    // Legacy API
    pub fn to_markdown(html: &str) -> String {
        Self::distill(html, DistillMode::Reader, None)
    }

    pub fn to_markdown_with_base(html: &str, base_url: Option<&str>) -> String {
        Self::distill(html, DistillMode::Reader, base_url)
    }

    pub fn to_text(html: &str) -> String {
        let rewritten = rewrite_reader(html, None, false);
        let text = strip_tags(&rewritten);
        finalize(&text)
    }

    // ---- Mode implementations ----

    /// 📖 Reader: extreme noise removal, clean markdown
    fn mode_reader(html: &str, base_url: Option<&str>) -> String {
        let rewritten = rewrite_reader(html, base_url, true);
        let text = strip_tags(&rewritten);
        finalize(&text)
    }

    /// 🕹️ Operator: preserve UI, annotate actionable elements
    fn mode_operator(html: &str, base_url: Option<&str>) -> String {
        let rewritten = rewrite_operator(html, base_url);
        let text = strip_tags(&rewritten);
        finalize(&text)
    }

    /// 🕸️ Spider: extract links only as JSON
    fn mode_spider(html: &str, base_url: Option<&str>) -> String {
        extract_links(html, base_url)
    }

    /// 🛠️ Developer: DOM skeleton with attributes
    fn mode_developer(html: &str) -> String {
        extract_dom_skeleton(html)
    }

    /// 📊 Data: structured table/list extraction
    fn mode_data(html: &str, base_url: Option<&str>) -> String {
        extract_structured_data(html, base_url)
    }
}

// ==== Reader mode (existing logic) ====

fn reader_noise_selectors() -> Vec<&'static str> {
    vec![
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
    ]
}

fn rewrite_reader(html: &str, base_url: Option<&str>, markdown: bool) -> String {
    use lol_html::html_content::ContentType::Text;
    let base_origin = base_url.and_then(extract_origin);

    let mut handlers = Vec::new();
    for sel in reader_noise_selectors() {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }

    if markdown {
        handlers.push(element!("h1", |el| { el.before("\n\n# ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("h2", |el| { el.before("\n\n## ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("h3", |el| { el.before("\n\n### ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("h4, h5, h6", |el| { el.before("\n\n#### ", Text); el.after("\n\n", Text); Ok(()) }));
        handlers.push(element!("p", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));
        // Reader mode: only absolute + root-relative links (conservative)
        let bo = base_origin.clone();
        handlers.push(element!("a[href]", move |el| {
            let href = el.get_attribute("href").unwrap_or_default();
            // Reader: only http:// and /path links, skip bare relative (item?id=123)
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

// ==== Operator mode ====

fn rewrite_operator(html: &str, base_url: Option<&str>) -> String {
    use lol_html::html_content::ContentType::Text;
    let base_origin = base_url.and_then(extract_origin);

    // Minimal noise removal — only scripts/styles, keep UI elements
    let minimal_noise = ["script", "style", "head", "noscript", "svg",
                         "img", "picture", "video", "audio", "canvas"];

    let mut handlers = Vec::new();
    for sel in minimal_noise {
        handlers.push(element!(sel, |el| { el.remove(); Ok(()) }));
    }

    // Headings
    handlers.push(element!("h1", |el| { el.before("\n\n# ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h2", |el| { el.before("\n\n## ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("h3", |el| { el.before("\n\n### ", Text); el.after("\n\n", Text); Ok(()) }));
    handlers.push(element!("p", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));

    // Links — annotate with action type, resolve all relative URLs
    let bo = base_origin.clone();
    let bu = base_url.map(|s| s.to_string());
    handlers.push(element!("a[href]", move |el| {
        let href = el.get_attribute("href").unwrap_or_default();
        let full_url = resolve_href(&href, bo.as_deref(), bu.as_deref())
            .unwrap_or(href);
        el.before("\x01L", Text);
        el.after(&format!("\x02{}\x03", full_url), Text);
        Ok(())
    }));

    // Buttons — annotate as actions
    handlers.push(element!("button", |el| {
        el.before("[Button: ", Text);
        el.after("]", Text);
        Ok(())
    }));

    // Input fields — show type and name
    handlers.push(element!("input", |el| {
        let input_type = el.get_attribute("type").unwrap_or_else(|| "text".to_string());
        let name = el.get_attribute("name").unwrap_or_else(|| "?".to_string());
        let placeholder = el.get_attribute("placeholder").unwrap_or_default();
        el.before(&format!("[Input: type={} name={}", input_type, name), Text);
        if !placeholder.is_empty() {
            el.after(&format!(" placeholder=\"{}\"]", placeholder), Text);
        } else {
            el.after("]", Text);
        }
        Ok(())
    }));

    // Forms
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

    handlers.push(element!("strong, b", |el| { el.before("**", Text); el.after("**", Text); Ok(()) }));
    handlers.push(element!("li", |el| { el.before("\n- ", Text); Ok(()) }));
    handlers.push(element!("tr", |el| { el.after("\n", Text); Ok(()) }));
    handlers.push(element!("div, section, article", |el| { el.before("\n", Text); el.after("\n", Text); Ok(()) }));
    handlers.push(element!("hr", |el| { el.before("\n---\n", Text); Ok(()) }));
    handlers.push(element!("br", |el| { el.before("\n", Text); Ok(()) }));
    handlers.push(element!("pre", |el| { el.before("\n\x04", Text); el.after("\x05\n", Text); Ok(()) }));

    rewrite_str(html, RewriteStrSettings {
        element_content_handlers: handlers,
        ..RewriteStrSettings::new()
    }).unwrap_or_else(|_| html.to_string())
}

// ==== Spider mode ====

fn extract_links(html: &str, base_url: Option<&str>) -> String {
    let base_origin = base_url.and_then(extract_origin);

    // Parse with scraper for reliable CSS selection
    let doc = scraper::Html::parse_document(html);
    let a_sel = scraper::Selector::parse("a[href]").unwrap();
    let nav_sel = scraper::Selector::parse("nav a[href], header a[href], [role='navigation'] a[href]").unwrap();
    let footer_sel = scraper::Selector::parse("footer a[href]").unwrap();

    let nav_ids: std::collections::HashSet<_> = doc.select(&nav_sel).map(|e| e.id()).collect();
    let footer_ids: std::collections::HashSet<_> = doc.select(&footer_sel).map(|e| e.id()).collect();

    let mut nav_links = Vec::new();
    let mut content_links = Vec::new();
    let mut footer_links = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for el in doc.select(&a_sel) {
        let href = el.value().attr("href").unwrap_or("");
        if href.is_empty() || href == "#" || href.starts_with("javascript:") {
            continue;
        }

        let full_url = match resolve_href(href, base_origin.as_deref(), base_url) {
            Some(url) => url,
            None => continue,
        };

        if !seen.insert(full_url.clone()) {
            continue;
        }

        let text: String = el.text().collect::<Vec<_>>().join(" ");
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }

        let link = serde_json::json!({"text": text, "url": full_url});

        if nav_ids.contains(&el.id()) {
            nav_links.push(link);
        } else if footer_ids.contains(&el.id()) {
            footer_links.push(link);
        } else {
            content_links.push(link);
        }
    }

    serde_json::json!({
        "nav_links": nav_links,
        "content_links": content_links,
        "footer_links": footer_links,
        "total": nav_links.len() + content_links.len() + footer_links.len(),
    }).to_string()
}

// ==== Developer mode ====

fn extract_dom_skeleton(html: &str) -> String {
    // Strip script/style content but keep structure
    let cleaned = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![
                element!("script, style", |el| { el.remove(); Ok(()) }),
                element!("head", |el| { el.remove(); Ok(()) }),
            ],
            ..RewriteStrSettings::new()
        },
    ).unwrap_or_else(|_| html.to_string());

    let doc = scraper::Html::parse_document(&cleaned);
    let body_sel = scraper::Selector::parse("body").unwrap();
    let body = match doc.select(&body_sel).next() {
        Some(b) => b,
        None => return String::new(),
    };

    let mut out = String::with_capacity(html.len() / 4);
    write_skeleton(&body, &mut out, 0);
    out
}

fn write_skeleton(el: &scraper::ElementRef<'_>, out: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    let tag = el.value().name();

    // Skip text-only wrappers at deep levels
    if depth > 8 {
        return;
    }

    // Build attribute string (only useful ones)
    let mut attrs = Vec::new();
    if let Some(id) = el.value().attr("id") {
        attrs.push(format!("id=\"{}\"", id));
    }
    if let Some(class) = el.value().attr("class") {
        // Truncate long class lists
        let short = if class.len() > 60 { &class[..60] } else { class };
        attrs.push(format!("class=\"{}\"", short));
    }
    if let Some(role) = el.value().attr("role") {
        attrs.push(format!("role=\"{}\"", role));
    }
    if let Some(href) = el.value().attr("href") {
        let short = if href.len() > 80 { &href[..80] } else { href };
        attrs.push(format!("href=\"{}\"", short));
    }
    if let Some(src) = el.value().attr("src") {
        let short = if src.len() > 80 { &src[..80] } else { src };
        attrs.push(format!("src=\"{}\"", short));
    }
    // data-* attributes
    for attr in el.value().attrs() {
        if attr.0.starts_with("data-") && attrs.len() < 8 {
            let val = if attr.1.len() > 40 { &attr.1[..40] } else { attr.1 };
            attrs.push(format!("{}=\"{}\"", attr.0, val));
        }
    }

    let attr_str = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };

    // Get direct text content (not from children)
    let text: String = el.children()
        .filter_map(|c| {
            if let scraper::Node::Text(t) = c.value() {
                let trimmed = t.trim();
                if !trimmed.is_empty() && trimmed.len() < 80 {
                    Some(trimmed.to_string())
                } else if !trimmed.is_empty() {
                    Some(format!("{}...", &trimmed[..60]))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let children: Vec<scraper::ElementRef> = el.children()
        .filter_map(scraper::ElementRef::wrap)
        .collect();

    if children.is_empty() && !text.is_empty() {
        out.push_str(&format!("{}<{}{}>{}</{}>\n", indent, tag, attr_str, text, tag));
    } else if children.is_empty() {
        out.push_str(&format!("{}<{}{} />\n", indent, tag, attr_str));
    } else {
        out.push_str(&format!("{}<{}{}>\n", indent, tag, attr_str));
        if !text.is_empty() {
            out.push_str(&format!("{}  {}\n", indent, text));
        }
        for child in children {
            write_skeleton(&child, out, depth + 1);
        }
        out.push_str(&format!("{}</{}>\n", indent, tag));
    }
}

// ==== Data mode ====

fn extract_structured_data(html: &str, base_url: Option<&str>) -> String {
    let doc = scraper::Html::parse_document(html);

    let mut result = serde_json::json!({});

    // Extract tables
    let table_sel = scraper::Selector::parse("table").unwrap();
    let tr_sel = scraper::Selector::parse("tr").unwrap();
    let th_sel = scraper::Selector::parse("th").unwrap();
    let td_sel = scraper::Selector::parse("td").unwrap();

    let mut tables = Vec::new();
    for table in doc.select(&table_sel) {
        let mut rows = Vec::new();
        let mut headers: Vec<String> = Vec::new();

        for tr in table.select(&tr_sel) {
            let ths: Vec<String> = tr.select(&th_sel)
                .map(|c| c.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();
            let tds: Vec<String> = tr.select(&td_sel)
                .map(|c| c.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if !ths.is_empty() && headers.is_empty() {
                headers = ths;
            } else if !tds.is_empty() {
                if !headers.is_empty() {
                    let mut row = serde_json::Map::new();
                    for (i, val) in tds.iter().enumerate() {
                        let key = headers.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
                        row.insert(key, serde_json::Value::String(val.clone()));
                    }
                    rows.push(serde_json::Value::Object(row));
                } else {
                    rows.push(serde_json::Value::Array(
                        tds.into_iter().map(serde_json::Value::String).collect()
                    ));
                }
            }
        }

        if !rows.is_empty() {
            tables.push(serde_json::json!({
                "headers": headers,
                "rows": rows,
            }));
        }
    }

    // Extract lists
    let ul_sel = scraper::Selector::parse("ul, ol").unwrap();
    let li_sel = scraper::Selector::parse("li").unwrap();

    let mut lists = Vec::new();
    for list in doc.select(&ul_sel) {
        let items: Vec<String> = list.select(&li_sel)
            .map(|li| li.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .filter(|s| !s.is_empty() && s.len() < 500)
            .collect();
        if items.len() >= 2 {
            lists.push(serde_json::Value::Array(
                items.into_iter().map(serde_json::Value::String).collect()
            ));
        }
    }

    // Extract definition lists
    let dl_sel = scraper::Selector::parse("dl").unwrap();
    let dt_sel = scraper::Selector::parse("dt").unwrap();
    let dd_sel = scraper::Selector::parse("dd").unwrap();

    let mut definitions = Vec::new();
    for dl in doc.select(&dl_sel) {
        let dts: Vec<String> = dl.select(&dt_sel)
            .map(|dt| dt.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect();
        let dds: Vec<String> = dl.select(&dd_sel)
            .map(|dd| dd.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .collect();
        let pairs: Vec<_> = dts.into_iter().zip(dds).map(|(k, v)| serde_json::json!({"term": k, "definition": v})).collect();
        if !pairs.is_empty() {
            definitions.push(serde_json::Value::Array(pairs));
        }
    }

    result["tables"] = serde_json::json!(tables);
    result["lists"] = serde_json::json!(lists);
    result["definitions"] = serde_json::json!(definitions);

    serde_json::to_string_pretty(&result).unwrap_or_default()
}

// ==== Shared utilities ====

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

fn collapse_whitespace(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last_was_space = true;
    for c in line.chars() {
        if c.is_whitespace() {
            if !last_was_space { out.push(' '); }
            last_was_space = true;
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    if out.ends_with(' ') { out.pop(); }
    out
}

fn finalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut seen = std::collections::HashSet::with_capacity(256);
    let mut blank_count = 0u32;
    let mut in_pre = false;

    let resolved = resolve_markers(text);

    for line in resolved.lines() {
        let trimmed = line.trim();

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
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let clean = collapse_whitespace(line);
        if clean.is_empty() {
            blank_count += 1;
            if blank_count <= 1 { result.push('\n'); }
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

fn resolve_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        match chars[i] {
            '\x01' if i + 1 < len && chars[i + 1] == 'L' => {
                i += 2;
                let text_start = i;
                while i < len && chars[i] != '\x02' { i += 1; }
                let link_text: String = chars[text_start..i].iter().collect();
                let link_text = link_text.trim();
                i += 1;
                let url_start = i;
                while i < len && chars[i] != '\x03' { i += 1; }
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
            '\x04' => { out.push_str("```\n"); i += 1; }
            '\x05' => { out.push_str("\n```"); i += 1; }
            '&' => {
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
            ch => { out.push(ch); i += 1; }
        }
    }
    out
}

fn decode_entity(entity: &str) -> Option<String> {
    let decoded = match entity {
        "&amp;" => "&", "&lt;" => "<", "&gt;" => ">", "&quot;" => "\"",
        "&apos;" => "'", "&nbsp;" => " ", "&ndash;" => "\u{2013}",
        "&mdash;" => "\u{2014}", "&copy;" => "\u{00A9}", "&reg;" => "\u{00AE}",
        "&trade;" => "\u{2122}", "&hellip;" => "\u{2026}", "&bull;" => "\u{2022}",
        "&middot;" => "\u{00B7}", "&larr;" => "\u{2190}", "&rarr;" => "\u{2192}",
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

fn decode_entities_fast(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '&' {
            let rest: String = chars[i..std::cmp::min(i + 12, len)].iter().collect();
            if let Some(semi) = rest.find(';') {
                if let Some(decoded) = decode_entity(&rest[..semi + 1]) {
                    out.push_str(&decoded);
                    i += semi + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn hash_fast(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Resolve an href to a full URL. Handles absolute, root-relative, and bare relative paths.
fn resolve_href(href: &str, base_origin: Option<&str>, base_url: Option<&str>) -> Option<String> {
    if href.starts_with("http") {
        Some(href.to_string())
    } else if href.starts_with("javascript:") || href.starts_with("mailto:") || href == "#" || href.is_empty() {
        None
    } else if href.starts_with('/') {
        base_origin.map(|o| format!("{}{}", o, href))
    } else {
        // Bare relative path like "item?id=123" or "foo/bar"
        base_url.map(|base| {
            let base_dir = if let Some(last_slash) = base.rfind('/') {
                &base[..last_slash + 1]
            } else {
                base
            };
            format!("{}{}", base_dir, href)
        })
    }
}

fn extract_origin(url: &str) -> Option<String> {
    let scheme_end = url.find("://")?;
    let after = &url[scheme_end + 3..];
    let host_end = after.find('/').unwrap_or(after.len());
    Some(url[..scheme_end + 3 + host_end].to_string())
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
    fn test_reader_noise() {
        let html = "<html><body><nav>Menu</nav><p>Content here</p><footer>Foot</footer></body></html>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("Content here"), "got: {md}");
        assert!(!md.contains("Menu"), "got: {md}");
    }

    #[test]
    fn test_reader_headings() {
        let html = "<h1>Title</h1><p>Text</p>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("# Title"), "got: {md}");
    }

    #[test]
    fn test_reader_links() {
        let html = r#"<p><a href="https://example.com">Click</a></p>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("[Click](https://example.com)"), "got: {md}");
    }

    #[test]
    fn test_reader_relative_links() {
        let html = r#"<p><a href="/foo">Link</a></p>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, Some("https://example.com/page"));
        assert!(md.contains("[Link](https://example.com/foo)"), "got: {md}");
    }

    #[test]
    fn test_operator_preserves_nav() {
        let html = "<html><body><nav><a href='/login'>Login</a></nav><p>Content</p></body></html>";
        let md = FastDistiller::distill(html, DistillMode::Operator, Some("https://example.com"));
        assert!(md.contains("Login"), "operator should preserve nav, got: {md}");
        assert!(md.contains("Content"), "got: {md}");
    }

    #[test]
    fn test_operator_annotates_buttons() {
        let html = "<button>Submit</button><button>Cancel</button>";
        let md = FastDistiller::distill(html, DistillMode::Operator, None);
        assert!(md.contains("[Button:"), "got: {md}");
        assert!(md.contains("Submit"), "got: {md}");
    }

    #[test]
    fn test_operator_annotates_forms() {
        let html = r#"<form action="/search" method="GET"><input type="text" name="q" placeholder="Search"></form>"#;
        let md = FastDistiller::distill(html, DistillMode::Operator, None);
        assert!(md.contains("[Form:"), "got: {md}");
        assert!(md.contains("[Input:"), "got: {md}");
        assert!(md.contains("name=q"), "got: {md}");
    }

    #[test]
    fn test_spider_extracts_links() {
        let html = r#"<html><body>
            <nav><a href="/about">About</a></nav>
            <main><a href="https://example.com">Example</a></main>
            <footer><a href="/privacy">Privacy</a></footer>
        </body></html>"#;
        let json = FastDistiller::distill(html, DistillMode::Spider, Some("https://test.com"));
        let data: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(data["total"].as_u64().unwrap() >= 2, "got: {json}");
        assert!(data["nav_links"].as_array().unwrap().len() >= 1, "got: {json}");
    }

    #[test]
    fn test_developer_skeleton() {
        let html = r#"<html><body><div id="app" class="container"><h1>Title</h1><p>Text</p></div></body></html>"#;
        let skeleton = FastDistiller::distill(html, DistillMode::Developer, None);
        assert!(skeleton.contains("id=\"app\""), "got: {skeleton}");
        assert!(skeleton.contains("class=\"container\""), "got: {skeleton}");
        assert!(skeleton.contains("<h1>"), "got: {skeleton}");
    }

    #[test]
    fn test_data_tables() {
        let html = r#"<table><tr><th>Name</th><th>Price</th></tr><tr><td>SSD</td><td>$99</td></tr></table>"#;
        let json = FastDistiller::distill(html, DistillMode::Data, None);
        let data: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let tables = data["tables"].as_array().unwrap();
        assert!(!tables.is_empty(), "got: {json}");
        assert!(json.contains("SSD"), "got: {json}");
        assert!(json.contains("$99"), "got: {json}");
    }

    #[test]
    fn test_data_lists() {
        let html = "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>";
        let json = FastDistiller::distill(html, DistillMode::Data, None);
        let data: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let lists = data["lists"].as_array().unwrap();
        assert!(!lists.is_empty(), "got: {json}");
    }

    #[test]
    fn test_reader_skips_bare_relative_links() {
        // Reader mode: bare relative (item?id=123) should be text only, not a link
        // This keeps Reader output lean. Spider/Operator modes resolve these.
        let html = r#"<p><a href="item?id=123">283 comments</a></p>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, Some("https://news.ycombinator.com/"));
        assert!(md.contains("283 comments"), "text should be preserved, got: {md}");
        assert!(!md.contains("[283 comments]("), "reader should NOT make bare relative a link, got: {md}");
    }

    #[test]
    fn test_operator_resolves_bare_relative_links() {
        // Operator mode: bare relative links ARE resolved (for interaction)
        let html = r#"<p><a href="item?id=123">283 comments</a></p>"#;
        let md = FastDistiller::distill(html, DistillMode::Operator, Some("https://news.ycombinator.com/"));
        assert!(md.contains("item?id=123"), "got: {md}");
        assert!(md.contains("[283 comments]"), "got: {md}");
    }

    #[test]
    fn test_spider_bare_relative_links() {
        let html = r#"<a href="item?id=123">Comments</a><a href="/about">About</a>"#;
        let json = FastDistiller::distill(html, DistillMode::Spider, Some("https://news.ycombinator.com/"));
        assert!(json.contains("item?id=123"), "got: {json}");
        assert!(json.contains("/about"), "got: {json}");
    }

    #[test]
    fn test_pre_preserves_whitespace() {
        let html = "<pre><code>fn main() {\n    println!(\"hello\");\n}</code></pre>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("    println!"), "got: {md}");
    }

    #[test]
    fn test_entities_decoded() {
        let html = "<p>Hello&#160;world &amp; foo</p>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("& foo"), "got: {md}");
        assert!(!md.contains("&#"), "got: {md}");
    }

    #[test]
    fn test_dedup() {
        let html = "<p>Same line here</p><p>Same line here</p><p>Unique</p>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert_eq!(md.matches("Same line here").count(), 1, "got: {md}");
    }
}
