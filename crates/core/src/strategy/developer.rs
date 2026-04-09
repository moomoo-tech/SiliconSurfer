//! Developer strategy — DOM skeleton with id/class/role/data-* attributes.
//!
//! For writing Playwright scripts, CSS selectors, or verifying DOM structure.

use lol_html::{RewriteStrSettings, element, rewrite_str};

pub fn distill(html: &str, _base_url: Option<&str>) -> String {
    // Strip script/style/head content
    let cleaned = rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("script, style, head", |el| {
                el.remove();
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap_or_else(|_| html.to_string());

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
    if depth > 8 {
        return;
    }
    let indent = "  ".repeat(depth);
    let tag = el.value().name();

    // Collect useful attributes
    let mut attrs = Vec::new();
    for (key, val) in [
        ("id", el.value().attr("id")),
        ("class", el.value().attr("class")),
        ("role", el.value().attr("role")),
        ("href", el.value().attr("href")),
        ("src", el.value().attr("src")),
        ("action", el.value().attr("action")),
        ("name", el.value().attr("name")),
        ("type", el.value().attr("type")),
    ] {
        if let Some(v) = val {
            let short = if v.len() > 60 { &v[..60] } else { v };
            attrs.push(format!("{}=\"{}\"", key, short));
        }
    }
    // data-* attributes
    for attr in el.value().attrs() {
        if attr.0.starts_with("data-") && attrs.len() < 10 {
            let v = if attr.1.len() > 40 {
                &attr.1[..40]
            } else {
                attr.1
            };
            attrs.push(format!("{}=\"{}\"", attr.0, v));
        }
    }

    let attr_str = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };

    // Direct text content
    let text: String = el
        .children()
        .filter_map(|c| {
            if let scraper::Node::Text(t) = c.value() {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    Some(if trimmed.len() > 60 {
                        format!("{}...", &trimmed[..60])
                    } else {
                        trimmed.to_string()
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let children: Vec<scraper::ElementRef> = el
        .children()
        .filter_map(scraper::ElementRef::wrap)
        .collect();

    if children.is_empty() && !text.is_empty() {
        out.push_str(&format!(
            "{}<{}{}>{}</{}>\n",
            indent, tag, attr_str, text, tag
        ));
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
