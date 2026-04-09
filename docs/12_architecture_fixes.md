# 12 Architecture Fixes — Three Critical Gaps

日期: 2026-04-09

## Gap 1: T0/T1 Cookie Sync (Session Brain-Split)

**Problem**: T1 (Chrome) logs in, gets cookies. T0 (reqwest) doesn't have them.
Agent switches to T0 Reader mode → gets bounced to login page (403).

**Fix**: session_sync.rs — extract cookies via CDP `Network.getCookies`,
inject into reqwest `CookieJar`. Bidirectional sync.

## Gap 2: @e Refs in T0 are "Marking a Moving Boat"

**Problem**: T0 generates `@e1` from static HTML. Agent says "click @e1".
But T0 has no browser to click. If we start T1, DOM may have changed,
@e1 might not exist or point to wrong element.

**Fix**: 
- Option A: Operator mode forces T1 route (only live browser gets @e refs)
- Option B: Locator Map — store CSS selector per @eN, use it to find element in T1

## Gap 3: Browser Context Isolation

**Problem**: Shared Chrome daemon. Agent A's Twitter login leaks to Agent B.
No timeout on zombie tabs → memory leak.

**Fix**: AgentSession wraps BrowserContext (incognito). Close context = 
nuke all cookies/state/tabs. One session per Agent task.

## Priority

1. Context isolation (security + memory) — easiest, biggest impact
2. Cookie sync (T0/T1 seamless switching) — enables login→read flows  
3. Locator map (safe @e targeting) — makes Operator+T0 reliable
