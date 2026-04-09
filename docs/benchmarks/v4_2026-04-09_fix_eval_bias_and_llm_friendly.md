# Benchmark v4 — 2026-04-09 — Fix Eval Bias + LlmFriendly Mode

Git: `cce6059`

## Changes

1. New `DistillMode::LlmFriendly` as default (= lol_html Reader + conservative links)
2. Scraper rewritten with Visitor+Context pattern
3. Both engines now pass base_url for link resolution  
4. Reader mode: skip bare relative links (conservative, consistent)
5. **Eval fix: `strip_markdown()` before faithfulness matching**

## The Bug We Fixed

Eval was unfairly penalizing Markdown outputs. All 4 tools had identical LLM responses for GitHub test, but:
- Playwright (plain text): 100% faithful
- Our Markdown: 67% faithful

Root cause: `"Rust".lower() in "[Rust](https://...)".lower()` works, but `"nickel.rs is a simple..."` might not match across Markdown line breaks and formatting.

Fix: `strip_markdown()` removes `[text](url)`, `**bold**`, `_italic_`, `` `code` ``, `# heading` before matching.

## LLM Judge Results (After Fix)

| Tool | TaskPass | Faithful | Hallu | Action | Tokens |
|------|----------|----------|-------|--------|--------|
| our_scraper | 5/6 83% | **100%** | 0 | 17% | 8,578 |
| our_lol_html | 5/6 83% | **100%** | 0 | 17% | 9,778 |
| trafilatura | 6/6 100% | **100%** | 0 | 17% | 6,848 |
| playwright | 6/6 100% | **100%** | 0 | 0% | 8,442 |

All tools now 100% faithful (was 86-100% before fix).

## Remaining Gaps

| Gap | Cause | Fix |
|-----|-------|-----|
| Task Pass 83% vs 100% | Example.com summarize + Python.org JSON parse | Improve lol_html output for tiny pages |
| Tokens 9.8K vs 6.8K | lol_html preserves more structure | Add Profile-based noise filtering |
| Action 17% vs 0% | We preserve some links, PW preserves none | Already winning here |

## Content Sizes (After Scraper base_url Fix)

| Site | scraper | lol_html | trafilatura |
|------|---------|----------|-------------|
| HN | 5,686 | 5,678 | 3,838 |
| Wikipedia | 134,465 | 134,459 | 0 |
| Python.org | 4,812 | 5,027 | 2,894 |
| Blog | 20,979 | 25,513 | 20,809 |
| GitHub | 2,232 | 5,265 | 2,079 |
| MDN | 27,415 | 27,471 | 24,222 |

## Key Lesson

> "Don't blame the engine when the evaluator is biased."

Markdown outputs need format-aware evaluation. Simple `substring in source` penalizes structured outputs unfairly.
