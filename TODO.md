# SiliconSurfer Roadmap

## V1.1 — Hardening

- [ ] **Profile integration**: wire `requires_t1()` / `custom_wait_ms()` / `wait_for_selector()` into router.rs and session.rs
- [ ] **China sites**: test xueqiu/eastmoney/sina after profile integration
- [ ] **Context isolation**: incognito BrowserContext per session (chromiumoxide `CreateBrowserContextParams`)
- [ ] **Session pool**: replace global `_session` singleton with session map (multi-conversation safe)
- [ ] **Trajectory eval CI**: expand eval/ into CI pipeline — 1000+ offline snapshots, auto-run on commit
- [ ] **Token truncation**: optional max_len in distiller, circuit breaker for 100K+ pages

## V2.0 — Production Infrastructure

- [ ] **HITL (Human-in-the-Loop)**: `act("ask_human")` — freeze CDP, stream browser via WebRTC, human takes over for 2FA/CAPTCHA/QR, returns control
- [ ] **DOM vector memory**: store @e action traces in local vector DB (LanceDB/Chroma), skip LLM on repeat visits — "muscle memory"
- [ ] **Stealth arsenal**: custom Chromium build (C++ fingerprint), residential proxy rotation, Canvas/WebGL spoofing — beat Cloudflare Turnstile/DataDome
- [ ] **TLS fingerprint**: revisit reqwest-impersonate ecosystem (currently broken — all deps yanked), or build on rquest v5 when API stabilizes

## V3.0 — Endgame

- [ ] **Action SLM**: fine-tune 7B/8B model on SiliconSurfer @e traces — local inference ~50ms, zero API cost
- [ ] **Multi-agent swarm**: Planner (CEO) → Surfer (executor) → Critic (QA) → Memory (archivist) — multiple brains, one body
- [ ] **Self-writing profiles**: agent encounters unknown site → generates probe JS → injects via CDP → writes bypass script → uploads to cloud profile DB

## Done (V1.0)

- [x] Rust core: T0/T1/Auto routing, 5 distill modes, 78 tests
- [x] BrowserSession: navigate/click/fill/submit via CDP
- [x] Cookie injection: CDP Network.setCookie
- [x] PyO3 bindings: fetch, probe, Session (with GIL release)
- [x] MCP server: observe + act, persistent session, state consistency
- [x] Profile system: profiles/ directory, per-site .toml, extensible schema
- [x] Mutex fix: lock scoped to page creation only
- [x] GIL fix: py.detach() on all blocking PyO3 methods
- [x] JS injection: serde_json escaping, evaluate_tolerant for navigation errors
- [x] Stealth patches: webdriver, plugins, languages, chrome.runtime, tab fix, dialog handler, shadow DOM
- [x] GitHub Actions CI: clippy -D warnings, fmt, test, pytest
- [x] Docs: English README + Chinese README_CN, all docs/ translated
- [x] License: Apache-2.0
