use scraper::{Html, Selector, ElementRef};
use std::collections::HashSet;

/// DOM noise reduction + content extraction pipeline.
///
/// Takes raw HTML → strips noise → extracts text → deduplicates → outputs clean Markdown or text.
pub struct Distiller {
    noise_selectors: Vec<Selector>,
    content_selectors: Vec<Selector>,
}

impl Distiller {
    pub fn new() -> Self {
        let noise_tags = [
            "script", "style", "nav", "footer", "header", "iframe",
            "noscript", "svg", "form", "button", "input", "select", "textarea",
            // Ad / tracking / UI noise
            "[class*='ad-']", "[class*='ads-']", "[class*='advert']",
            "[id*='google_ads']",
            "[class*='cookie-']", "[class*='cookie_']",
            ".popup", ".modal", "[class*='-popup']", "[class*='-modal']",
            "[class*='-banner'][class*='ad']",
            "[class*='social']", "[class*='share']", "[class*='newsletter']",
            "[class*='related']", "[class*='recommended']",
            "[role='navigation']", "[role='complementary']",
            "[role='search']", "[aria-hidden='true']",
        ];

        let content_tags = [
            "article", "main", "[role='main']",
            ".post-content", ".article-content", ".entry-content",
            ".post-body", ".article-body",
            "#content", ".content", "#main-content",
        ];

        Self {
            noise_selectors: noise_tags
                .iter()
                .filter_map(|s| Selector::parse(s).ok())
                .collect(),
            content_selectors: content_tags
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

    /// Convert HTML to clean Markdown
    pub fn to_markdown(&self, html: &str) -> String {
        let doc = Html::parse_document(html);
        let content_root = self.find_content_root(&doc);
        let noise_ids = self.collect_noise_ids(&doc);

        let mut md = String::with_capacity(html.len() / 4);
        self.walk_to_markdown(content_root, &doc, &noise_ids, &mut md);

        dedup_and_clean(&md)
    }

    /// Convert HTML to plain text
    pub fn to_text(&self, html: &str) -> String {
        let doc = Html::parse_document(html);
        let content_root = self.find_content_root(&doc);
        let noise_ids = self.collect_noise_ids(&doc);

        let mut text = String::with_capacity(html.len() / 4);
        self.walk_to_text(content_root, &doc, &noise_ids, &mut text);

        dedup_and_clean(&text)
    }

    /// Find the best content root element
    fn find_content_root<'a>(&self, doc: &'a Html) -> ElementRef<'a> {
        for sel in &self.content_selectors {
            if let Some(el) = doc.select(sel).next() {
                return el;
            }
        }
        // Fallback: body or root
        let body_sel = Selector::parse("body").unwrap();
        doc.select(&body_sel).next().unwrap_or(doc.root_element())
    }

    /// Collect node IDs of all noise elements (and their descendants)
    fn collect_noise_ids(&self, doc: &Html) -> HashSet<ego_tree::NodeId> {
        let mut ids = HashSet::new();
        for sel in &self.noise_selectors {
            for el in doc.select(sel) {
                // Mark this element and all descendants as noise
                ids.insert(el.id());
                for desc in el.descendants() {
                    ids.insert(desc.id());
                }
            }
        }
        ids
    }

    /// Check if a node is noise
    fn is_noise(&self, node_id: ego_tree::NodeId, noise_ids: &HashSet<ego_tree::NodeId>) -> bool {
        noise_ids.contains(&node_id)
    }

    /// Walk DOM tree, output Markdown, skip noise nodes
    fn walk_to_markdown(
        &self,
        el: ElementRef<'_>,
        doc: &Html,
        noise_ids: &HashSet<ego_tree::NodeId>,
        md: &mut String,
    ) {
        use scraper::Node;

        for child in el.children() {
            if self.is_noise(child.id(), noise_ids) {
                continue;
            }

            match child.value() {
                Node::Text(text) => {
                    let t = text.trim();
                    if !t.is_empty() {
                        md.push_str(t);
                        md.push(' ');
                    }
                }
                Node::Element(elem) => {
                    let tag = elem.name();
                    let child_ref = ElementRef::wrap(child);

                    match tag {
                        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            let level = tag[1..].parse::<usize>().unwrap_or(1);
                            let prefix = "#".repeat(level);
                            md.push_str("\n\n");
                            md.push_str(&prefix);
                            md.push(' ');
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, md);
                            }
                            md.push_str("\n\n");
                        }
                        "p" | "div" => {
                            md.push('\n');
                            if let Some(r) = child_ref {
                                self.walk_to_markdown(r, doc, noise_ids, md);
                            }
                            md.push('\n');
                        }
                        "br" => md.push('\n'),
                        "a" => {
                            let href = elem.attr("href").unwrap_or("");
                            // Collect link text first
                            let mut link_text = String::new();
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, &mut link_text);
                            }
                            let link_text = link_text.trim().to_string();

                            if link_text.is_empty() {
                                // Empty link (vote buttons etc) — skip entirely
                            } else if href.is_empty() || href == "#"
                                || href.starts_with("vote?")
                                || href.starts_with("hide?")
                                || href.starts_with("login")
                                || href.starts_with("javascript:")
                            {
                                // Non-useful link, output text only
                                md.push_str(&link_text);
                                md.push(' ');
                            } else if href.starts_with("http") {
                                // Full URL — render as markdown link
                                md.push('[');
                                md.push_str(&link_text);
                                md.push_str("](");
                                md.push_str(href);
                                md.push(')');
                                md.push(' ');
                            } else {
                                // Relative URL — just output text
                                md.push_str(&link_text);
                                md.push(' ');
                            }
                        }
                        "strong" | "b" => {
                            md.push_str("**");
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, md);
                            }
                            md.push_str("** ");
                        }
                        "em" | "i" => {
                            md.push('_');
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, md);
                            }
                            md.push_str("_ ");
                        }
                        "code" => {
                            md.push('`');
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, md);
                            }
                            md.push('`');
                        }
                        "pre" => {
                            md.push_str("\n\n```\n");
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, md);
                            }
                            md.push_str("\n```\n\n");
                        }
                        "ul" | "ol" => {
                            md.push('\n');
                            if let Some(r) = child_ref {
                                let li_sel = Selector::parse("li").unwrap();
                                for (i, li) in r.select(&li_sel).enumerate() {
                                    if self.is_noise(li.id(), noise_ids) {
                                        continue;
                                    }
                                    if tag == "ol" {
                                        md.push_str(&format!("\n{}. ", i + 1));
                                    } else {
                                        md.push_str("\n- ");
                                    }
                                    self.collect_clean_text(li, noise_ids, md);
                                }
                            }
                            md.push('\n');
                        }
                        "blockquote" => {
                            md.push_str("\n\n> ");
                            if let Some(r) = child_ref {
                                self.collect_clean_text(r, noise_ids, md);
                            }
                            md.push_str("\n\n");
                        }
                        "hr" => md.push_str("\n\n---\n\n"),
                        "img" => {
                            if let Some(alt) = elem.attr("alt") {
                                if !alt.is_empty() {
                                    md.push_str(&format!("[image: {}] ", alt));
                                }
                            }
                        }
                        // Layout tables → flatten to text (not markdown tables)
                        // Only render as markdown table if it looks like a data table
                        "table" => {
                            if let Some(r) = child_ref {
                                if is_data_table(r) {
                                    self.render_data_table(r, noise_ids, md);
                                } else {
                                    // Layout table: just extract text with line breaks
                                    md.push('\n');
                                    self.walk_to_markdown(r, doc, noise_ids, md);
                                    md.push('\n');
                                }
                            }
                        }
                        "tr" => {
                            if let Some(r) = child_ref {
                                self.walk_to_markdown(r, doc, noise_ids, md);
                            }
                            md.push('\n');
                        }
                        "td" | "th" => {
                            if let Some(r) = child_ref {
                                self.walk_to_markdown(r, doc, noise_ids, md);
                            }
                            md.push(' ');
                        }
                        // Skip remaining noise
                        "script" | "style" | "svg" | "iframe" | "noscript" => {}
                        // Recurse into everything else
                        _ => {
                            if let Some(r) = child_ref {
                                self.walk_to_markdown(r, doc, noise_ids, md);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Walk DOM tree, output plain text only
    fn walk_to_text(
        &self,
        el: ElementRef<'_>,
        doc: &Html,
        noise_ids: &HashSet<ego_tree::NodeId>,
        text: &mut String,
    ) {
        use scraper::Node;

        for child in el.children() {
            if self.is_noise(child.id(), noise_ids) {
                continue;
            }
            match child.value() {
                Node::Text(t) => {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        text.push_str(trimmed);
                        text.push(' ');
                    }
                }
                Node::Element(elem) => {
                    match elem.name() {
                        "script" | "style" | "svg" | "iframe" | "noscript" => {}
                        "br" => text.push('\n'),
                        "p" | "div" | "tr" | "li" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                            text.push('\n');
                            if let Some(r) = ElementRef::wrap(child) {
                                self.walk_to_text(r, doc, noise_ids, text);
                            }
                            text.push('\n');
                        }
                        _ => {
                            if let Some(r) = ElementRef::wrap(child) {
                                self.walk_to_text(r, doc, noise_ids, text);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Collect clean text from an element, skipping noise descendants
    fn collect_clean_text(
        &self,
        el: ElementRef<'_>,
        noise_ids: &HashSet<ego_tree::NodeId>,
        out: &mut String,
    ) {
        use scraper::Node;

        for desc in el.children() {
            if self.is_noise(desc.id(), noise_ids) {
                continue;
            }
            match desc.value() {
                Node::Text(text) => {
                    let t = text.trim();
                    if !t.is_empty() {
                        out.push_str(t);
                        out.push(' ');
                    }
                }
                Node::Element(_) => {
                    if let Some(r) = ElementRef::wrap(desc) {
                        self.collect_clean_text(r, noise_ids, out);
                    }
                }
                _ => {}
            }
        }
    }

    /// Render a data table as markdown table
    fn render_data_table(
        &self,
        table: ElementRef<'_>,
        noise_ids: &HashSet<ego_tree::NodeId>,
        md: &mut String,
    ) {
        let tr_sel = Selector::parse("tr").unwrap();
        let cell_sel = Selector::parse("th, td").unwrap();

        md.push_str("\n\n");
        let mut first_row = true;
        for tr in table.select(&tr_sel) {
            if self.is_noise(tr.id(), noise_ids) {
                continue;
            }
            let cells: Vec<String> = tr
                .select(&cell_sel)
                .filter(|c| !self.is_noise(c.id(), noise_ids))
                .map(|c| {
                    let mut text = String::new();
                    self.collect_clean_text(c, noise_ids, &mut text);
                    text.trim().to_string()
                })
                .collect();

            if cells.is_empty() {
                continue;
            }

            md.push('|');
            for cell in &cells {
                md.push(' ');
                md.push_str(cell);
                md.push_str(" |");
            }
            md.push('\n');

            if first_row {
                md.push('|');
                for _ in &cells {
                    md.push_str(" --- |");
                }
                md.push('\n');
                first_row = false;
            }
        }
        md.push('\n');
    }
}

/// Heuristic: is this a data table (should render as markdown table)
/// or a layout table (should flatten to text)?
fn is_data_table(table: ElementRef<'_>) -> bool {
    let th_sel = Selector::parse("th").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    // Has <th> headers → likely data table
    let has_headers = table.select(&th_sel).next().is_some();

    // Count rows and cells to check regularity
    let rows: Vec<usize> = table
        .select(&tr_sel)
        .map(|tr| tr.select(&td_sel).count() + tr.select(&th_sel).count())
        .filter(|&c| c > 0)
        .collect();

    if rows.is_empty() {
        return false;
    }

    // Data tables usually have consistent column counts
    let first_cols = rows[0];
    let is_regular = rows.iter().all(|&c| c == first_cols);

    // Data table: has headers OR (regular shape AND multiple columns AND multiple rows)
    has_headers || (is_regular && first_cols >= 2 && rows.len() >= 2)
}

/// Deduplicate lines and clean up whitespace
fn dedup_and_clean(raw: &str) -> String {
    let mut seen = HashSet::new();
    let mut result = String::with_capacity(raw.len());
    let mut blank_count = 0;

    for line in raw.lines() {
        // Collapse internal whitespace
        let clean: String = line.split_whitespace().collect::<Vec<_>>().join(" ");

        if clean.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
            continue;
        }

        blank_count = 0;

        // Skip duplicate lines (exact match after whitespace normalization)
        if clean.len() > 10 && !seen.insert(clean.clone()) {
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
    fn test_to_text_strips_tags() {
        let d = Distiller::new();
        let html = "<html><body><article><p>Hello <b>world</b></p></article></body></html>";
        let text = d.to_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
        assert!(!text.contains("<"));
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
        assert!(text.contains("Main content"));
        assert!(!text.contains("Navigation"));
        assert!(!text.contains("Footer"));
    }

    #[test]
    fn test_markdown_headings() {
        let d = Distiller::new();
        let html = "<html><body><article><h1>Title</h1><p>Content</p></article></body></html>";
        let md = d.to_markdown(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("Content"));
    }

    #[test]
    fn test_markdown_links() {
        let d = Distiller::new();
        let html = r#"<html><body><article><p><a href="https://example.com">Click here</a></p></article></body></html>"#;
        let md = d.to_markdown(html);
        // Link text may have trailing space before ']'
        assert!(md.contains("[Click here](https://example.com)") ||
                md.contains("[Click here ](https://example.com)"));
    }

    #[test]
    fn test_layout_table_flattened() {
        let d = Distiller::new();
        // HN-style layout table (single wide cell) should NOT become markdown table
        let html = r#"<html><body><table><tr><td>
            <a href="/item">Some title</a>
            <span>100 points</span>
        </td></tr></table></body></html>"#;
        let md = d.to_markdown(html);
        // Should NOT have markdown table pipes as structure
        assert!(!md.starts_with("|"));
        assert!(md.contains("Some title"));
    }

    #[test]
    fn test_data_table_rendered() {
        let d = Distiller::new();
        let html = r#"<html><body><table>
            <tr><th>Name</th><th>Price</th></tr>
            <tr><td>SSD</td><td>$99</td></tr>
            <tr><td>HDD</td><td>$49</td></tr>
        </table></body></html>"#;
        let md = d.to_markdown(html);
        assert!(md.contains("| Name | Price |"));
        assert!(md.contains("| SSD | $99 |"));
    }

    #[test]
    fn test_dedup_lines() {
        let d = Distiller::new();
        // article wraps content so distiller finds it; repeated divs are outside
        let html = r#"<html><body>
            <article>
                <p>Repeated navigation text here</p>
                <p>Repeated navigation text here</p>
                <p>Unique content</p>
                <p>Repeated navigation text here</p>
            </article>
        </body></html>"#;
        let md = d.to_markdown(html);
        let count = md.matches("Repeated navigation text here").count();
        assert_eq!(count, 1, "Duplicate lines should be removed");
        assert!(md.contains("Unique content"));
    }
}
