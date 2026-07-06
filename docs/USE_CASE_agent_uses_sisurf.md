# Use Case: An Agent Reads the Web Through sisurf

*All I/O below is captured verbatim from a live run on 2026-07-05 (server `:9883`). No numbers are invented.*

## Scenario

An LLM agent must research current model pricing across four provider docs sites
(Anthropic, OpenAI, Gemini, DeepSeek/xAI). It has **no** raw-HTTP or browser UI —
its only web tool is sisurf. This is the exact run that produced our
`models.toml` source data; one leg was driven by a *subagent* that was told nothing
but "you have this HTTP tool," and it completed the task in 7 calls / ~100s / 0 errors.

---

## 1. How the agent issues a command

One HTTP POST. That's the whole interface.

```bash
curl -s -X POST http://127.0.0.1:9883/fetch \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://api-docs.deepseek.com/quick_start/pricing",
       "mode":"auto",          # t0 | t1 | auto  (fetch tier)
       "fast":true,            # AST engine (false) | stream engine (true)
       "distill":"reader"}'    # reader|spider|operator|data|developer
```

Three orthogonal knobs the agent controls:

| knob | values | meaning |
|---|---|---|
| `mode` | `t0` / `t1` / `auto` | **how to fetch** — t0 = raw HTTP (~1ms, no JS), t1 = headless Chrome (renders JS), auto = t0 then fall back to t1 if sparse |
| `distill` | `reader` / `spider` / `operator` / `data` / `developer` | **what shape to read back** |
| `fast` | `false` / `true` | which distill engine (see Bug #2 — this one bites) |

## 2. How the agent reads data back

A flat JSON envelope — no HTML parsing on the agent side, ever:

```json
{
  "url": "https://api-docs.deepseek.com/quick_start/pricing",
  "title": "Models & Pricing | DeepSeek API Docs",
  "content": "# Models & Pricing\n\nThe prices listed below are ...",
  "content_length": 2160,
  "mode_used": "t0"        // ← tells the agent which tier actually served it
}
```

The agent reads `content` directly into its context. `mode_used` is the honest
receipt of what happened (t0 vs t1), so the agent can reason about freshness/cost.
On failure the envelope carries `{"error": "..."}` — a real error string, never a
sentinel stuffed into `content`.

## 3. HTML vs the distill formats (real samples, same URL)

The agent never touches HTML. It picks a **distill mode** and gets purpose-built data:

**`reader`** — LLM markdown (the default, for "just read it"):
```
# Models & Pricing
MODEL deepseek-v4-flash  deepseek-v4-pro
PRICING 1M INPUT (CACHE MISS) $0.14  $0.435
```

**`spider`** — JSON link topology (for "map the site / find more pages"):
```json
{"content_links":[{"text":"Skip to main content","url":"https://api-docs.deepseek.com/quick_start/#..."}, ...]}
```

**`operator`** — `@e` element refs (for "act on this page" — click/fill/submit):
```
@e1 [Skip to main content](...)
@e2 [Button: ]
@e3 [[image: DeepSeek API Docs Logo](...)]
```

**`data`** — structured JSON tables/lists (for "give me the rows, not prose"):
```json
{"lists":[["English","中文（中国）"],["Your First API Call","Models & Pricing", ...]]}
```

**`developer`** — DOM skeleton with attributes (for "I need the structure"):
```html
<body class="navigation-with-keyboard">
  <div id="__docusaurus">...
```

## 4. Is the data clean? (measured, not asserted)

Same DeepSeek pricing page, raw HTML vs sisurf `reader`:

| | bytes | tags | `<script>` |
|---|---|---|---|
| raw HTML (what a naive scraper eats) | 21,219 | 532 | 3 |
| sisurf `reader` | 2,128 | 0 | 0 |

→ **90.0% stripped, ~10× smaller, zero markup/script noise.** The 2,128 bytes are
all signal: model ids, thinking mode, context length, and every price cell survived
intact (verified against the live page). That is clean.

---

## Dogfooding findings (this run surfaced two real bugs)

**Bug #1 — `auto` doesn't escalate a JS "Loading…" shell to t1.**
Anthropic `docs.claude.com` served a 142-byte SPA shell containing only `Loading...`.
`auto` accepted it and returned it as success; the agent had to *manually* force
`mode:t1` to get real content. The sparse-content fallback threshold is byte/word
based, so a short-but-empty skeleton slips through. Fix: treat placeholder skeletons
(tiny body, `Loading…`, empty `<main>`) as sparse → trigger T1. (`router.rs`)

**Bug #2 — `distill` mode is silently ignored unless `fast:true`.**
With `fast:false` (the default), `spider` / `operator` / `data` all returned the
*identical* `reader` markdown (byte-identical, len 2128). Only with `fast:true` do the
five modes actually dispatch (distinct md5s / shapes, confirmed). So an agent that
asks for `operator` refs without knowing to also set `fast:true` gets reader markdown
back — **with no error** — and can't tell. The default AST distiller drops the mode
param on the floor. Fix: either honor `distill` in the AST path too, or return an
error when a mode the active engine can't serve is requested. (`distiller.rs` /
`fetcher.rs`)

Both are "silent success on wrong output" — the worst failure class for an agent,
because it has no signal to retry.
