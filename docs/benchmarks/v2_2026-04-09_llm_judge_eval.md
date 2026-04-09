# Benchmark v2 — 2026-04-09 — LLM-as-Judge Evaluation

Git: (pending)
Tool: Gemini (gemini-3.1-flash-lite-preview)

## Test Results

6 tests x 4 tools = 24 evaluations

| Test | our_scraper | our_lol_html | trafilatura | playwright |
|------|-------------|-------------|-------------|------------|
| HN: Extract top 3 stories | ✓ | ✓ | ✓ | ✓ |
| HN: Find comment URL | ✓* | ✓* | ✓* | ✓* |
| Example: Summarize | ✓ | ✓ | ✓ | ✓ |
| GitHub: Extract repo info | ✓ | ✓ | ✓ | ✓ |
| MDN: Extract function syntax | ✓ | ✓ | ✓ | ✓ |
| Python.org: Extract key info | ✓ | ✓ | ✓ | ✓ |

*⚠️ Comment URL test is flawed — LLM guessed the URL pattern, none of the tools actually preserved the comment link in their output.

### Pass Rate

| Tool | Passed | Total | Rate |
|------|--------|-------|------|
| our_scraper | 6 | 6 | 100% |
| our_lol_html | 6 | 6 | 100% |
| trafilatura | 6 | 6 | 100% |
| playwright | 6 | 6 | 100% |

### Token Cost

| Tool | Total Prompt Tokens | vs Best |
|------|-------------------|---------|
| trafilatura | **6,690** | baseline |
| our_scraper | 7,646 | +14% |
| playwright | 8,322 | +24% |
| our_lol_html | 8,660 | +29% |

## Key Findings

1. **All tools pass all tests** — current tests are too easy, don't differentiate quality
2. **Trafilatura most token-efficient** — best at noise removal, least tokens consumed
3. **Comment URL test is invalid** — LLM hallucinated/guessed the URL, no tool actually preserved it
4. **None of the tools preserve HN comment links** — `item?id=...` URLs are relative, all tools miss them

## Action Items

1. Add harder tests that only pass with actual content (not LLM guessing)
2. Add "URL must appear in source text" validation
3. Fix HN comment link preservation (relative URL `item?id=...` needs base URL)
4. Reduce lol_html token overhead (29% more than trafilatura — too much structural markup?)
