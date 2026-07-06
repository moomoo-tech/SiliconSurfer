# Design: SiliconSurfer Agentic Runtime

> A self-contained agent loop — **LLM plans → act → observe → repeat until goal** —
> that drives SiliconSurfer as its browser, selects its model from `models.toml`,
> and is validated end-to-end on **WebArena**.

Status: **Design** (development-ready). Grounded in the codebase at the commit where
`router.rs` auto-fallback and `fetcher.rs` distill-mode dispatch were just fixed.

---

## 1. Overview & goal

Today SiliconSurfer is a **body without a brain**: it exposes `observe`/`act` tools
(`mcp_server.py`) and stateless fetch tools (`python/agent_browser/agent_api.py`), but the
planning loop lives *outside*, in whatever host drives the MCP session (Claude Code).
There is no in-repo agent that, given a goal, autonomously loops observe→decide→act to
completion.

**Goal.** Add an in-repo **Agentic Runtime**: given `(goal, start_url)` it runs a bounded
observe→plan→act loop using an LLM chosen from the `models.toml` registry, and returns a
result plus a full trajectory. Prove it against WebArena's functional-correctness suite.

**Non-goals.**
- Not a new browser engine — reuse `AgentSession` (session.rs) verbatim.
- Not multi-agent swarm (that is TODO.md V3.0 "Planner→Surfer→Critic→Memory"); this is the
  single-agent substrate that swarm will later compose.
- Not a hosted service — local process, same as the MCP path (docs/14).
- Not fine-tuned Action-SLM (TODO.md V3.0) — we consume hosted LLM APIs.

---

## 2. CUJs (Critical User Journeys)

| id | actor | trigger | steps | success outcome |
|---|---|---|---|---|
| **CUJ-1** | App developer | `runtime.run(goal, start_url)` | agent observes page, LLM emits an action, runtime executes it, repeats until the LLM emits `finish` or a budget cap | returns `{answer, trajectory, tokens, cost, steps}`; goal achieved |
| **CUJ-2** | App developer | passes `provider="anthropic"` or `tier="small"` (or nothing) | registry resolves a concrete model id + endpoint; loop routes calls there; token cost accrues from registry prices | correct model used; `cost` reflects registry pricing |
| **CUJ-3** | Agent researcher | `webarena-eval --tasks 0-811` (or a subset) | harness boots WebArena sites, runs the runtime per task, scores with each task's programmatic checker | prints `success_rate`, writes per-task trajectories + verdicts |
| **CUJ-4** | App developer | goal requires login/form (e.g. "sign in and post") | agent uses `observe(operator)` to get `@e` refs, `act(fill,@e1,..)`, `act(click,@e3)`, re-observes | multi-step form completed; `@e` refs resolve to live elements |
| **CUJ-5** | App developer | a page is a JS SPA / needs auth cookies | loop uses the persistent stateful session (cookies + T1 render survive across steps) | later steps see logged-in state; SPA shells render (no `Loading…` stall) |

---

## 3. Feature list (derived from CUJs)

| id | feature | serves |
|---|---|---|
| F1 | Bounded agent loop (observe→plan→act, stop conditions) | CUJ-1, CUJ-4 |
| F2 | Model registry read + resolution (default/tier/alias) | CUJ-2 |
| F3 | Provider-router LLM client (OpenAI-compat + Anthropic-compat), tool-calling | CUJ-1, CUJ-2 |
| F4 | Observe/act tool schemas presented to the LLM | CUJ-1, CUJ-4 |
| F5 | Stateful session reuse across loop steps (cookies, T1, `@e` live-DOM stamping) | CUJ-4, CUJ-5 |
| F6 | Trajectory + token/cost accounting | CUJ-1, CUJ-2, CUJ-3 |
| F7 | WebArena action-space adapter | CUJ-3 |
| F8 | WebArena eval harness (loader, runner, scorer, report) | CUJ-3 |
| F9 | `AgentSession` exposed to Python (PyO3) — prerequisite for F5 | CUJ-4, CUJ-5 |

---

## 4. Requirements

### Functional
| id | requirement | feature |
|---|---|---|
| FR-1 | `run(goal, start_url, *, model=None, provider=None, tier=None, max_steps=30, token_budget=None)` executes the loop and returns a `RunResult`. | F1 |
| FR-2 | Each step: build a prompt from the latest `ObserveResult` + goal + history; call the LLM with the observe/act tool schemas; parse exactly one tool call; execute it. | F1, F4 |
| FR-3 | Loop terminates on: LLM `finish(answer)`, `max_steps`, `token_budget` exhaustion, or unrecoverable session error. Every termination is typed (`StopReason` enum), never a silent stall. | F1 |
| FR-4 | Registry loader resolves a model from (explicit id) → (provider default) → (tier match) and returns `ModelSpec{id, endpoint, price_in, price_cached, price_out, thinking, off_knob}`. Unknown/ambiguous input raises, does not guess. | F2 |
| FR-5 | LLM client dispatches by endpoint family: `base_url_openai` → OpenAI Chat Completions tool-calling; `base_url_anthropic` → Anthropic Messages tool-use. Returns a normalized `LLMTurn{text, tool_call, usage}`. | F3 |
| FR-6 | `observe` tool ⇒ `AgentSession::observe(mode)`; `act` tool ⇒ `AgentSession::act(action,target,value)`. `@e` refs resolve against the live-DOM stamps written by observe. | F4, F5, F9 |
| FR-7 | WebArena adapter maps its action grammar (`click [id]`, `type [id] [text]`, `goto [url]`, `go_back`, `scroll [dir]`, `stop [answer]`) onto `act`/`observe`, and maps WebArena's a11y-tree observation onto `observe(operator)`. | F7 |
| FR-8 | Harness loads WebArena task configs, runs one runtime per task, invokes the task's `eval` (functional checker), and records pass/fail + trajectory. | F8 |
| FR-9 | `RunResult` carries `{answer, steps:[{observe, action, usage}], stop_reason, total_tokens, total_cost_usd}`. Cost = Σ usage × registry price (fresh vs cached split). | F6 |

### Non-functional
| id | requirement | threshold | feature |
|---|---|---|---|
| NFR-1 (perf) | Per-step overhead excluding LLM latency (observe distill + act dispatch) | ≤ 200 ms p50 on a rendered page (distill is 6.76 ms/500 KB baseline, README) | F1 |
| NFR-2 (reliability) | A JS SPA shell must not stall the loop | `observe` inherits router auto-escalation; a `Loading…`-only page escalates to T1 (router.rs `is_sparse_content`) | F5 |
| NFR-3 (reliability) | Stale `@e` after `act` must never silently mis-click | `act` clears `locator_map` (session.rs:477); runtime MUST re-`observe` before next `@e` act; adapter enforces this | F5 |
| NFR-4 (cost) | Every run reports token cost; a `token_budget` cap is honored as a hard ceiling | run aborts with `StopReason.BudgetExhausted` before exceeding | F6 |
| NFR-5 (security) | LLM-proposed actions are constrained to the tool schema; no arbitrary JS eval from model output | `act` enum is closed (click/fill/submit/navigate/set_cookies); `eval` (cdp.rs:230) is NOT exposed to the model | F4 |
| NFR-6 (scale) | Harness runs N WebArena tasks with bounded concurrency without cross-task state bleed | one `AgentSession` per task; no shared mutable globals | F8 |

---

## 5. Infra

| need | exists? | where |
|---|---|---|
| Headless Chrome pool (T1) | ✅ | `BrowserPool` (browser.rs), shared via `Engine::browser_pool()` |
| Distiller (5 modes) | ✅ | `FastDistiller::distill` (distiller_fast.rs:47) |
| Stateful session w/ stealth + live-DOM `@e` stamps | ✅ (Rust only) | `AgentSession` (session.rs) — **not exposed to Python** |
| PyO3 bridge | ✅ (partial) | `agent_browser` module (lib.rs) exposes `BrowserSession` only |
| Model registry | ✅ (new, this design) | `models.toml` (repo root, already synthesized w/ real 2026 data) |
| LLM provider SDKs | ➕ new dep | `openai`, `anthropic` Python packages (both speak the two endpoint families in the registry) |
| WebArena sites | ➕ external | Docker images (shopping / reddit / gitlab / cms / map / wiki), self-hosted per WebArena README |
| Secrets | ✅ pattern | `config.toml` (gitignored) already holds API keys per CLAUDE.md |

---

## 6. Components

> Convention: the runtime is **Python** (`python/agent_browser/runtime/`), because the LLM
> SDKs, the existing tool surfaces (`agent_api.py`, `mcp_server.py`), and WebArena's harness
> are all Python. The one Rust change is F9 (expose `AgentSession`).

### C1 — Model Registry (`python/agent_browser/runtime/registry.py`) — NEW
- **Responsibility:** parse `models.toml`, resolve a model, expose pricing for cost accounting.
- **Reuses:** `models.toml` (repo root, this design). Parsing via `tomllib` (stdlib).
- **Interface:**
  ```python
  @dataclass
  class ModelSpec:
      provider: str; id: str; endpoint: str; endpoint_family: str  # "openai"|"anthropic"
      price_in: float; price_cached: float | None; price_out: float
      context: int | None; thinking: str; off_knob: str | None
  class Registry:
      @classmethod
      def load(cls, path="models.toml") -> "Registry": ...
      def resolve(self, *, model=None, provider=None, tier=None) -> ModelSpec: ...  # FR-4
  ```

### C2 — LLM Client / Provider Router (`runtime/llm.py`) — NEW
- **Responsibility:** given a `ModelSpec` + messages + tool schemas, return one normalized turn.
- **Reuses:** `ModelSpec.endpoint`/`endpoint_family` from C1; `off_knob`/`thinking` to set the
  disable-thinking param correctly per generation (registry already encodes e.g. Gemini
  `thinking_level=minimal` vs 2.5 `thinkingBudget=0`, Anthropic `thinking.type=disabled`).
- **Interface:**
  ```python
  @dataclass
  class LLMTurn: text: str; tool_call: ToolCall | None; usage: Usage  # fresh/cached/out tokens
  class LLMClient:
      def __init__(self, spec: ModelSpec, api_key: str): ...
      def step(self, messages: list[dict], tools: list[dict]) -> LLMTurn: ...  # FR-5
  ```

### C3 — Agent Loop / Runtime (`runtime/agent.py`) — NEW (the core)
- **Responsibility:** the observe→plan→act loop, stop conditions, trajectory + cost.
- **Reuses:** C1 (resolve model), C2 (LLM turns), C4 (tool schemas), C5/F9 (`AgentSession` via
  PyO3). Cost math uses `ModelSpec` prices split fresh vs cached (registry mirrors the
  Anthropic fresh/cache-read split noted during research).
- **Interface:**
  ```python
  class StopReason(enum.Enum): FINISHED; MAX_STEPS; BUDGET_EXHAUSTED; SESSION_ERROR
  @dataclass
  class RunResult: answer: str|None; steps: list[Step]; stop_reason: StopReason
                   total_tokens: int; total_cost_usd: float
  class Runtime:
      def __init__(self, registry: Registry, session_factory=AgentSession): ...
      def run(self, goal: str, start_url: str, *, model=None, provider=None, tier=None,
              max_steps=30, token_budget=None) -> RunResult: ...  # FR-1..3, FR-9
  ```

### C4 — Tool Schemas (`runtime/tools.py`) — NEW, but shapes REUSED
- **Responsibility:** the `observe` + `act` + `finish` tool JSON-schemas handed to the LLM.
- **Reuses:** copy the exact schemas already shipped in `mcp_server.py:126-171` (observe modes
  enum, act action enum, target/value semantics) so the model-facing contract is identical to
  the proven MCP one. Adds a `finish(answer)` terminal tool (new).

### C5 — `AgentSession` PyO3 exposure (`crates/python/src/lib.rs`) — NEW Rust binding (F9)
- **Responsibility:** expose the *stamping* session to Python so `@e` refs resolve on live DOM.
- **Reuses:** wraps `AgentSession::new/navigate/observe/act/close` (session.rs:42/53/209/391/492),
  returning `ObserveResult`/`ActResult` (session.rs:24-39) as PyDicts — mirror the existing
  `Session` pyclass pattern (lib.rs:240-386: `Arc<TokioMutex<..>>`, `py.detach`, `run_async`,
  `action_result_to_dict` lib.rs:399).
- **Why not reuse the existing `Session`:** the current PyO3 `Session` wraps `BrowserSession`
  (cdp.rs), whose `see()` (cdp.rs:103) distills the HTML *string* and stamps `data-agent-id`
  only in the returned markdown — **the live DOM is never stamped**, so `click_agent_ref`
  (cdp.rs:139 → `[data-agent-id='eN']`) can miss. `AgentSession::observe` stamps the live DOM
  via the shadow-piercer (session.rs:253) and keeps a `locator_map` (session.rs:19). F5/CUJ-4
  require this. **This is the highest-risk dependency — do it first (see roadmap M1).**
- **Interface (new pyclass `AgentSession`):**
  ```
  observe(mode: str) -> dict   # {content,title,url,content_length,mode,element_count}
  act(action: str, target: str, value: str) -> dict   # {success,url,detail}
  navigate(url: str) -> dict
  ```

### C6 — WebArena Adapter (`eval/webarena/adapter.py`) — NEW
- **Responsibility:** translate between WebArena's action/observation space and `observe`/`act`.
- **Reuses:** C3 runtime; `observe(operator)` for the a11y-tree-equivalent view; `act` verbs.
- **Mapping:**
  | WebArena action | runtime call |
  |---|---|
  | `click [id]` | `act("click", "@e{id}", "")` |
  | `type [id] [text]` | `act("fill", "@e{id}", text)` then optional `act("submit", ...)` |
  | `goto [url]` | `act("navigate", url, "")` |
  | `go_back` / `go_forward` | `navigate` to history (or JS `history.back()` via a bounded helper) |
  | `scroll [up/down]` | new bounded `act("scroll", dir, "")` verb (small addition) |
  | `stop [answer]` | `finish(answer)` |
  - WebArena element ids come from its a11y tree; we substitute SiliconSurfer's `@e` ids from
    `observe(operator)`, and feed the operator view as the observation. (This is the crux — see
    §8 algorithm and the open question on id alignment.)

### C7 — WebArena Eval Harness (`eval/webarena/harness.py`) — NEW
- **Responsibility:** load tasks, run runtime per task, score, report.
- **Reuses:** C3, C6; WebArena's own per-task `eval_types`/checkers (string_match,
  url_match, program_html) invoked unmodified; extends the existing `eval/` pipeline.
- **Interface:** `run_suite(task_ids: list[int], model_sel: dict) -> SuiteReport` (success_rate,
  per-task {passed, stop_reason, steps, cost}).

---

## 7. Interfaces with other modules (both directions)

**Runtime → Rust core (via PyO3), after F9:**
```
agent_browser.AgentSession()                 # new pyclass, C5
  .navigate(url) -> {success,url,detail}
  .observe(mode) -> {content,title,url,content_length,mode,element_count}   # session.rs:209
  .act(action,target,value) -> {success,url,detail}                        # session.rs:391
```
**Runtime → Registry:** `Registry.resolve(...) -> ModelSpec` (C1).
**Runtime → LLM providers:** OpenAI Chat Completions / Anthropic Messages (C2), endpoint from
`ModelSpec.endpoint`.
**Harness → Runtime:** `Runtime.run(goal, start_url, model_sel...) -> RunResult` (C3).
**Harness → WebArena checkers:** WebArena's `evaluator_router(config).__call__(trajectory,...)`
(unmodified upstream), fed our `RunResult.answer` + final URL + page HTML (`content()`).
**Existing surfaces left intact:** `mcp_server.py` observe/act (mcp_server.py:175) and
`agent_api.TOOL_DEFINITIONS` (agent_api.py:124) are unchanged; the runtime is additive and
reuses their schema shapes (C4), not their code paths.

---

## 8. Main algorithms

### 8.1 Agent loop (C3, FR-2/3)
```
run(goal, start_url, model_sel, max_steps, token_budget):
    spec  = registry.resolve(**model_sel)                 # FR-4; raises if ambiguous
    llm   = LLMClient(spec, api_key_for(spec.provider))
    sess  = AgentSession(); sess.navigate(start_url)       # F9 pyclass
    msgs  = [system_prompt(goal, tool_docs)]
    trajectory, tokens, cost = [], 0, 0.0
    for step in range(max_steps):
        obs = sess.observe("operator")                     # stamps live DOM + @e refs
        msgs.append(observation_msg(obs))                  # content is operator markdown
        turn = llm.step(msgs, tools=[OBSERVE, ACT, FINISH])# C2, C4
        tokens += turn.usage.total
        cost   += price(spec, turn.usage)                  # fresh vs cached split, FR-9
        if token_budget and tokens > token_budget:
            return RunResult(None, trajectory, BUDGET_EXHAUSTED, tokens, cost)   # NFR-4
        if turn.tool_call is None or turn.tool_call.name == "finish":
            return RunResult(turn.tool_call.answer if finish else turn.text,
                             trajectory, FINISHED, tokens, cost)
        tc = turn.tool_call
        if tc.name == "observe":                           # model asked to re-view a mode/url
            continue_with(tc.mode, tc.url)                 # optional explicit re-observe
        else:  # act
            res = sess.act(tc.action, tc.target, tc.value) # session.rs:391; clears locator_map
            msgs.append(action_result_msg(res))
        trajectory.append(Step(obs, tc, turn.usage))
    return RunResult(None, trajectory, MAX_STEPS, tokens, cost)
```
**Invariants / edge cases.**
- **I1 — observe-before-@e-act:** because `act` clears `locator_map` (session.rs:477), the loop
  ALWAYS `observe`s at the top of each iteration before issuing an `@e` action (NFR-3).
- **I2 — SPA escalation:** `observe` runs on the persistent T1 session (already rendered);
  first `navigate` inherits router behavior. A `Loading…` shell escalates via
  `is_sparse_content` (router.rs) so the loop never plans on an empty page (NFR-2).
- **I3 — closed action set:** only the C4 tool enum reaches `act`; `eval` (cdp.rs:230) is never
  model-reachable (NFR-5).
- **E1 — malformed tool call:** if the model returns no valid tool call twice in a row, emit a
  reminder message once; on repeat, terminate `SESSION_ERROR` (no infinite loop).

### 8.2 WebArena id alignment (C6, FR-7)
WebArena tasks reference elements by *its* a11y-tree ids; SiliconSurfer emits `@e` ids in
operator-source order (operator.rs:23 shared `AtomicUsize`). We do **not** try to match
upstream ids — instead we feed the model **our** operator observation, so the model plans in
`@e` space, and the adapter only needs to translate the final action verbs. Upstream's checker
scores on outcome (URL / HTML / answer), not on which id was clicked — so id divergence is safe.
Confirm this against `program_html` checkers during M4 (open question below).

---

## 9. Integration / E2E tests (every CUJ → ≥1 test)

| id | CUJ | setup → action → assertion |
|---|---|---|
| E2E-1 | CUJ-1 | Local static form page → `run("submit the form with name=Leo", url)` → asserts `stop_reason=FINISHED`, form POST observed, ≤ N steps |
| E2E-2 | CUJ-2 | `run(..., tier="small")` twice with 2 providers → asserts resolved ids differ, `total_cost_usd` matches `registry price × usage` within ε |
| E2E-3 | CUJ-4 | Static login page (csrf+user+pass+submit; mirrors integration_tests.rs:179) → agent fills `@e1/@e2`, clicks `@e4` → asserts logged-in marker in next observe |
| E2E-4 | CUJ-5 | `docs.claude.com` SPA (the `Loading…` shell) → first observe → asserts `mode_used`/content is rendered (T1), not the 142-byte shell |
| E2E-5 | CUJ-3 | WebArena `shopping` subset (5 tasks) booted in Docker → `run_suite([...])` → asserts each task's upstream checker runs and a `success_rate` is produced (baseline recorded, not thresholded) |
| E2E-6 | CUJ-3 | WebArena full or 50-task sample → nightly → records `success_rate` trend (regression guard) |

---

## 10. Success criteria
- **Functional:** FR-1..FR-9 all met; E2E-1..E2E-5 green.
- **CUJ-3 headline:** the runtime produces a **real WebArena `success_rate`** on ≥1 site
  (was: never measured), run with **DeepSeek (`deepseek-v4-flash`)** as the brain — the only
  model cheap enough (cache-hit $0.0028 / miss $0.14 / out $0.28 per 1M) to run the full 812
  tasks × multi-step × large operator observations affordably. First bar = beat a random/no-op
  baseline; reference = published GPT-4 WebArena ~14% on the same subset.
- **The metric is the point (harness-value axis):** report **`success_rate` AND
  cost-per-solved-task**. The thesis SiliconSurfer is trying to prove is *"a cheap/small brain +
  SiliconSurfer's body clears tasks that others need a frontier model for."* So the deliverable
  is not one number — it's the **model-scaling curve with the harness held fixed** (§10.1).
- **NFR thresholds:** NFR-1 ≤200 ms/step overhead; NFR-4 budget cap honored (E2E via a tiny
  budget forcing `BUDGET_EXHAUSTED`); NFR-3 no stale-`@e` mis-click (I1 enforced).

### 10.1 Model-scaling ablation (fix the harness, shrink the brain)
The eval axis is **not** "which frontier model wins" — it's "how small a brain can SiliconSurfer
carry." Hold the harness (SiliconSurfer operator observation + `@e` action space) constant and
walk the brain down:
1. **DeepSeek `deepseek-v4-flash`** (now) — cheap hosted, thinking-default, 1M ctx. Establishes
   the reference `success_rate` and cost-per-solved-task.
2. **Small models** (next) — small/nano tier from the registry (e.g. `gpt-5.4-nano`,
   `gemini-3.1-flash-lite`, or an open small model via an OpenAI-compat endpoint). Same tasks,
   same harness; watch how far the score holds as the brain shrinks and $/task drops.
3. **Self-trained Action-SLM** (later, TODO.md V3.0) — fine-tune a 7B/8B on the `@e` trajectories
   this harness produces; local inference, ~zero API cost.

**What "success" means for the harness:** if the curve stays high as the brain shrinks, the score
is attributable to SiliconSurfer, not to an expensive LLM. That is the claim the benchmark exists
to support. (Complementary secondary cut — fix the LLM, swap the *observation layer* SiliconSurfer
`operator` vs a baseline a11y/raw view — is optional; the primary axis above is the shrink-the-brain
curve the project committed to.)

---

## 11. Performance considerations
- Hot path per step = `observe` (distill 6.76 ms/500 KB baseline) + one LLM round-trip (secs,
  dominant) + `act` (JS eval + fixed sleeps: navigate 500 ms, act 800 ms — session.rs). Budget
  overhead excludes LLM. The fixed sleeps dominate NFR-1; measure and, if needed, replace with
  event-based waits (out of scope, note it).
- Token cost is the real budget: operator markdown can be large. Consider a max-content clamp
  before sending to the LLM (TODO.md V1.1 "Token truncation" — reuse when it lands).
- Harness throughput (NFR-6): concurrency bounded by the Chrome pool; one session/task.

---

## 12. Reliability considerations
- **Fail-closed loop:** every exit is a typed `StopReason` (FR-3); no infinite loop (E1), no
  silent stall on SPA shells (NFR-2, already fixed in router.rs).
- **`@e` invalidation** is enforced structurally (I1) — the design leans on the existing
  `locator_map.clear()` contract (session.rs:477) rather than re-implementing it.
- **Per-task isolation** (NFR-6): fresh `AgentSession` per WebArena task; matches TODO.md V1.1
  "Context isolation / Session pool" direction — align, don't fork.
- **`/health` caveat:** the server's `browser_ready` can report `true` when Chrome failed to
  launch (observed this session; stale `SingletonLock`). The harness must **not** trust
  `/health`; it should probe with a real `observe` and clear stale `chromiumoxide-runner` locks
  on startup (mcp_server.py:44 already does the lock cleanup — reuse that logic). *(Tracked as a
  separate bug; the runtime works around it, and the fix belongs in the server.)*

---

## 13. Security considerations
- **Trust boundary:** the LLM is untrusted input. Its only effectors are the C4 tool enum
  (click/fill/submit/navigate/set_cookies/scroll/finish). No raw JS (`eval`, cdp.rs:230) and no
  arbitrary shell are exposed (NFR-5).
- **Cookie/secret handling:** `set_cookies` can inject auth; the runtime must not log cookie
  values or API keys into the trajectory. Keys come from `config.toml` (gitignored), never from
  model output.
- **SSRF / navigation scope:** `navigate`/`goto` targets from the model should be constrained to
  an allowlist in eval mode (WebArena hosts only) to keep the agent inside the benchmark.
- **Sandbox:** WebArena sites run in Docker; the agent drives a headless Chrome — no host FS
  access from page context.

---

## 14. Abstraction & reuse

**Reuse map (from Phase 0 grounding):**
| symbol | file:line | how the runtime uses it |
|---|---|---|
| `AgentSession::observe` | session.rs:209 | per-step view; stamps live DOM `data-agent-id` (session.rs:253) + builds `locator_map` (session.rs:344-374) |
| `AgentSession::act` | session.rs:391 | executes model action; resolves `@e` (session.rs:401-413); invalidates map (session.rs:477) |
| `AgentSession::navigate` | session.rs:53 | initial + `goto`; injects stealth/tab-fix/dialog patches (session.rs:98-165) |
| `ObserveResult`/`ActResult` | session.rs:24-39 | returned to Python as dicts (C5) |
| PyO3 `Session` pattern | lib.rs:240-386 | template for the new `AgentSession` pyclass (`Arc<TokioMutex>`, `py.detach`, `run_async`, `action_result_to_dict` lib.rs:399) |
| `DistillMode`/`FastDistiller::distill` | distiller_fast.rs:13-57 | `operator` mode drives the observation; `@e` format from operator.rs:84-149 |
| Router auto-fallback | router.rs `fetch_auto` + `is_sparse_content` | guarantees rendered observations (NFR-2) |
| MCP `observe`/`act` schemas | mcp_server.py:126-171 | copied as the model-facing tool contract (C4) — identical to the proven MCP surface |
| `handle_tool_call` dispatch | agent_api.py:185 | pattern for C3's tool dispatch (name→callable, splat args) |
| `client.py` server spawn | client.py:55-95 | pattern for booting the Rust server if the runtime uses HTTP instead of PyO3 |
| `models.toml` | repo root (new) | registry source (C1) |
| WebArena checkers | upstream `evaluator_router` | unmodified scoring (C7) |

**New abstractions & justification:**
- `Registry`/`ModelSpec` (C1) — a typed model KB; justified because model choice + pricing must
  be data-driven (the whole point of `models.toml`), not hardcoded like the current `builtin.rs`.
- `LLMClient`/`LLMTurn` (C2) — one normalized turn over two endpoint families; justified to keep
  C3 provider-agnostic.
- `Runtime`/`RunResult`/`StopReason` (C3) — the loop is genuinely new; nothing in-repo loops.
- `AgentSession` pyclass (C5) — thin binding, not new logic; unblocks Python reuse of existing
  Rust (F9).
- WebArena `adapter`/`harness` (C6/C7) — glue to an external benchmark; kept in `eval/` beside
  the existing eval pipeline.

---

## Roadmap

Sequenced by **dependency** and **risk-up-front**. Each milestone is independently shippable and
verified by a named E2E test.

| id | scope | delivers | depends on | verified by |
|---|---|---|---|---|
| **M1** | **Expose `AgentSession` to Python (PyO3 pyclass)** — the load-bearing, highest-risk dependency (live-DOM `@e` stamping). Includes a maturin rebuild + a Python smoke test that `observe(operator)`→`act(click,@e)` resolves on a live page. | F9, C5; FR-6 | — | E2E-3 (login form, `@e` resolves) |
| **M2** | **Registry + LLM client** — `models.toml` loader/resolver (C1) + provider router (C2) with OpenAI-compat & Anthropic-compat, tool-calling, usage/cost. | F2, F3; FR-4/5 | — (parallel to M1) | E2E-2 (2 providers, cost math) |
| **M3** | **Agent loop MVP** — C3 + C4 wired over M1 session + M2 client; stop conditions, trajectory, budget. | F1, F4, F5, F6; FR-1/2/3/9 | M1, M2 | E2E-1, E2E-4 (SPA), E2E-3 |
| **M4** | **WebArena adapter + single-task run** — C6 + boot one Docker site; run one task end-to-end through the runtime; invoke upstream checker. Resolve the id-alignment open question here. | F7; FR-7 | M3 | E2E-5 setup (1 task green path) |
| **M5** | **WebArena harness + subset score** — C7 loader/runner/scorer/report; produce a real `success_rate` + cost-per-solved-task on a 5–50 task subset with **DeepSeek `deepseek-v4-flash`** as the brain (§10.1). | F8; FR-8 | M4 | E2E-5, E2E-6 |
| **M5b** | **Model-scaling ablation** — rerun the M5 subset with progressively smaller brains (small/nano tier), harness fixed; plot `success_rate` vs `$/task` (§10.1 axis). This is the harness-value evidence. | F8 | M5 | E2E-6 (curve, not single number) |
| **M6** | **Hardening** — token clamp (TODO V1.1), event-based waits (NFR-1), server `/health` truthfulness fix, allowlist for eval navigation, nightly WebArena regression. | NFR-1/4/5/6 | M5 | E2E-6 nightly trend |

**Open questions to close during M1/M4 (do not hand-wave):**
1. **M1:** does `AgentSession::observe`'s shadow-piercer stamp survive a subsequent `act`'s
   navigation for the *next* observe? (It rebuilds each observe — verify the timing with the
   800 ms post-act sleep.)
2. **M4:** do any WebArena `program_html` checkers assert on specific element ids/paths that our
   `@e` substitution would break? Sample the `shopping` + `gitlab` configs before committing to
   the "outcome-only scoring is safe" assumption (§8.2).
