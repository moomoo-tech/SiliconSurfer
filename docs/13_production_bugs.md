# Production Bugs — Status Tracker

## Fixed

| Bug | Description | Fix |
|-----|-------------|-----|
| Bug 1 | Tokio/Asyncio event loop collision | `run_async()` with thread detection in PyO3 |
| Bug 2 | Zombie Chrome processes | PID file + cleanup + atexit in mcp_server.py |
| Bug 3 | Iframe cross-origin blindspot | Flatten same-origin frames, mark cross-origin in session.rs |
| Bug 4 | @e locator drift on SPA | Clear locator_map after act(), force re-observe |
| Bug 5 | Ghost text (display:none) | T1: JS injection removes invisible elements before extraction |
| Bug 6 | Anti-bot detection | Stealth patches in session.rs (webdriver, plugins, languages, chrome.runtime) |
| Bug 8 | CDP WebSocket broken pipe | Timeout wrappers on CDP calls |
| Bug 9 | target="_blank" new tab black hole | JS injection rewrites to _self in session.rs |
| Bug 10 | JS dialog deadlock | Override alert/confirm/prompt in session.rs |
| Bug 11 | Shadow DOM invisibility | Shadow Piercer: recursive JS to tag+flatten shadowRoot |
| Bug 12 | networkidle trap | MutationObserver quiescence (500ms no changes) in session.rs |
| Bug 13 | Mutex chokehold (browser.rs) | Scope lock to new_page() only, drop before async I/O |
| Bug 14 | act() was mock/stub | Real CDP via BrowserSession PyO3 bindings |
| Bug 15 | JS injection escaping | serde_json::to_string for all user input in cdp.rs |

## Open

| Bug | Description | Priority |
|-----|-------------|----------|
| Bug 7 | Token explosion on infinite scroll (Reddit/Twitter 50K+) | Deferred — needs truncation strategy |
| — | reqwest-impersonate TLS fingerprint integration | Future |
| — | Large page content truncation (Wikipedia 134K chars) | Future |
