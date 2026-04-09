# Benchmark v5 — 2026-04-09 — Operator Mode Beats Playwright

Git: `b49ac80`

## What Changed

- Server accepts `distill` parameter for all 6 modes
- distill_mode threaded through engine → router → fetcher → FastDistiller
- Full eval pipeline: collect all modes + Playwright + LLM judge

## Operator Mode Eval (Interaction Tasks)

5 tests on scrape-friendly sites:

| Test | Reader | Operator | Scraper | Playwright |
|------|--------|----------|---------|------------|
| Login: form fields | ✗ | **✓** | ✗ | ✗ |
| Pagination link | ✗ | **✓** | ✗ | ✗ |
| Book shopping | ✓ | **✓** | ✓ | ✓ |
| Form inputs | ✗ EMPTY | **✓** (11 fields) | ✗ EMPTY | ✓ (8 fields) |
| Navigation links | ✓ | **✓** | ✓ | ✓ |

### Pass Rate

| Tool | Pass | Rate |
|------|------|------|
| **Operator (ours)** | **5/5** | **100%** |
| Playwright | 3/5 | 60% |
| Reader (ours) | 2/5 | 40% |
| Scraper (ours) | 2/5 | 40% |

## Why Operator Beats Playwright

| Capability | Operator | Playwright innerText |
|-----------|----------|---------------------|
| Form field names | ✓ `[Input: type=text name=username]` | ✗ (no field names) |
| Form action URL | ✓ `[Form: POST /login]` | ✗ (no URLs) |
| Pagination URL | ✓ `[/page/2/](url)` | ✗ (text "Next" but no URL) |
| Button labels | ✓ `[Button: Submit]` | ✗ (just text "Submit") |
| Link URLs | ✓ `[text](absolute_url)` | ✗ (text only) |

## Content Size Comparison

| Site | Reader | Operator | Spider | Playwright |
|------|--------|----------|--------|------------|
| toscrape_login | 94 | **390** | 296 | 91 |
| httpbin_forms | **0** | **600** | 63 | 207 |
| books_toscrape | 2,302 | **8,187** | 5,574 | 2,029 |
| herokuapp | 3,165 | 3,165 | 3,705 | 774 |

## Multi-Mode Vision Validated

```
LLM-Friendly (Reader): for content understanding    — 100% faithful
Operator:               for interaction/automation  — 100% task pass
Spider:                 for navigation/exploration  — link topology JSON
```

Different tasks need different "eyes". One mode can't serve all Agent needs.
