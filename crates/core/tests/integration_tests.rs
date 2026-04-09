//! Integration tests for SiliconSurfer core.
//!
//! Tests cover all 5 distill modes, edge cases, and bug regression.

use agent_browser_core::distiller_fast::{DistillMode, FastDistiller};

// ============================================================
// Reader Mode — comprehensive edge cases
// ============================================================

#[test]
fn reader_basic_article() {
    let html = r#"<html><body>
        <article>
            <h1>Breaking News</h1>
            <p>First paragraph with <strong>bold</strong> and <em>italic</em>.</p>
            <p>Second paragraph with <a href="https://example.com">a link</a>.</p>
        </article>
    </body></html>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("# Breaking News"), "heading missing: {md}");
    assert!(md.contains("**bold**"), "bold missing: {md}");
    assert!(md.contains("_italic_"), "italic missing: {md}");
    assert!(md.contains("[a link](https://example.com)"), "link missing: {md}");
}

#[test]
fn reader_strips_all_noise() {
    let html = r#"<html><body>
        <nav>Navigation Bar</nav>
        <header>Site Header</header>
        <main><p>Main content</p></main>
        <footer>Copyright 2026</footer>
        <script>alert('xss')</script>
        <style>.foo{color:red}</style>
        <iframe src="ad.html"></iframe>
        <noscript>Enable JS</noscript>
    </body></html>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("Main content"), "content lost: {md}");
    assert!(!md.contains("Navigation"), "nav not stripped: {md}");
    assert!(!md.contains("Site Header"), "header not stripped: {md}");
    assert!(!md.contains("Copyright"), "footer not stripped: {md}");
    assert!(!md.contains("alert"), "script not stripped: {md}");
    assert!(!md.contains("color:red"), "style not stripped: {md}");
    assert!(!md.contains("Enable JS"), "noscript not stripped: {md}");
}

#[test]
fn reader_code_block_preserves_indentation() {
    let html = r#"<pre><code>
def hello():
    print("world")
    for i in range(10):
        print(i)
</code></pre>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("```"), "no code fence: {md}");
    assert!(md.contains("    print(\"world\")"), "indentation lost: {md}");
    assert!(md.contains("        print(i)"), "nested indent lost: {md}");
}

#[test]
fn reader_multiple_headings() {
    let html = "<h1>H1</h1><h2>H2</h2><h3>H3</h3><h4>H4</h4><p>text</p>";
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("# H1"), "h1: {md}");
    assert!(md.contains("## H2"), "h2: {md}");
    assert!(md.contains("### H3"), "h3: {md}");
    assert!(md.contains("#### H4"), "h4: {md}");
}

#[test]
fn reader_ordered_and_unordered_lists() {
    let html = "<ul><li>Apple</li><li>Banana</li></ul><ol><li>First</li><li>Second</li></ol>";
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("- Apple"), "unordered list: {md}");
    assert!(md.contains("- Banana"), "unordered list: {md}");
    // Ordered lists render as unordered in lol_html (known limitation)
    assert!(md.contains("First"), "ordered list: {md}");
}

#[test]
fn reader_entities_comprehensive() {
    let html = "<p>&amp; &lt; &gt; &quot; &apos; &nbsp; &copy; &reg; &#8212; &#x2014;</p>";
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("&"), "amp: {md}");
    assert!(md.contains("<"), "lt: {md}");
    assert!(md.contains(">"), "gt: {md}");
    assert!(md.contains("\""), "quot: {md}");
    assert!(md.contains("\u{00A9}"), "copy: {md}");
    assert!(!md.contains("&#"), "unresolved entities: {md}");
}

#[test]
fn reader_img_with_alt_and_lazy_load() {
    let html = r#"
        <img alt="Photo 1" src="https://img.com/1.jpg">
        <img data-src="https://cdn.com/real.png" src="placeholder.gif" alt="Lazy">
        <img src="data:image/gif;base64,R0lGOD" alt="">
    "#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("[image: Photo 1]"), "regular img: {md}");
    assert!(md.contains("cdn.com/real.png"), "lazy load src: {md}");
    assert!(!md.contains("data:image"), "base64 should be skipped: {md}");
}

#[test]
fn reader_javascript_links_skipped() {
    let html = "<a href=\"javascript:void(0)\">Click</a><a href=\"mailto:x@y.com\">Email</a><a href=\"#\">Top</a>";
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(!md.contains("javascript:"), "js link: {md}");
    assert!(!md.contains("mailto:"), "mailto link: {md}");
    assert!(md.contains("Click"), "text preserved: {md}");
}

#[test]
fn reader_dedup_removes_repeated_lines() {
    let html = r#"
        <p>This is a repeated navigation item here</p>
        <p>This is a repeated navigation item here</p>
        <p>This is a repeated navigation item here</p>
        <p>Unique content</p>
    "#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert_eq!(md.matches("repeated navigation").count(), 1, "dedup failed: {md}");
    assert!(md.contains("Unique content"), "lost unique: {md}");
}

#[test]
fn reader_relative_link_with_base() {
    let html = r#"<a href="/docs/api">API Docs</a>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, Some("https://example.com/page"));
    assert!(md.contains("[API Docs](https://example.com/docs/api)"), "got: {md}");
}

#[test]
fn reader_bare_relative_not_linked() {
    let html = r#"<a href="item?id=42">Details</a>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, Some("https://example.com/"));
    assert!(md.contains("Details"), "text: {md}");
    assert!(!md.contains("[Details]("), "bare relative should not be linked: {md}");
}

// ============================================================
// Operator Mode — @e references, forms, buttons
// ============================================================

#[test]
fn operator_full_login_form() {
    let html = r#"
        <form action="/login" method="POST">
            <input type="hidden" name="csrf" value="abc123">
            <label>Username</label>
            <input type="text" name="username" placeholder="Enter username">
            <label>Password</label>
            <input type="password" name="password">
            <button type="submit">Sign In</button>
        </form>
    "#;
    let md = FastDistiller::distill(html, DistillMode::Operator, None);
    assert!(md.contains("[Form: POST /login]"), "form: {md}");
    assert!(md.contains("@e1"), "csrf input: {md}");
    assert!(md.contains("@e2"), "username input: {md}");
    assert!(md.contains("@e3"), "password input: {md}");
    assert!(md.contains("@e4"), "submit button: {md}");
    assert!(md.contains("name=username"), "field name: {md}");
    assert!(md.contains("name=password"), "field name: {md}");
    assert!(md.contains("[Button:"), "button: {md}");
    assert!(md.contains("Sign In"), "button text: {md}");
}

#[test]
fn operator_select_and_textarea() {
    let html = r#"
        <select name="country"><option>US</option><option>UK</option></select>
        <textarea name="comments">Hello</textarea>
    "#;
    let md = FastDistiller::distill(html, DistillMode::Operator, None);
    assert!(md.contains("[Select: country]"), "select: {md}");
    assert!(md.contains("[Textarea: comments]"), "textarea: {md}");
    assert!(md.contains("@e1"), "select ref: {md}");
    assert!(md.contains("@e2"), "textarea ref: {md}");
}

#[test]
fn operator_preserves_nav_and_footer() {
    let html = r#"<nav><a href="/home">Home</a></nav><p>Content</p><footer><a href="/about">About</a></footer>"#;
    let md = FastDistiller::distill(html, DistillMode::Operator, Some("https://test.com"));
    assert!(md.contains("Home"), "nav preserved: {md}");
    assert!(md.contains("About"), "footer preserved: {md}");
    assert!(md.contains("[Nav]"), "nav tag: {md}");
}

#[test]
fn operator_radio_and_checkbox() {
    let html = r#"
        <input type="radio" name="size" value="S"> Small
        <input type="radio" name="size" value="M"> Medium
        <input type="checkbox" name="extra" value="cheese"> Cheese
    "#;
    let md = FastDistiller::distill(html, DistillMode::Operator, None);
    assert!(md.contains("type=radio"), "radio: {md}");
    assert!(md.contains("type=checkbox"), "checkbox: {md}");
    assert!(md.contains("@e1") && md.contains("@e2") && md.contains("@e3"), "refs: {md}");
}

#[test]
fn operator_bare_relative_resolved() {
    let html = r#"<a href="item?id=42">View</a>"#;
    let md = FastDistiller::distill(html, DistillMode::Operator, Some("https://example.com/list"));
    assert!(md.contains("item?id=42"), "bare relative resolved: {md}");
    assert!(md.contains("example.com"), "base applied: {md}");
}

#[test]
fn operator_placeholder_shown() {
    let html = r#"<input type="email" name="email" placeholder="you@example.com">"#;
    let md = FastDistiller::distill(html, DistillMode::Operator, None);
    assert!(md.contains(r#"placeholder="you@example.com""#), "placeholder: {md}");
}

// ============================================================
// Spider Mode — link extraction
// ============================================================

#[test]
fn spider_categorizes_links() {
    let html = r#"<html><body>
        <nav><a href="/home">Home</a><a href="/about">About</a></nav>
        <main>
            <a href="https://external.com">External</a>
            <a href="/products">Products</a>
        </main>
        <footer><a href="/privacy">Privacy</a></footer>
    </body></html>"#;
    let json = FastDistiller::distill(html, DistillMode::Spider, Some("https://test.com"));
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(data["nav_links"].as_array().unwrap().len() >= 2, "nav: {json}");
    assert!(data["content_links"].as_array().unwrap().len() >= 1, "content: {json}");
    assert!(data["footer_links"].as_array().unwrap().len() >= 1, "footer: {json}");
}

#[test]
fn spider_skips_javascript_and_empty() {
    let html = "<a href=\"javascript:void(0)\">JS Link</a><a href=\"#\">Anchor</a><a href=\"\">Empty</a><a href=\"https://real.com\">Real</a>";
    let json = FastDistiller::distill(html, DistillMode::Spider, None);
    assert!(!json.contains("javascript:"), "js link: {json}");
    assert!(json.contains("real.com"), "real link: {json}");
}

#[test]
fn spider_deduplicates() {
    let html = r#"
        <a href="https://example.com/page">Link 1</a>
        <a href="https://example.com/page">Link 2</a>
    "#;
    let json = FastDistiller::distill(html, DistillMode::Spider, None);
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(data["total"].as_u64().unwrap(), 1, "should dedup: {json}");
}

// ============================================================
// Developer Mode — DOM skeleton
// ============================================================

#[test]
fn developer_preserves_attributes() {
    let html = r#"<html><body>
        <div id="app" class="container" role="main" data-page="home">
            <h1>Title</h1>
            <a href="/link" class="btn">Click</a>
        </div>
    </body></html>"#;
    let skeleton = FastDistiller::distill(html, DistillMode::Developer, None);
    assert!(skeleton.contains(r#"id="app""#), "id: {skeleton}");
    assert!(skeleton.contains(r#"class="container""#), "class: {skeleton}");
    assert!(skeleton.contains(r#"role="main""#), "role: {skeleton}");
    assert!(skeleton.contains(r#"data-page="home""#), "data attr: {skeleton}");
    assert!(skeleton.contains(r#"href="/link""#), "href: {skeleton}");
}

#[test]
fn developer_strips_script_style() {
    let html = "<html><head><style>.x{}</style></head><body><script>alert(1)</script><p>Text</p></body></html>";
    let skeleton = FastDistiller::distill(html, DistillMode::Developer, None);
    assert!(!skeleton.contains("alert"), "script: {skeleton}");
    assert!(!skeleton.contains(".x{}"), "style: {skeleton}");
    assert!(skeleton.contains("<p>"), "p tag: {skeleton}");
}

// ============================================================
// Data Mode — tables and lists
// ============================================================

#[test]
fn data_extracts_table_with_headers() {
    let html = r#"<table>
        <tr><th>Product</th><th>Price</th><th>Stock</th></tr>
        <tr><td>SSD 1TB</td><td>$89</td><td>In stock</td></tr>
        <tr><td>HDD 2TB</td><td>$49</td><td>Out of stock</td></tr>
    </table>"#;
    let json = FastDistiller::distill(html, DistillMode::Data, None);
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let tables = data["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 1, "tables: {json}");
    let table = &tables[0];
    assert_eq!(table["headers"].as_array().unwrap().len(), 3, "headers: {json}");
    assert_eq!(table["rows"].as_array().unwrap().len(), 2, "rows: {json}");
    assert!(json.contains("SSD 1TB"), "content: {json}");
    assert!(json.contains("$89"), "price: {json}");
}

#[test]
fn data_extracts_lists() {
    let html = r#"
        <ul><li>Red</li><li>Green</li><li>Blue</li></ul>
        <ol><li>First</li><li>Second</li></ol>
    "#;
    let json = FastDistiller::distill(html, DistillMode::Data, None);
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let lists = data["lists"].as_array().unwrap();
    assert!(lists.len() >= 2, "lists: {json}");
}

#[test]
fn data_table_without_headers() {
    let html = r#"<table>
        <tr><td>A</td><td>1</td></tr>
        <tr><td>B</td><td>2</td></tr>
    </table>"#;
    let json = FastDistiller::distill(html, DistillMode::Data, None);
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let tables = data["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 1, "no-header table: {json}");
}

// ============================================================
// Title extraction
// ============================================================

#[test]
fn title_basic() {
    assert_eq!(
        FastDistiller::extract_title("<html><head><title>My Page</title></head></html>"),
        Some("My Page".to_string())
    );
}

#[test]
fn title_with_entities() {
    assert_eq!(
        FastDistiller::extract_title("<html><head><title>A &amp; B</title></head></html>"),
        Some("A & B".to_string())
    );
}

#[test]
fn title_empty() {
    assert_eq!(
        FastDistiller::extract_title("<html><head><title></title></head></html>"),
        None
    );
}

#[test]
fn title_missing() {
    assert_eq!(FastDistiller::extract_title("<html><body>No title</body></html>"), None);
}

// ============================================================
// Cross-mode consistency
// ============================================================

#[test]
fn llm_friendly_equals_reader() {
    let html = "<h1>Title</h1><p>Content with <a href=\"https://x.com\">link</a></p>";
    let lf = FastDistiller::distill(html, DistillMode::LlmFriendly, None);
    let rd = FastDistiller::distill(html, DistillMode::Reader, None);
    assert_eq!(lf, rd, "LlmFriendly should equal Reader");
}

#[test]
fn operator_has_more_content_than_reader() {
    let html = r#"<html><body><nav><a href="/x">Nav</a></nav><form><input name="q"><button>Go</button></form><p>Text</p></body></html>"#;
    let reader = FastDistiller::distill(html, DistillMode::Reader, None);
    let operator = FastDistiller::distill(html, DistillMode::Operator, None);
    assert!(operator.len() > reader.len(), "operator should have more: reader={} operator={}", reader.len(), operator.len());
    assert!(operator.contains("[Form:"), "operator has form: {operator}");
    assert!(operator.contains("[Input:"), "operator has input: {operator}");
}

#[test]
fn spider_returns_valid_json() {
    let html = r#"<a href="https://a.com">A</a><a href="https://b.com">B</a>"#;
    let json = FastDistiller::distill(html, DistillMode::Spider, None);
    let data: serde_json::Value = serde_json::from_str(&json).expect("spider must return valid JSON");
    assert!(data["total"].is_number(), "total field: {json}");
}

#[test]
fn data_returns_valid_json() {
    let html = "<p>No tables here</p>";
    let json = FastDistiller::distill(html, DistillMode::Data, None);
    let data: serde_json::Value = serde_json::from_str(&json).expect("data must return valid JSON");
    assert!(data["tables"].is_array(), "tables field: {json}");
    assert!(data["lists"].is_array(), "lists field: {json}");
}

// ============================================================
// Bug regression tests
// ============================================================

#[test]
fn regression_code_block_noise_stripped() {
    // Bug: line numbers and copy buttons leaked into code blocks
    let html = r#"
        <pre><code>console.log("hello");</code></pre>
        <div class="copy-btn">Copy</div>
        <ul class="line-numbers"><li>1</li></ul>
    "#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("console.log"), "code: {md}");
    assert!(!md.contains("Copy"), "copy button: {md}");
}

#[test]
fn regression_title_not_in_body() {
    // Bug: <title> text leaked into body output
    let html = "<html><head><title>Secret Title</title></head><body><p>Body</p></body></html>";
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(md.contains("Body"), "body: {md}");
    assert!(!md.contains("Secret Title"), "title leaked: {md}");
}

#[test]
fn regression_empty_links_not_rendered() {
    // Bug: empty <a> tags rendered as []()
    let html = r#"<a href="https://x.com"></a><a href="https://y.com">Y</a>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(!md.contains("[]("), "empty link: {md}");
    assert!(md.contains("[Y](https://y.com)"), "real link: {md}");
}

#[test]
fn regression_base64_images_skipped() {
    let html = r#"<img src="data:image/png;base64,iVBOR"><p>After</p>"#;
    let md = FastDistiller::distill(html, DistillMode::Reader, None);
    assert!(!md.contains("data:image"), "base64: {md}");
    assert!(md.contains("After"), "content: {md}");
}

#[test]
fn regression_large_page_no_panic() {
    // Ensure large pages don't cause O(n²) or panic
    let mut html = String::from("<html><body>");
    for i in 0..1000 {
        html.push_str(&format!("<p>Paragraph {} with <a href='https://x.com/{}'>link</a></p>", i, i));
    }
    html.push_str("</body></html>");

    let md = FastDistiller::distill(&html, DistillMode::Reader, None);
    assert!(md.len() > 1000, "should have content: {}", md.len());
    assert!(md.contains("Paragraph 999"), "last paragraph: {}...", &md[md.len().saturating_sub(200)..]);
}
