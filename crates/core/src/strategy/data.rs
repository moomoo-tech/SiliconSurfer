//! Data strategy — structured table/list extraction as JSON.
//!
//! For extracting tabular data, price lists, comparison tables.

pub fn distill(html: &str, _base_url: Option<&str>) -> String {
    let doc = scraper::Html::parse_document(html);

    let table_sel = scraper::Selector::parse("table").unwrap();
    let tr_sel = scraper::Selector::parse("tr").unwrap();
    let th_sel = scraper::Selector::parse("th").unwrap();
    let td_sel = scraper::Selector::parse("td").unwrap();

    let mut tables = Vec::new();
    for table in doc.select(&table_sel) {
        let mut rows = Vec::new();
        let mut headers: Vec<String> = Vec::new();

        for tr in table.select(&tr_sel) {
            let ths: Vec<String> = tr
                .select(&th_sel)
                .map(|c| c.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();
            let tds: Vec<String> = tr
                .select(&td_sel)
                .map(|c| c.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();

            if !ths.is_empty() && headers.is_empty() {
                headers = ths;
            } else if !tds.is_empty() {
                if !headers.is_empty() {
                    let mut row = serde_json::Map::new();
                    for (i, val) in tds.iter().enumerate() {
                        let key = headers
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| format!("col_{}", i));
                        row.insert(key, serde_json::Value::String(val.clone()));
                    }
                    rows.push(serde_json::Value::Object(row));
                } else {
                    rows.push(serde_json::Value::Array(
                        tds.into_iter().map(serde_json::Value::String).collect(),
                    ));
                }
            }
        }
        if !rows.is_empty() {
            tables.push(serde_json::json!({"headers": headers, "rows": rows}));
        }
    }

    // Lists
    let ul_sel = scraper::Selector::parse("ul, ol").unwrap();
    let li_sel = scraper::Selector::parse("li").unwrap();
    let mut lists = Vec::new();
    for list in doc.select(&ul_sel) {
        let items: Vec<String> = list
            .select(&li_sel)
            .map(|li| li.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .filter(|s| !s.is_empty() && s.len() < 500)
            .collect();
        if items.len() >= 2 {
            lists.push(serde_json::Value::Array(
                items.into_iter().map(serde_json::Value::String).collect(),
            ));
        }
    }

    serde_json::json!({"tables": tables, "lists": lists}).to_string()
}
