# SiliconSurfer 🏄

> The MCP-compatible browser built for silicon-based lifeforms.

## Project Structure

```
crates/
  core/           — Rust core library
    src/
      strategy/   — 5 distill modes (reader/operator/spider/developer/data)
      browser.rs  — Chrome daemon pool (chromiumoxide CDP)
      cdp.rs      — CDP interaction layer (click/fill/submit)
      session.rs  — AgentSession (stateful: navigate/observe/act + stealth patches)
      distiller.rs      — AST engine (scraper, Visitor+Context)
      distiller_fast.rs — Stream engine dispatcher (lol_html)
      profiles.rs — Site-specific noise profiles (loaded from profiles.toml)
      probe.rs    — DOM smoke test / assertion API
      router.rs   — T0/T1/auto routing engine
      extract.rs  — Extractor trait + Profile config
      fetcher.rs  — HTTP fetch + distill
    tests/        — 38 integration tests
    benches/      — Criterion benchmarks
  server/         — axum HTTP server (/fetch, /distill, /probe, /health)
  python/         — PyO3 bindings (fetch, probe, Session)

python/           — Python API + Agent tools
tests/            — Python tests (pytest)
eval/             — Evaluation pipeline (heuristic + LLM judge)
mcp_server.py     — MCP server (2 tools: observe + act)
profiles/         — Site profiles (one .toml per site: noise, force_t1, wait_ms)
config.toml       — API keys (gitignored)
.github/workflows/ci.yml — GitHub Actions CI
```

## Key Commands

```bash
# Build
cargo build --release -p agent-browser-server

# Test (Rust)
cargo test --workspace

# Test (Python)
uv run python -m pytest tests/ -v

# Lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# Benchmark
cargo bench -p agent-browser-core

# Run server
PORT=9883 ./target/release/agent-browser-server

# Run MCP server (for Claude Code)
uv run python mcp_server.py
```

## Architecture

- **T0 (reqwest)**: HTTP fetch, no browser, 1ms distill
- **T1 (Chrome CDP)**: JS rendering, shared daemon, session state
- **Auto**: T0 first, fallback to T1 if content sparse

5 distill modes via Strategy Pattern:
- **Reader**: LLM-friendly markdown, aggressive noise removal
- **Operator**: @e element refs, form/button/input annotations
- **Spider**: JSON link topology (nav/content/footer)
- **Developer**: DOM skeleton with attributes
- **Data**: Structured JSON tables/lists

MCP tools: `observe(url, mode)` + `act(action, target, value)`

## Config

- `config.toml` — API keys (Gemini, OpenAI). Gitignored.
- `config.example.toml` — Template.
- `profiles/` — Site profiles (one .toml per site). Supports: extra_noise, force_t1, wait_ms, wait_for_selector.
- `.mcp.json` — MCP server registration for Claude Code.

## Benchmark Results

- Distiller: 6.76ms / 500KB (39x improvement from v0)
- LLM Judge: 30/30 (only tool with perfect score)
- E2E Agent: 5/5 in 34.4s vs browser-use 0/5
- vs Jina Reader: 30/30 vs 20/30
- vs Trafilatura: 30/30 vs 15/30
