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

## Priority

1. Bug 4 (@e drift) — easiest to fix, most likely to hit in E2E
2. Bug 2 (zombie Chrome) — critical for production
3. Bug 1 (event loop) — blocks PyO3 T1 usage
4. Bug 3 (iframe) — edge case but important for payment/login
