# Benchmark v12 — 2026-04-09 — Slickdeals Real World Shopping

Git: `5befcdd`

## Real shopping site: slickdeals.net/deals/

No anti-bot. T0 reqwest direct fetch, no browser needed.

| Metric | SiliconSurfer | Jina Reader | Trafilatura |
|--------|--------------|-------------|-------------|
| **Speed** | **604ms** | 16,635ms (27x slower) | 692ms |
| **Content** | 13,639 chars | 30,252 chars | 5,468 chars |
| **Links** | **66** | 205 | **0** |
| **Prices** | **71** | 76 | 52 |
| **Est tokens** | **~6,800** | ~15,100 | ~2,700 |

## Verdict

- **Trafilatura**: cheapest tokens but USELESS for Agent — zero links, incomplete prices, only got category navigation not actual deals
- **Jina**: most links but 27x slower and 2.2x more tokens
- **SiliconSurfer**: best balance — fast, complete deals with prices, clickable links, reasonable tokens
