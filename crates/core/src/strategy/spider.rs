//! Spider strategy — extract link topology as JSON.
//!
//! Categorizes links into nav_links, content_links, footer_links.
//! Resolves all relative URLs.

use super::{extract_origin, resolve_href};

pub fn distill(html: &str, base_url: Option<&str>) -> String {
    let base_origin = base_url.and_then(extract_origin);
    let doc = scraper::Html::parse_document(html);
    let a_sel = scraper::Selector::parse("a[href]").unwrap();
    let nav_sel =
        scraper::Selector::parse("nav a[href], header a[href], [role='navigation'] a[href]")
            .unwrap();
    let footer_sel = scraper::Selector::parse("footer a[href]").unwrap();

    let nav_ids: std::collections::HashSet<_> = doc.select(&nav_sel).map(|e| e.id()).collect();
    let footer_ids: std::collections::HashSet<_> =
        doc.select(&footer_sel).map(|e| e.id()).collect();

    let mut nav_links = Vec::new();
    let mut content_links = Vec::new();
    let mut footer_links = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for el in doc.select(&a_sel) {
        let href = el.value().attr("href").unwrap_or("");
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
    })
    .to_string()
}
