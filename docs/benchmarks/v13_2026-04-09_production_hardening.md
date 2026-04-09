# Benchmark v13 — 2026-04-09 — Production Hardening (8 Bugs)

Git: `278e25c`

## 8 Production Bugs — 7 Fixed, 1 Deferred

| # | Bug | Impact | Fix | Status |
|---|-----|--------|-----|--------|
| 1 | Event loop collision | PyO3 panic | run_async() thread detection | ✓ |
| 2 | Zombie Chrome | RAM leak | PID file + atexit + cleanup | ✓ |
| 3 | Iframe blindspot | Missing content | Flatten same-origin iframes | ✓ |
| 4 | @e locator drift | Wrong element | Clear map after act() | ✓ |
| 5 | Ghost text (display:none) | Wrong data | Remove invisible DOM before extraction | ✓ |
| 6 | Anti-bot detection | 403 blocked | Stealth patches before navigation | ✓ |
| 7 | Token explosion | Cost/accuracy | Token circuit breaker | 🟡 TODO |
| 8 | CDP WebSocket timeout | Hang/crash | tokio::time::timeout(10s) | ✓ |

## Stealth Patches (Bug 6)

Injected before first navigation via `inject_stealth()`:
- `navigator.webdriver` → undefined
- Fake plugins array (5 plugins)
- Fake languages (en-US, en)
- Permissions API fix
- `chrome.runtime` object

## Ghost Text Removal (Bug 5)

Before extracting HTML from Chrome, inject JS to remove:
- `display: none` elements
- `visibility: hidden` elements
- `opacity: 0` elements
- `aria-hidden="true"` elements

Agent never sees prices/text that humans can't see.

## Token Cost Analysis

| Approach | Tokens/page | Pages per $0.25 | Cost per page (CNY) |
|----------|-------------|-----------------|---------------------|
| **SiliconSurfer** | ~4,000 | **~250** | **¥0.007** |
| Raw HTML | ~25,000 | ~40 | ¥0.045 |
| Screenshot multimodal | ~10,000 | ~100 | ¥0.018 |

SiliconSurfer: 6x cheaper than raw HTML, 2.5x cheaper than screenshot.

## Chrome Performance Mode

Two modes via `BrowserPool`:
- **Performance (default)**: No images, fonts, GPU, canvas, background services
- **Vision (on demand)**: Full rendering for captcha/charts/screenshots

## Architecture After All Fixes

```
                Claude / LLM
                    ↓ MCP (observe + act)
                mcp_server.py
                    ↓ HTTP
                Rust Server
                    ↓
    ┌──────────────────────────────────────┐
    │  session.rs (AgentSession)           │
    │    ├─ inject_stealth()     [Bug 6]   │
    │    ├─ remove ghost text    [Bug 5]   │
    │    ├─ flatten iframes      [Bug 3]   │
    │    ├─ observe() → locator map        │
    │    ├─ act() → timeout wrap [Bug 8]   │
    │    └─ clear map after act  [Bug 4]   │
    ├──────────────────────────────────────┤
    │  browser.rs (BrowserPool)            │
    │    ├─ performance_mode toggle        │
    │    └─ zombie cleanup       [Bug 2]   │
    ├──────────────────────────────────────┤
    │  python/lib.rs (PyO3)                │
    │    └─ run_async() safe     [Bug 1]   │
    ├──────────────────────────────────────┤
    │  strategy/ (5 modes)                 │
    │  profiles.toml (9 sites)             │
    │  distiller_fast.rs (6.76ms/500KB)    │
    └──────────────────────────────────────┘
```

## Complete Project Stats

- **Rust tests**: 40 pass
- **Benchmarks**: v0 → v13 (13 versions in 1 day)
- **Distiller speed**: 264ms → 6.76ms (39x)
- **E2E Agent**: 5/5 in 34.4s
- **vs browser-use**: 5/5 vs 0/5, 6.2x faster
- **vs Jina**: 30/30 vs 20/30, 27x faster
- **vs Trafilatura**: 30/30 vs 15/30
- **MCP**: 2 tools (observe + act), working in Claude Code
- **Production bugs**: 7/8 fixed
- **Site profiles**: 9 configured
- **Chrome modes**: Performance + Vision
