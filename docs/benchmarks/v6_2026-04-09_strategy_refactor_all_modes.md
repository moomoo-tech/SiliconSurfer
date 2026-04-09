# Benchmark v6 — 2026-04-09 — Strategy Refactor + All Modes

Git: `d1c4090`

## Changes

- Strategy Pattern refactor: each mode in its own file
- All 5 modes benchmarked with criterion
- Operator mode eval: 5/5 pass vs Playwright 3/5

## Criterion: All Modes (50KB HTML)

| Mode | Time | vs Reader | Engine |
|------|------|-----------|--------|
| Spider | **1.11ms** | 0.73x fastest | lol_html + scraper (links only) |
| Operator | **1.13ms** | 0.74x | lol_html (minimal noise removal) |
| Data | **1.25ms** | 0.82x | scraper (tables/lists only) |
| Reader | **1.52ms** | 1.0x baseline | lol_html (full noise removal) |
| Developer | **1.90ms** | 1.25x slowest | scraper (full DOM + attributes) |

## Criterion: Size Comparison (Reader mode)

| Size | scraper AST | lol_html stream |
|------|------------|-----------------|
| 500B | 11.5 µs | 28.5 µs |
| 50KB | 2.96ms | **1.52ms** |
| 500KB | 11.7ms | **6.95ms** |

lol_html faster at 50KB+ (the real-world sweet spot).

## Operator Mode Output Quality

Login form (quotes.toscrape.com/login):
```
[Form: POST /login]
  [Input: type=hidden name=csrf_token]
  Username [Input: type=text name=username]
  Password [Input: type=password name=password]
  [Input: type=submit]
[/Form]
```

Pizza order form (httpbin.org/forms/post):
```
[Form: POST /post]
  Customer name: [Input: type=text name=custname]
  Telephone: [Input: type=tel name=custtel]
  Pizza Size [Input: type=radio name=size] Small/Medium/Large
  Pizza Toppings [Input: type=checkbox name=topping] Bacon/Cheese/Onion/Mushroom
  [Button: Submit order]
[/Form]
```

## Operator vs Playwright (Interaction Tasks)

| Test | Operator | Playwright | Why |
|------|----------|------------|-----|
| Login form | **✓** | ✗ | PW has no field names |
| Pagination | **✓** | ✗ | PW has no URLs |
| Shopping | ✓ | ✓ | Both work |
| Form fields | **✓ (11)** | ✓ (8) | We find more fields |
| Navigation | ✓ | ✓ | Both work |
| **Total** | **5/5** | **3/5** | |

## Architecture After Refactor

```
strategy/
  mod.rs         — shared utils (5 functions)
  reader.rs      — 📖 Reader (55 LOC)
  operator.rs    — 🕹️ Operator (80 LOC)
  spider.rs      — 🕸️ Spider (40 LOC)
  developer.rs   — 🛠️ Developer (55 LOC)
  data.rs        — 📊 Data (55 LOC)
distiller_fast.rs — dispatcher (20 LOC)
```

Each mode is isolated, testable, independently optimizable.
32 tests pass across all modes.
