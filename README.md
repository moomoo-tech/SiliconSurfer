# SiliconSurfer 🏄

> The MCP-compatible browser built for silicon-based lifeforms.

English | [中文](README_CN.md)

## Why Not Playwright MCP?

Playwright MCP gives LLM raw HTML (25,000 tokens of noise). SiliconSurfer gives LLM **finished data**:

| | Playwright MCP | SiliconSurfer |
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
cargo build --release -p agent-browser-server
PORT=9883 ./target/release/agent-browser-server
```

## Design Philosophy

AI Agents need to fetch information and perform actions on the web, but don't need CSS rendering, visual debugging, or other human-facing features.

SiliconSurfer lets AI see the web **the silicon way** — 5 vision modes, @e element references, millisecond response times.

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

## License

Apache-2.0
