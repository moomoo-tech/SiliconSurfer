# Architecture: Two Scenarios on One Engine

## Engine Foundation (Complete)

```
              ┌─────────────────┐  ┌──────────────────┐
              │ Scenario 1:      │  │ Scenario 2:       │
              │ Executor         │  │ Logic Probe       │
              │ Cookie injection │  │ Smoke tests       │
              │ CDP click/form   │  │ DOM snapshot diff  │
              │ Anti-bot         │  │ Regression scan    │
              └────────┬────────┘  └────────┬─────────┘
                       │                     │
              ┌────────┴─────────────────────┴─────────┐
              │         Agent Browser Core              │
              │  T0 (reqwest + lol_html)     ✅         │
              │  T1 (Chrome daemon + CDP)    ✅         │
              │  Router (auto/t0/t1)         ✅         │
              │  Distiller (scraper + lol_html) ✅      │
              └─────────────────────────────────────────┘
```

## Scenario 1: Executor (Productivity Tool)

Agent doesn't just read the web — it operates on it: login, place orders, grab deals.

### 1.1 State Injection: Skip Login Walls

Don't make the Agent go through username/password every time.

- Human or dedicated login module obtains Cookie / Storage State once
- Serialize to JSON, persist
- Rust injects cookies when creating Context — page opens already authenticated
- Skips 80% of tedious steps

**Status: ✅ Implemented** — `BrowserSession.set_cookie()` / `set_cookies_from_json()` via CDP `Network.setCookie`

### 1.2 CDP Interaction

Agent doesn't need pixel coordinates — it operates on the DOM directly.

- Distiller outputs Markdown with @e element references
- Rust sends `Runtime.evaluate` via CDP
- Triggers JS functions in memory, no rendering needed
- Order of magnitude faster than Playwright "wait for render → find coordinates → click"

**Status: ✅ Implemented** — `BrowserSession.click()` / `fill()` / `submit()` in cdp.rs

### 1.3 Verification Loop

- Click "Place Order" → page navigates
- Rust engine instantly fetches new page DOM
- Distiller scans for "Order Confirmed" / order number → success
- Scans for "Out of Stock" / "Payment Failed" → feedback to LLM for retry

### 1.4 Anti-Bot (Future)

Login/ordering scenarios must face anti-fraud:

- **TLS fingerprint spoofing**: reqwest-impersonate ecosystem currently broken (deps yanked)
- **Behavioral simulation**: random delays, mouse trajectories via CDP Input domain
- **CAPTCHA**: third-party solving services or human-in-the-loop

### 1.5 Performance Comparison

| Dimension | Playwright/Selenium | SiliconSurfer |
|-----------|--------------------|--------------------|
| Memory/instance | 200-500MB | 20-50MB |
| Startup time | 500ms-2s | < 50ms (Context) |
| Network traffic | Full site resources | HTML/JSON only |
| Concurrency | Dozens | Thousands |

## Scenario 2: Logic Probe (Development Tool)

Ultra-fast feedback loop after Agent writes code. Not QA — developer self-testing.

### 2.1 Smoke Tests (Sanity Check)

After writing a feature, Agent just needs to confirm:

- **Service is up**: HTTP 200?
- **Component mounted**: DOM has `<div id="stats-chart">`?
- **Data injected**: `window.__INITIAL_STATE__` value correct?

Rust layer completes in 10-30ms. Agent can iterate at second-level speed.

**Status: ✅ Implemented** — `probe.rs` with CSS selector checks, text contains, DOM snapshots

### 2.2 State as Truth

Agent can't see pixels, doesn't need Vision models.

- Read DOM structure directly — 100x faster than screenshot + OCR
- Check `data-timestamp` attribute for correct updates
- Tests "is the logic correct", not "does it look correct"

### 2.3 DOM Snapshot Diff (Zero Regression)

Agent maintains a DOM snapshot library:

- Before code change: scan key pages' DOM structure, store snapshot
- After code change: scan again, diff compare
- Navigation node missing? Caught instantly, revert the CL

**Status: ✅ Implemented** — `snapshot()` + `diff_snapshots()` in Python probe API

## Shared Tech Stack

| Component | Scenario 1 | Scenario 2 | Status |
|-----------|-----------|-----------|--------|
| T0 reqwest + lol_html | Fetch pages/APIs | Smoke tests | ✅ |
| T1 Chrome daemon + CDP | Interactive ops | JS-rendered page verification | ✅ |
| Distiller | Extract order results | DOM snapshots | ✅ |
| Cookie injection | Skip login | Auth'd page testing | ✅ |
| CDP interaction (click/form) | Place orders | - | ✅ |
| DOM snapshot diff | - | Regression detection | ✅ |
| Python Agent API | tool calling | tool calling | ✅ |
