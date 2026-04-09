# 13 Production-Grade Bugs — Must Fix Before Deploy

## Bug 1: Tokio/Asyncio Event Loop Collision ⚠️ HIGH

PyO3 `block_on()` inside Python asyncio = panic or deadlock.
Fix: Dedicated OnceLock<Runtime> in Rust, never nest runtimes.
Status: Partially done (OnceLock exists in python/src/lib.rs), but T1 browser 
calls cause nested runtime panic. Need to move all browser ops to dedicated thread.

## Bug 2: Zombie Chrome Processes 🔴 CRITICAL

MCP server crash / SIGKILL → Chrome orphan processes eat RAM forever.
Fix: OS-level parent death signal (PR_SET_PDEATHSIG on Linux).
Also: PID file + cleanup on startup.
Status: Not implemented.

## Bug 3: Iframe Cross-Origin Blindspot ⚠️ MEDIUM

page.content() only returns main frame DOM.
Stripe payment, Google login, captcha widgets in iframes = invisible.
Fix: Iterate page.frames(), extract each frame's DOM, flatten into main HTML.
Status: Not implemented.

## Bug 4: @e Locator Drift on SPA ⚠️ HIGH  

React/Vue partial DOM update after click → cached @e selectors point to stale elements.
Fix: Invalidate locator_map after every act(). Force re-observe before next act.
Status: Partially mitigated (session.rs rebuilds map on observe), but not enforced.

## Status

- Bug 1 (event loop) ✓ Fixed — run_async() with thread detection
- Bug 2 (zombie Chrome) ✓ Fixed — PID file + cleanup + atexit
- Bug 3 (iframe) ✓ Fixed — flatten same-origin, mark cross-origin
- Bug 4 (@e drift) ✓ Fixed — clear map after act, force re-observe

## Bug 5: Ghost Text (display:none) 🔴

lol_html extracts hidden elements (display:none, opacity:0).
Agent sees two prices, picks wrong one.
Fix (T1): inject JS to remove invisible elements before extraction.
Fix (T0): no fix possible (no rendering engine).

## Bug 6: Cloudflare / Anti-bot Detection 🔴

T1 Chrome has obvious bot fingerprints (navigator.webdriver=true).
Fix: Page.addScriptToEvaluateOnNewDocument with stealth patches.
Similar to puppeteer-extra-plugin-stealth.

## Bug 7: Token Explosion on Infinite Scroll 🟡 TODO (discuss later)

Reddit/Twitter DOM = 50K tokens of Markdown.
Fix: Token circuit breaker — truncate at 8K tokens, add scroll instruction.
Deferred for architectural discussion.

## Bug 8: CDP WebSocket Broken Pipe 🔴

Chrome JS blocking → WebSocket timeout → Rust panic.
Fix: tokio::time::timeout on all CDP calls + retry/self-heal.

## Priority (remaining)

1. Bug 5 (ghost text) — data accuracy
2. Bug 6 (anti-bot) — site coverage
3. Bug 8 (CDP timeout) — stability
