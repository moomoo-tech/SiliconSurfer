# Benchmark v3 — 2026-04-09 — Multi-mode + Relative Links

Git: `fd9f080`

## Changes

- 5 distill modes: Reader/Operator/Spider/Developer/Data
- Bare relative link resolution (item?id=123 → full URL)
- resolve_href() shared across all modes

## Criterion (no regression)

| Size | scraper | lol_html |
|------|---------|----------|
| 500B | 10.5 µs | 28.1 µs |
| 50KB | 2.34 ms | **1.47 ms** |
| 500KB | 8.26 ms | **6.89 ms** |

## LLM Judge

| Tool | Pass | Rate | Tokens | vs v2 |
|------|------|------|--------|-------|
| our_scraper | 6/6 | **100%** | 7,652 | same |
| trafilatura | 6/6 | **100%** | 6,772 | same |
| our_lol_html | 5/6 | **83%** ↓ | 12,349 | was 100%, 8.6K |
| playwright | 5/6 | 83% | 8,410 | same |

## Regression: lol_html quality dropped

### What broke

lol_html Example.com summarize test failed. LLM couldn't find "example" and "domain" in response.

### Root cause

Bare relative link fix (`resolve_href`) now preserves ALL relative links including nav/footer/UI links. lol_html has no content area detection (no DOM tree), so everything gets through.

Content size explosion:
- HN: 5.7K → 14.5K (+154%)
- Wikipedia: 97K → 176K (+82%)
- GitHub: 3.4K → 5.5K (+61%)

### Why scraper is unaffected

Scraper distiller has DOM tree → finds `<article>`/`<main>` content area first → only processes links inside that area.

### Fix options

1. Reader mode: only resolve relative links that start with `/` (root-relative), skip bare relative (item?id=123)
2. Reader mode: add nav/header/footer link filtering even though elements aren't removed
3. Accept: different modes for different needs — Reader stays conservative, Spider/Operator get all links

## Test Results

25/25 Rust tests pass (17 distiller_fast + 8 distiller)
