# Multi-mode Vision Architecture

## Core Insight

Different Agent tasks need different "views":
- Summarize news → maximum noise removal (Reader)
- Like/order → preserve UI buttons (Operator)
- Explore site → just the link graph (Spider)
- Write scraper script → need DOM skeleton (Developer)
- Extract tabular data → structured output (Data)

## 5 Modes (All Implemented)

### 1. Reader 📖
- Maximum signal-to-noise ratio, saves tokens
- Strips nav/footer/ads/UI buttons
- Keeps only headings, body text, links, tables
- Use case: "summarize article", "extract product specs"

### 2. Operator 🕹️
- Preserves all actionable anchors
- Annotates UI elements with @eN references
- Output: `@e3 [Button: Add to Cart]`, `@e4 [Input: name=email]`
- Use case: "register account", "add to cart", "login"

### 3. Spider 🕸️
- Extracts link topology only
- Categorized by region: nav_links / content_links / footer_links
- JSON array output
- Use case: "find About Us page", "all article links"
- Tokens: very low

### 4. Developer 🛠️
- DOM skeleton + attributes (id/class/role/data-*)
- Clears script/style content, preserves tag structure
- Use case: "write Playwright script", "verify #app rendered"

### 5. Data 📊
- Tables/lists → JSON
- Discards text paragraphs
- Use case: "extract stock data", "compare product specs"

## Implementation

Single `lol_html` codebase, switches via `DistillMode` enum:

```rust
pub enum DistillMode {
    Reader,    // default — aggressive noise removal
    Operator,  // preserve UI, annotate with @eN refs
    Spider,    // links only → JSON
    Developer, // DOM skeleton with attributes
    Data,      // structured tables/lists → JSON
}
```

Each mode shares the same streaming pipeline, differing only in:
- Reader: most aggressive noise selectors
- Operator: most lenient noise selectors, adds `[Button: ...]` / `[Input: ...]` annotations
- Spider: only processes `<a href>`, ignores all text
- Developer: preserves tag structure, strips script/style content
- Data: only processes `<table>` and lists

## MCP Agent Workflow Example

```
1. Spider mode  → scan homepage, find login link
2. Operator mode → see login form with @e refs
3. act(fill/click) → fill form, submit
4. Reader mode  → fetch internal knowledge base article
5. Data mode    → extract report data as JSON
```
