# sisurf 🏄

> The MCP-compatible browser built for silicon-based lifeforms.

English | [中文](README_CN.md)

## Why Not Playwright MCP?

Playwright MCP gives LLM raw HTML (25,000 tokens of noise). sisurf gives LLM **finished data**:

| | Playwright MCP | sisurf |
|---|---|---|
| Read a page | Raw HTML, 25K tokens | Clean Markdown, 5K tokens |
| Find form fields | LLM parses HTML | `@e3 [Input: name=username]` |
| Get all links | LLM searches `<a>` tags | `observe(mode="spider")` → JSON |
| Extract table | LLM parses `<table>` | `observe(mode="data")` → JSON rows |
| Modes | 1 (everything) | 5 (Reader/Operator/Spider/Developer/Data) |
| Speed | Browser startup per call | 1ms T0, shared daemon T1 |

**Results: 30/30 eval (vs Jina 20/30), 5/5 E2E (vs browser-use 0/5), 6.2x faster.**

## MCP Tools

Two tools, five vision modes:

```
observe(url, mode)   # See a webpage
  mode="reader"      → Clean Markdown (default)
  mode="operator"    → @e1 @e2 @e3 interactive element refs
  mode="spider"      → JSON link map {nav, content, footer}
  mode="data"        → Structured JSON tables/lists
  mode="developer"   → DOM skeleton with attributes

act(action, target, value)   # Interact with the page
  act("navigate", url)       → Go to URL
  act("click", "@e3")        → Click element
  act("fill", "@e1", "admin")→ Fill form field
  act("submit", "@e5")       → Submit form
  act("set_cookies", "", '[{"name":"session","value":"abc","domain":".example.com"}]')
```

Workflow: `observe(mode="operator")` → see elements → `act("click", "@e3")` → `observe` again.

## Quick Start

```bash
# Build Rust + PyO3 bindings
uv sync --dev

# Use with Claude Code — add .mcp.json, restart
uv run python mcp_server.py
```

Or run as HTTP server:

```bash
cargo build --release -p sisurf-server
PORT=9883 ./target/release/sisurf-server
```

## Design Philosophy

AI Agents need to fetch information and perform actions on the web, but don't need CSS rendering, visual debugging, or other human-facing features.

sisurf lets AI see the web **the silicon way** — 5 vision modes, @e element references, millisecond response times.

## Two-Tier Architecture

Automatically selects the optimal fetching strategy based on page complexity:

```
              Agent / LLM request
                    │
                    ▼
            ┌──────────────┐
            │ Routing Engine │
            └──┬────────┬──┘
               │        │
     Static    │        │  Dynamic (SPA / JS-rendered / interactive)
               ▼        ▼
        ┌──────────┐  ┌─────────────────┐
        │ T0: Light │  │ T1: Headless     │
        │ reqwest   │  │ Chromium + CDP   │
        └─────┬────┘  └───────┬─────────┘
              │               │
              ▼               ▼
        ┌─────────────────────────────┐
        │      Distiller               │
        │  HTML → clean Markdown/JSON  │
        └─────────────────────────────┘
                    │
                    ▼
              Agent / LLM
```

### T0: Lightweight — reqwest (static pages)

Pure HTTP requests, no browser. For:

- Static HTML pages, blogs, documentation sites
- Open APIs, RSS feeds
- Pages with no JS rendering dependencies

Features:
- Thousands of concurrent requests per machine
- Minimal memory footprint (KB per request)
- Millisecond response times

### T1: Headless Browser — Chromium + CDP (dynamic pages)

Heavily stripped-down Chromium kernel, keeping only the JS engine and DOM parser. For:

- SPA apps (React/Vue/Angular)
- Pages that load data via AJAX/WebSocket
- Login, click, form interaction scenarios
- Pages requiring JS execution to render content

Features:
- Single global daemon process, millisecond context creation/destruction
- Intercepts CSS/images/fonts/media — only keeps JS execution and DOM
- Supports interaction: navigate, click, fill forms, submit
- Persistent BrowserSession: observe and act share the same tab
- Cookie injection: skip login flows by injecting session cookies

### Routing Logic

```rust
match mode {
    FetchMode::T0   => fetch_t0(url),      // reqwest → distill
    FetchMode::T1   => fetch_t1(url),      // Chrome → distill
    FetchMode::Auto => {                   // T0 first, fallback T1
        let result = fetch_t0(url);
        if result.content_length < 100 { fetch_t1(url) }
        else { result }
    }
}
```

## Distiller

The common exit point for both tiers. Regardless of how HTML is fetched, it goes through the same cleaning pipeline:

1. **DOM noise removal**: strips `<nav>`, `<footer>`, `<script>`, `<style>`, ad containers
2. **Content targeting**: locks onto `<article>`, `<main>`, or primary content `<div>`
3. **Format conversion**: outputs clean Markdown or structured JSON
4. **Token compression**: compresses hundreds of thousands of raw HTML chars into hundreds to thousands of high-density tokens

Dual engine: `scraper` (DOM AST) for precise extraction, `lol_html` (streaming) for high-speed batch processing (6.76ms/500KB).

## Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust | Memory safety, zero-cost abstractions, extreme concurrency |
| Async runtime | tokio | De facto standard in Rust ecosystem |
| T0 HTTP client | reqwest | Fast requests, gzip/brotli/deflate |
| T1 CDP | chromiumoxide | Most mature CDP wrapper in Rust |
| HTML parsing | scraper + lol_html | AST precision + streaming speed |
| Serialization | serde + serde_json | High-performance JSON output |
| Python bridge | PyO3 | Rust → Python FFI, zero network overhead |
| HTTP API | axum | Lightweight HTTP server |

## API

```bash
# T0 fetch static page (returns Markdown)
curl http://localhost:9883/fetch \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Specify engine and distill mode
curl http://localhost:9883/fetch \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "mode": "t1", "distill": "operator"}'

# Distill raw HTML directly
curl http://localhost:9883/distill \
  -H "Content-Type: application/json" \
  -d '{"html": "<html>...</html>", "distill": "reader"}'

# DOM probe
curl http://localhost:9883/probe \
  -H "Content-Type: application/json" \
  -d '{"url": "http://localhost:3000", "checks": [{"selector": "#app"}]}'
```

## Rust Library API (`sisurf-core`)

Two clean primitives a downstream consumer links directly — no server, no MCP.
Both return one typed error (`WebError`) and retry transient failures internally.

### `fetch(url) → Page`

```rust
use sisurf_core::{fetch, FetchOpts, Page, Tier, WebError};

let page: Page = fetch("https://example.com", FetchOpts::default()).await?;
// Page { url, title: Option<String>, content: String, content_length, tier: Tier }
// tier == Tier::Static  → served by reqwest, no browser
// tier == Tier::Browser → escalated to headless Chromium
```

**Fastest-first escalation (the caller never picks the tier).** `fetch` tries the
static reqwest path first and only escalates to headless Chromium when the page is
flagged as JS-only in a site profile (`force_t1`) **or** the static body comes back
as an unrendered shell (empty / a `Loading…` placeholder). If escalation is needed
but no browser is available, it **falls back to the static result** rather than
failing.

**`NoBrowser` + the Chromium requirement.** JS-rendered pages need a Chromium-family
browser (Chrome / Chromium / Edge / Brave). sisurf auto-detects one on `PATH` and in
the usual install locations; set **`CHROME_PATH`** to override. When a page genuinely
requires the browser tier and none can be launched, the error is the typed
`WebError::NoBrowser { hint }` — branch on it to fall back or tell the user to install
a browser. Transient faults (timeout, 5xx, connection reset, a mid-session browser
crash) retry with backoff; permanent ones (4xx, invalid URL, `NoBrowser`) do not.

### `search(query) → Vec<SearchResult>`

```rust
use sisurf_core::{search, SearchOpts, SearchResult};

let hits: Vec<SearchResult> = search("rust async runtime", SearchOpts::default()).await?;
// SearchResult { title: String, url: String, snippet: String }
```

**"Each search engine is a dialect."** A `SearchDialect` trait factors every engine
into two steps — `build_request` (assemble the HTTP request) and `parse_response`
(map the raw body → results). A shared executor drives any dialect over the same
fetch/escalation/retry stack, so adding an engine is one small `impl`. `parse_response`
distinguishes a legitimately empty result set (`Ok(vec![])`) from changed markup /
schema (`Err(WebError::SearchParse)`) — never a silent empty.

### Backends & the `[web_search]` config schema

sisurf **owns** which backends exist and what each needs (the `SearchConfig` serde
schema); the **consumer deserializes the TOML and injects the resolved API key** —
sisurf is a library and never reads env vars or files itself.

| Backend (`backend = …`) | Key? | When to use |
|---|---|---|
| `"ddg"` *(default)* | none | Keyless DuckDuckGo HTML scrape. Zero setup. |
| `"google_cse"` | API key + `cx` | **Recommended.** Google Programmable Search JSON API, 100 queries/day free. |
| `"brave"` | API key | Brave Search API. |

```toml
[web_search]
backend = "google_cse"          # "ddg" (default) | "google_cse" | "brave"

[web_search.google_cse]
cx = "0123abc..."               # Programmable Search Engine id — committed, not a secret
# api_key is injected by the consumer at load time (never committed)

[web_search.brave]
# api_key injected by the consumer
```

```rust
use sisurf_core::{SearchConfig, SearchOpts};

// Consumer deserializes the [web_search] section, injects the key, then:
let backend = cfg.build()?;      // typed MissingApiKey / BackendConfig on misconfig
let hits = search("query", SearchOpts { backend, ..Default::default() }).await?;
```

`SearchConfig::build()` fails typed — `WebError::MissingApiKey` when a selected backend
has no key, `WebError::BackendConfig` when a section (`[web_search.google_cse]`) or the
`cx` id is missing — so a misconfiguration is a clear error, never a silent fallback.

## License

Apache-2.0
