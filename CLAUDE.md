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
      distiller.rs      — AST engine (scraper, Visitor+Context)
      distiller_fast.rs — Stream engine dispatcher (lol_html)
      profiles.rs — Site-specific noise profiles (loaded from profiles.toml)
      probe.rs    — DOM smoke test / assertion API
      router.rs   — T0/T1/auto routing engine
      extract.rs  — Extractor trait + Profile config
      fetcher.rs  — HTTP fetch + distill
  server/         — axum HTTP server (/fetch, /distill, /probe, /health)
  python/         — PyO3 bindings (fetch, probe)

python/           — Python API + Agent tools
examples/         — E2E Agent loop demos
eval/             — Evaluation pipeline (heuristic + LLM judge)
mcp_server.py     — MCP server (5 tools for Claude)
profiles.toml     — Site noise database (add sites without recompiling)
config.toml       — API keys (gitignored)
docs/benchmarks/  — Performance diary (v0-v11)
```

## Key Commands

```bash
# Build
cargo build --release -p agent-browser-server

# Test
cargo test -p agent-browser-core

# Benchmark
cargo bench -p agent-browser-core

# Run server
PORT=9883 ./target/release/agent-browser-server

# Run MCP server (for Claude Code)
uv run python mcp_server.py

# Run eval
python3 eval/llm_judge.py
python3 eval/llm_judge_comprehensive.py
python3 eval/e2e_browser_use.py
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

## Config

- `config.toml` — API keys (Gemini, OpenAI). Gitignored.
- `config.example.toml` — Template.
- `profiles.toml` — Site-specific noise selectors. Add new sites here.
- `.mcp.json` — MCP server registration for Claude Code.

## Benchmark Results

- Distiller: 6.76ms / 500KB (39x improvement from v0)
- LLM Judge: 30/30 (only tool with perfect score)
- E2E Agent: 5/5 in 34.4s vs browser-use 0/5
- vs Jina Reader: 30/30 vs 20/30
- vs Trafilatura: 30/30 vs 15/30
