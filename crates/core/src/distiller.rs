//! AST-based distiller using scraper (Servo DOM parser).
//!
//! Single-pass Visitor + Context pattern:
//! - ParseContext tracks state (in_pre, in_table, list_depth)
//! - visit_node recursively walks DOM, pruning noise immediately
//! - URL resolution built into context
//!
//! Use for: complex tables, Developer mode, targeted extraction.
//! For bulk/fast processing, use distiller_fast.rs (lol_html stream).

use scraper::{ElementRef, Html, Node, Selector};
use std::collections::HashSet;

use crate::distiller_fast::DistillMode;
use crate::extract::{ExtractionResult, Extractor, Profile};

/// Parse context — carries state through recursive DOM walk.
struct ParseContext {
    base_origin: Option<String>,
    in_pre: bool,
    in_table: bool,
    list_depth: usize,
    list_ordered: bool,
    list_index: usize,
    output: String,
    links_count: usize,
    headings_count: usize,
}

impl ParseContext {
    fn new(base_url: Option<&str>, capacity: usize) -> Self {
        Self {
            base_origin: base_url.and_then(extract_origin),
            in_pre: false,
            in_table: false,
            list_depth: 0,
            list_ordered: false,
            list_index: 0,
            output: String::with_capacity(capacity),
            links_count: 0,
            headings_count: 0,
        }
    }

    /// Resolve URL — Reader mode is conservative (only http + root-relative).
    fn resolve_url(&self, href: &str) -> Option<String> {
        if href.starts_with("http") {
            Some(href.to_string())
        } else if href.starts_with("javascript:")
            || href.starts_with("mailto:")
            || href == "#"
            || href.is_empty()
        {
            None
        } else if href.starts_with('/') {
            self.base_origin.as_ref().map(|o| format!("{}{}", o, href))
        } else {
            // Bare relative (item?id=123): skip in Reader mode to keep output lean.
            // Operator/Spider modes should resolve these.
            None
        }
    }

    fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn push_char(&mut self, c: char) {
        self.output.push(c);
    }
}

/// AST Distiller — Visitor pattern over scraper DOM.
pub struct Distiller {
    noise_selectors: Vec<Selector>,
    content_selectors: Vec<Selector>,
}

impl Default for Distiller {
    fn default() -> Self {
        Self::new()
    }
}

impl Distiller {
    pub fn new() -> Self {
        let profile = Profile::reader(None);
        Self::from_profile(&profile)
    }

    pub fn from_profile(profile: &Profile) -> Self {
        Self {
            noise_selectors: profile
                .noise_selectors
                .iter()
                .filter_map(|s| Selector::parse(s).ok())
                .collect(),
            content_selectors: profile
                .content_selectors
                .iter()
                .filter_map(|s| Selector::parse(s).ok())
                .collect(),
        }
    }

    pub fn extract_title(&self, html: &str) -> Option<String> {
        let doc = Html::parse_document(html);
        let sel = Selector::parse("title").ok()?;
        doc.select(&sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty())
    }

    pub fn to_markdown(&self, html: &str) -> String {
        self.to_markdown_with_base(html, None)
    }

    pub fn to_markdown_with_base(&self, html: &str, base_url: Option<&str>) -> String {
        let doc = Html::parse_document(html);
        let root = self.find_content_root(&doc);
        let mut ctx = ParseContext::new(base_url, html.len() / 3);
        self.visit_node(root, &mut ctx);
        dedup_and_clean(&ctx.output)
    }

    pub fn to_text(&self, html: &str) -> String {
        let doc = Html::parse_document(html);
        let root = self.find_content_root(&doc);
        let mut ctx = ParseContext::new(None, html.len() / 3);
        ctx.in_pre = false; // text mode doesn't use pre markers
        self.visit_text_only(root, &mut ctx);
        dedup_and_clean(&ctx.output)
    }

    /// Find best content container.
    fn find_content_root<'a>(&self, doc: &'a Html) -> ElementRef<'a> {
        for sel in &self.content_selectors {
            if let Some(el) = doc.select(sel).next() {
                return el;
            }
        }
        let body_sel = Selector::parse("body").unwrap();
        doc.select(&body_sel).next().unwrap_or(doc.root_element())
    }

    /// Check if element matches any noise selector.
    fn is_noise(&self, el: &ElementRef<'_>) -> bool {
        self.noise_selectors.iter().any(|sel| sel.matches(el))
    }

    /// Single-pass recursive visitor — the core of the AST engine.
    fn visit_node(&self, el: ElementRef<'_>, ctx: &mut ParseContext) {
        // Immediate pruning — skip entire subtree
        if self.is_noise(&el) {
            return;
        }

        let tag = el.value().name();

        // --- Enter node: set context + emit prefix ---
        match tag {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag[1..].parse::<usize>().unwrap_or(1);
                ctx.push("\n\n");
                for _ in 0..level {
                    ctx.push_char('#');
                }
                ctx.push_char(' ');
                ctx.headings_count += 1;
                // Collect heading text directly (don't recurse into children for headings)
                let text: String = el.text().collect::<Vec<_>>().join(" ");
                ctx.push(text.trim());
                ctx.push("\n\n");
                return; // Don't recurse into heading children
            }
            "p" => ctx.push("\n"),
            "pre" => {
                ctx.in_pre = true;
                ctx.push("\n\n```\n");
            }
            "ul" => {
                ctx.list_depth += 1;
                ctx.list_ordered = false;
            }
            "ol" => {
                ctx.list_depth += 1;
                ctx.list_ordered = true;
                ctx.list_index = 0;
            }
            "li" => {
                if ctx.list_ordered {
                    ctx.list_index += 1;
                    ctx.push(&format!("\n{}. ", ctx.list_index));
                } else {
                    ctx.push("\n- ");
                }
            }
            "blockquote" => ctx.push("\n\n> "),
            "hr" => {
                ctx.push("\n\n---\n\n");
                return;
            }
            "br" => {
                ctx.push("\n");
                return;
            }
            "a" => {
                let href = el.value().attr("href").unwrap_or("");
                let text: String = el.text().collect::<Vec<_>>().join("");
                let text = text.trim();
                if text.is_empty() {
                    return;
                }

                if let Some(url) = ctx.resolve_url(href) {
                    ctx.push("[");
                    ctx.push(text);
                    ctx.push("](");
                    ctx.push(&url);
                    ctx.push(") ");
                    ctx.links_count += 1;
                } else {
                    ctx.push(text);
                    ctx.push(" ");
                }
                return; // Don't recurse into <a> children (already collected text)
            }
            "strong" | "b" => ctx.push("**"),
            "em" | "i" => ctx.push("_"),
            "code" if !ctx.in_pre => ctx.push("`"),
            "table" => {
                ctx.in_table = true;
                ctx.push("\n\n");
            }
            "tr" => ctx.push("\n"),
            "td" | "th" => ctx.push(" "),
            "div" | "section" | "article" => ctx.push("\n"),
            "img" => {
                if let Some(alt) = el.value().attr("alt")
                    && !alt.is_empty()
                {
                    ctx.push(&format!("[image: {}] ", alt));
                }
                return;
            }
            "script" | "style" | "svg" | "iframe" | "noscript" => return,
            _ => {}
        }

        // --- Recurse into children ---
        for child in el.children() {
            match child.value() {
                Node::Text(text) => {
                    if ctx.in_pre {
                        ctx.push(text); // Preserve whitespace in <pre>
                    } else {
                        let t = text.trim();
                        if !t.is_empty() {
                            ctx.push(t);
                            ctx.push_char(' ');
                        }
                    }
                }
                Node::Element(_) => {
                    if let Some(child_el) = ElementRef::wrap(child) {
                        self.visit_node(child_el, ctx);
                    }
                }
                _ => {}
            }
        }

        // --- Leave node: close markers + restore context ---
        match tag {
            "p" => ctx.push("\n"),
            "pre" => {
                ctx.in_pre = false;
                ctx.push("\n```\n\n");
            }
            "ul" | "ol" => {
                ctx.list_depth -= 1;
                ctx.push("\n");
            }
            "strong" | "b" => ctx.push("** "),
            "em" | "i" => ctx.push("_ "),
            "code" if !ctx.in_pre => ctx.push("`"),
            "table" => {
                ctx.in_table = false;
                ctx.push("\n");
            }
            "div" | "section" | "article" => ctx.push("\n"),
            "blockquote" => ctx.push("\n\n"),
            _ => {}
        }
    }

    /// Text-only visitor (no markdown formatting).
    fn visit_text_only(&self, el: ElementRef<'_>, ctx: &mut ParseContext) {
        if self.is_noise(&el) {
            return;
        }

        let tag = el.value().name();
        match tag {
            "script" | "style" | "svg" | "iframe" | "noscript" => return,
            "br" => {
                ctx.push("\n");
                return;
            }
            "p" | "div" | "tr" | "li" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => ctx.push("\n"),
            _ => {}
        }

        for child in el.children() {
            match child.value() {
                Node::Text(text) => {
                    let t = text.trim();
                    if !t.is_empty() {
                        ctx.push(t);
                        ctx.push_char(' ');
                    }
                }
                Node::Element(_) => {
                    if let Some(child_el) = ElementRef::wrap(child) {
                        self.visit_text_only(child_el, ctx);
                    }
                }
                _ => {}
            }
        }

        match tag {
            "p" | "div" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => ctx.push("\n"),
            _ => {}
        }
    }
}

/// Implement the Extractor trait for AST distiller.
impl Extractor for Distiller {
    fn extract(&self, html: &str, profile: &Profile) -> ExtractionResult {
        let distiller = Distiller::from_profile(profile);
        let doc = Html::parse_document(html);
        let root = distiller.find_content_root(&doc);
        let mut ctx = ParseContext::new(profile.base_url.as_deref(), html.len() / 3);

        match profile.mode {
            DistillMode::LlmFriendly | DistillMode::Reader | DistillMode::Operator => {
                distiller.visit_node(root, &mut ctx);
            }
            DistillMode::Data => {
                distiller.visit_node(root, &mut ctx);
                // TODO: structured JSON extraction
            }
            DistillMode::Developer => {
                distiller.visit_node(root, &mut ctx);
                // TODO: DOM skeleton output
            }
            DistillMode::Spider => {
                distiller.visit_node(root, &mut ctx);
                // TODO: link-only JSON output
            }
        }

        let content = dedup_and_clean(&ctx.output);
        let content_length = content.len();

        ExtractionResult {
            content,
            title: distiller.extract_title(html),
            content_length,
            links_count: ctx.links_count,
            headings_count: ctx.headings_count,
            engine: "ast-scraper".to_string(),
            mode: profile.mode,
        }
    }

    fn name(&self) -> &str {
        "ast-scraper"
    }
}

fn extract_origin(url: &str) -> Option<String> {
    let scheme_end = url.find("://")?;
    let after = &url[scheme_end + 3..];
    let host_end = after.find('/').unwrap_or(after.len());
    Some(url[..scheme_end + 3 + host_end].to_string())
}

fn hash_fast(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn dedup_and_clean(raw: &str) -> String {
    let mut seen = HashSet::with_capacity(256);
    let mut result = String::with_capacity(raw.len());
    let mut blank_count = 0u32;
    let mut in_pre = false;

    for line in raw.lines() {
        let trimmed = line.trim();

        // Track code fence state
        if trimmed == "```" {
            in_pre = !in_pre;
            blank_count = 0;
            result.push_str("```\n");
            continue;
        }

        if in_pre {
            // Preserve whitespace inside code blocks
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let clean: String = line.split_whitespace().collect::<Vec<_>>().join(" ");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title() {
        let d = Distiller::new();
        let html = "<html><head><title>Test Page</title></head><body></body></html>";
        assert_eq!(d.extract_title(html), Some("Test Page".to_string()));
    }

    #[test]
    fn test_noise_removal() {
        let d = Distiller::new();
        let html = r#"<html><body>
            <nav>Navigation</nav>
            <article><p>Main content here</p></article>
            <footer>Footer stuff</footer>
        </body></html>"#;
        let text = d.to_text(html);
        assert!(text.contains("Main content"), "got: {text}");
        assert!(!text.contains("Navigation"), "got: {text}");
        assert!(!text.contains("Footer"), "got: {text}");
    }

    #[test]
    fn test_markdown_headings() {
        let d = Distiller::new();
        let html = "<html><body><article><h1>Title</h1><p>Content</p></article></body></html>";
        let md = d.to_markdown(html);
        assert!(md.contains("# Title"), "got: {md}");
        assert!(md.contains("Content"), "got: {md}");
    }

    #[test]
    fn test_markdown_links() {
        let d = Distiller::new();
        let html = r#"<html><body><article><p><a href="https://example.com">Click here</a></p></article></body></html>"#;
        let md = d.to_markdown(html);
        assert!(
            md.contains("[Click here](https://example.com)"),
            "got: {md}"
        );
    }

    #[test]
    fn test_relative_links() {
        let d = Distiller::new();
        let html =
            r#"<html><body><article><p><a href="/foo/bar">Link</a></p></article></body></html>"#;
        let md = d.to_markdown_with_base(html, Some("https://example.com/page"));
        assert!(
            md.contains("[Link](https://example.com/foo/bar)"),
            "got: {md}"
        );
    }

    #[test]
    fn test_bare_relative_links_reader_skips() {
        // Reader mode: bare relative links (item?id=123) should be text only
        let d = Distiller::new();
        let html = r#"<html><body><article><p><a href="item?id=123">Comments</a></p></article></body></html>"#;
        let md = d.to_markdown_with_base(html, Some("https://news.ycombinator.com/"));
        assert!(
            md.contains("Comments"),
            "text should be preserved, got: {md}"
        );
        assert!(
            !md.contains("[Comments]("),
            "reader should NOT link bare relative, got: {md}"
        );
    }

    #[test]
    fn test_pre_preserves_whitespace() {
        let d = Distiller::new();
        let html = r#"<html><body><article><pre><code>fn main() {
    println!("hello");
}</code></pre></article></body></html>"#;
        let md = d.to_markdown(html);
        assert!(md.contains("```"), "got: {md}");
        assert!(
            md.contains("    println!"),
            "indentation preserved, got: {md}"
        );
    }

    #[test]
    fn test_dedup() {
        let d = Distiller::new();
        let html = "<html><body><article><p>Same line here</p><p>Same line here</p><p>Unique</p></article></body></html>";
        let md = d.to_markdown(html);
        assert_eq!(md.matches("Same line here").count(), 1, "got: {md}");
        assert!(md.contains("Unique"), "got: {md}");
    }

    #[test]
    fn test_profile_custom() {
        let profile = Profile::hacker_news();
        let d = Distiller::from_profile(&profile);
        assert!(d.noise_selectors.len() > 5);
    }

    #[test]
    fn test_extractor_trait() {
        let d = Distiller::new();
        let profile = Profile::reader(Some("https://example.com"));
        let html = "<html><body><article><h1>Test</h1><p>Content with <a href='/link'>link</a></p></article></body></html>";
        let result = d.extract(html, &profile);
        assert!(result.content.contains("# Test"), "got: {}", result.content);
        assert!(result.links_count >= 1, "links: {}", result.links_count);
        assert!(
            result.headings_count >= 1,
            "headings: {}",
            result.headings_count
        );
        assert_eq!(result.engine, "ast-scraper");
    }
}
