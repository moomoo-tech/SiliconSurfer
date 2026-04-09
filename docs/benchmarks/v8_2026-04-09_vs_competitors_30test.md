# Benchmark v8 — 2026-04-09 — vs Competitors (30 tests)

Git: `26cde5b`

## Head-to-Head: Agent Browser vs Jina Reader vs Trafilatura

30 tests, 3 categories, 4 tools. Same LLM judge (Gemini).

### Grand Summary

| Tool | Navigation | Forms | Content | **TOTAL** |
|------|-----------|-------|---------|-----------|
| **Our Operator** | **10/10** | **10/10** | **10/10** | **30/30 (100%)** |
| Jina Reader | 10/10 | 0/10 | 10/10 | 20/30 (67%) |
| Our Reader | 6/10 | 0/10 | 10/10 | 16/30 (53%) |
| Trafilatura | 3/10 | 3/10 | 9/10 | 15/30 (50%) |

**Agent Browser Operator is the ONLY tool that passes all 30 tests.**

### Navigation (10 tests)

| Test | Operator | Jina | Reader | Trafilatura |
|------|----------|------|--------|-------------|
| Login URL | ✓ | ✓ | ✓ | ✗ |
| Next page URL | ✓ | ✓ | ✗ | ✗ |
| Category link | ✓ | ✓ | ✗ | ✗ |
| Count links | ✓ | ✓ | ✓ | ✓ |
| Specific link | ✓ | ✓ | ✓ | ✓ |
| Author page | ✓ | ✓ | ✓ | ✗ |
| Tag page | ✓ | ✓ | ✓ | ✗ |
| Footer link | ✓ | ✓ | ✗ | ✗ |
| External link | ✓ | ✓ | ✗ | ✗ |
| Link text | ✓ | ✓ | ✓ | ✓ |

Jina matches us here — both preserve markdown links. Trafilatura only 3/10 (no links).

### Forms / Login (10 tests)

| Test | Operator | Jina | Reader | Trafilatura |
|------|----------|------|--------|-------------|
| Login field names | **✓** | ✗ | ✗ | ✗ |
| Form action URL | **✓** | ✗ | ✗ | ✗ |
| Hidden CSRF token | **✓** | ✗ | ✗ | ✗ |
| All field names | **✓** | ✗ | ✗ | ✗ |
| Input types | **✓** | ✗ | ✗ | ✗ |
| Submit button text | **✓** | ✗ | ✗ | ✓ |
| Radio options | **✓** | ✗ | ✗ | ✓ |
| Checkbox options | **✓** | ✗ | ✗ | ✓ |
| POST target URL | **✓** | ✗ | ✗ | ✗ |
| Email field name | **✓** | ✗ | ✗ | ✗ |

**Only Operator can find form field names, action URLs, CSRF tokens, and input types.**
Jina/Reader/Trafilatura all strip form elements — Agent cannot interact.
Trafilatura picks up 3 (button text + visible radio/checkbox labels, but not field names).

### Content Extraction (10 tests)

| Test | Operator | Jina | Reader | Trafilatura |
|------|----------|------|--------|-------------|
| First quote | ✓ | ✓ | ✓ | ✓ |
| Quote author | ✓ | ✓ | ✓ | ✓ |
| Count quotes | ✓ | ✓ | ✓ | ✓ |
| Book title | ✓ | ✓ | ✓ | ✓ |
| Book price | ✓ | ✓ | ✓ | ✓ |
| Count books | ✓ | ✓ | ✓ | ✓ |
| Quote tags | ✓ | ✓ | ✓ | ✗ |
| Page heading | ✓ | ✓ | ✓ | ✓ |
| Book availability | ✓ | ✓ | ✓ | ✓ |
| Total quote count | ✓ | ✓ | ✓ | ✓ |

Content extraction is a tie — all tools handle reading well.

### Why Operator Wins

| Capability | Operator | Jina | Trafilatura |
|-----------|----------|------|-------------|
| Markdown links | ✓ | ✓ | ✗ |
| Form field names | **✓** `[Input: name=username]` | ✗ | ✗ |
| Form action URL | **✓** `[Form: POST /login]` | ✗ | ✗ |
| CSRF tokens | **✓** `[Input: type=hidden name=csrf_token]` | ✗ | ✗ |
| Button labels | **✓** `[Button: Submit]` | ✗ | partial |
| Input types | **✓** `type=text/radio/checkbox/email` | ✗ | ✗ |
| Nav annotations | **✓** `[Nav]` sections | ✗ | ✗ |

### Token Cost

| Tool | Total Tokens (30 tests) |
|------|------------------------|
| Trafilatura | **7,360** (cheapest) |
| Our Reader | 23,597 |
| Our Operator | 32,445 |
| Jina Reader | 32,541 |

Trafilatura is cheapest but passes only 50%. Operator costs 4.4x more tokens but passes 100%.

### The Multi-Mode Advantage

No single mode wins everything. Agent Browser's strength is choosing the right mode:

| Task | Best Mode | Why |
|------|-----------|-----|
| "Summarize this article" | Reader | Cleanest, fewest tokens |
| "Log in to this site" | **Operator** | Has field names + form action |
| "Find the pricing page" | Spider | JSON link topology |
| "Extract the comparison table" | Data | Structured JSON |
| "Write a test script for this page" | Developer | DOM skeleton + selectors |

**No competitor offers multiple modes. They're stuck with one view.**
