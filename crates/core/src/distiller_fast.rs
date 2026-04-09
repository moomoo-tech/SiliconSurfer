//! Fast streaming distiller — thin dispatcher over strategy modules.
//!
//! Each mode is implemented in its own strategy file under `strategy/`.
//! This file provides the public API and backward-compatible methods.

use serde::{Deserialize, Serialize};

use crate::strategy;

/// Distiller output mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DistillMode {
    /// Default for LLM: extreme noise removal + token compression + base_url links.
    #[default]
    LlmFriendly,
    /// Same as LlmFriendly (alias).
    Reader,
    /// Preserve UI elements, annotate buttons/forms/inputs/nav.
    Operator,
    /// Extract link topology only — JSON.
    Spider,
    /// DOM skeleton with id/class/role/data-* attributes.
    Developer,
    /// Structured table/list extraction as JSON.
    Data,
}

pub struct FastDistiller;

impl FastDistiller {
    pub fn extract_title(html: &str) -> Option<String> {
        let start = html.find("<title")?.checked_add(html[html.find("<title")?..].find('>')?)?;
        let start = start + 1;
        let end = html[start..].find("</title>").map(|i| i + start)?;
        let title = html[start..end].trim();
        if title.is_empty() { None } else { Some(decode_title(title)) }
    }

    /// Distill HTML with the specified mode.
    pub fn distill(html: &str, mode: DistillMode, base_url: Option<&str>) -> String {
        match mode {
            DistillMode::LlmFriendly | DistillMode::Reader => strategy::reader::distill(html, base_url),
            DistillMode::Operator => strategy::operator::distill(html, base_url),
            DistillMode::Spider => strategy::spider::distill(html, base_url),
            DistillMode::Developer => strategy::developer::distill(html, base_url),
            DistillMode::Data => strategy::data::distill(html, base_url),
        }
    }

    // Legacy API
    pub fn to_markdown(html: &str) -> String {
        strategy::reader::distill(html, None)
    }

    pub fn to_markdown_with_base(html: &str, base_url: Option<&str>) -> String {
        strategy::reader::distill(html, base_url)
    }

    pub fn to_text(html: &str) -> String {
        strategy::reader::to_text(html)
    }
}

fn decode_title(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '&' {
            let rest: String = chars[i..std::cmp::min(i + 12, len)].iter().collect();
            if let Some(semi) = rest.find(';') {
                if let Some(decoded) = strategy::decode_entity(&rest[..semi + 1]) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Title ----
    #[test]
    fn test_title() {
        let html = "<html><head><title>Hello World</title></head><body></body></html>";
        assert_eq!(FastDistiller::extract_title(html), Some("Hello World".to_string()));
    }

    // ---- Reader mode ----
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
    fn test_reader_skips_bare_relative() {
        let html = r#"<p><a href="item?id=123">Comments</a></p>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, Some("https://hn.com/"));
        assert!(md.contains("Comments"), "got: {md}");
        assert!(!md.contains("[Comments]("), "reader should not link bare relative, got: {md}");
    }

    #[test]
    fn test_reader_empty_links_skipped() {
        let html = r#"<a href="https://example.com"></a><a href="https://real.com">Real</a>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(!md.contains("[]("), "got: {md}");
        assert!(md.contains("[Real](https://real.com)"), "got: {md}");
    }

    #[test]
    fn test_reader_entities() {
        let html = "<p>Hello&#160;world &amp; foo&#91;bar&#93;</p>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("& foo[bar]"), "got: {md}");
    }

    #[test]
    fn test_reader_title_not_in_body() {
        let html = "<html><head><title>My Title</title></head><body><p>Content</p></body></html>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(!md.contains("My Title"), "got: {md}");
    }

    #[test]
    fn test_reader_pre_whitespace() {
        let html = "<pre><code>fn main() {\n    println!(\"hello\");\n}</code></pre>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("    println!"), "got: {md}");
    }

    #[test]
    fn test_reader_dedup() {
        let html = "<p>Same line here</p><p>Same line here</p><p>Unique</p>";
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert_eq!(md.matches("Same line here").count(), 1, "got: {md}");
    }

    #[test]
    fn test_reader_text_mode() {
        let html = "<html><body><h1>Title</h1><p>Hello <b>world</b></p></body></html>";
        let text = FastDistiller::to_text(html);
        assert!(text.contains("Title") && text.contains("Hello") && !text.contains("<"), "got: {text}");
    }

    // ---- Operator mode ----
    #[test]
    fn test_operator_preserves_nav() {
        let html = "<html><body><nav><a href='/login'>Login</a></nav><p>Content</p></body></html>";
        let md = FastDistiller::distill(html, DistillMode::Operator, Some("https://example.com"));
        assert!(md.contains("Login"), "got: {md}");
        assert!(md.contains("[Nav]"), "got: {md}");
    }

    #[test]
    fn test_operator_annotates_buttons() {
        let html = "<button>Submit</button>";
        let md = FastDistiller::distill(html, DistillMode::Operator, None);
        assert!(md.contains("[Button:") && md.contains("Submit"), "got: {md}");
    }

    #[test]
    fn test_operator_annotates_forms() {
        let html = r#"<form action="/search" method="GET"><input type="text" name="q" placeholder="Search"></form>"#;
        let md = FastDistiller::distill(html, DistillMode::Operator, None);
        assert!(md.contains("[Form:") && md.contains("[Input:") && md.contains("name=q"), "got: {md}");
    }

    #[test]
    fn test_operator_resolves_bare_relative() {
        let html = r#"<p><a href="item?id=123">Comments</a></p>"#;
        let md = FastDistiller::distill(html, DistillMode::Operator, Some("https://hn.com/"));
        assert!(md.contains("item?id=123"), "got: {md}");
        assert!(md.contains("[Comments]"), "got: {md}");
    }

    // ---- Spider mode ----
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
    }

    #[test]
    fn test_spider_bare_relative() {
        let html = r#"<a href="item?id=123">Comments</a>"#;
        let json = FastDistiller::distill(html, DistillMode::Spider, Some("https://hn.com/"));
        assert!(json.contains("item?id=123"), "got: {json}");
    }

    // ---- Developer mode ----
    #[test]
    fn test_developer_skeleton() {
        let html = r#"<html><body><div id="app" class="container"><h1>Title</h1></div></body></html>"#;
        let skeleton = FastDistiller::distill(html, DistillMode::Developer, None);
        assert!(skeleton.contains("id=\"app\"") && skeleton.contains("class=\"container\""), "got: {skeleton}");
    }

    // ---- Data mode ----
    #[test]
    fn test_data_tables() {
        let html = r#"<table><tr><th>Name</th><th>Price</th></tr><tr><td>SSD</td><td>$99</td></tr></table>"#;
        let json = FastDistiller::distill(html, DistillMode::Data, None);
        let data: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(!data["tables"].as_array().unwrap().is_empty(), "got: {json}");
    }

    #[test]
    fn test_data_lists() {
        let html = "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>";
        let json = FastDistiller::distill(html, DistillMode::Data, None);
        let data: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(!data["lists"].as_array().unwrap().is_empty(), "got: {json}");
    }

    // ---- Image handling ----
    #[test]
    fn test_reader_img_alt_text() {
        let html = r#"<p>Check this:</p><img alt="Architecture diagram" src="https://example.com/arch.png"><p>More text</p>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("[image: Architecture diagram]"), "should preserve alt text, got: {md}");
        assert!(md.contains("arch.png"), "should preserve src, got: {md}");
    }

    #[test]
    fn test_reader_img_lazy_load() {
        let html = r#"<img data-src="https://cdn.example.com/real.png" src="placeholder.gif" alt="Flow chart">"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("[image: Flow chart]"), "got: {md}");
        assert!(md.contains("cdn.example.com/real.png"), "should use data-src over src, got: {md}");
        assert!(!md.contains("placeholder.gif"), "should prefer data-src, got: {md}");
    }

    #[test]
    fn test_reader_img_no_alt_no_src() {
        let html = r#"<img src="data:image/gif;base64,R0lGOD"><p>Text</p>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(!md.contains("data:image"), "should skip base64 images, got: {md}");
        assert!(md.contains("Text"), "got: {md}");
    }

    #[test]
    fn test_reader_code_block_noise() {
        let html = r#"<pre><code>fn main() {}</code></pre><div class="copy-btn">Copy</div><ul class="line-numbers"><li>1</li></ul>"#;
        let md = FastDistiller::distill(html, DistillMode::Reader, None);
        assert!(md.contains("fn main()"), "code should be preserved, got: {md}");
        assert!(!md.contains("Copy"), "copy button should be removed, got: {md}");
    }

    // ---- LlmFriendly = Reader ----
    #[test]
    fn test_llm_friendly_same_as_reader() {
        let html = "<h1>Title</h1><p>Content</p>";
        let friendly = FastDistiller::distill(html, DistillMode::LlmFriendly, None);
        let reader = FastDistiller::distill(html, DistillMode::Reader, None);
        assert_eq!(friendly, reader);
    }
}
