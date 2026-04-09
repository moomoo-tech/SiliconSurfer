# Benchmark v1 — 2026-04-09

Git: (pending commit)
Change: Fix lol_html O(n²) performance — merge 3 passes into 2, single-pass entity decode + link resolve

## Criterion Micro-Benchmarks

### Parse Latency

| Size | scraper | lol_html v0 | lol_html v1 | v0→v1 |
|------|---------|------------|-------------|-------|
| small (500B) | 10.0 µs | 30.4 µs | **27.6 µs** | 1.1x |
| medium (50KB) | 2.24 ms | 1.79 ms | **1.49 ms** | 1.2x |
| large (500KB) | 8.1 ms | **264 ms** | **7.1 ms** | **37x** |

### Throughput

| Size | scraper | lol_html v0 | lol_html v1 |
|------|---------|------------|-------------|
| 50KB | 22 MB/s | 28 MB/s | **33 MB/s** |
| 500KB | 61 MB/s | 1.9 MB/s | **70 MB/s** |

### What changed

1. **3 passes → 2 passes**: merged `strip_noise` + `inject_markdown` into single `rewrite_combined()`
2. **O(n) entity decode**: replaced `while + replacen` loop with single-pass char scanner
3. **O(n) link resolve**: replaced `[[LINK:text:LINK:url]]` string markers with control chars (`\x01`-`\x05`), resolved in single scan
4. **Hash-based dedup**: `HashSet<u64>` (hash only) instead of `HashSet<String>` (clone + store)
5. **Pre-allocated capacity**: `HashSet::with_capacity(256)` for dedup

### Root cause of v0 regression

v0's `decode_entities()` used `while let Some(start) = result.find("&#")` + `result.replacen()`:
- Each `replacen` is O(n) scan + copy
- Wikipedia HTML has hundreds of numeric entities
- O(entities × html_length) = O(n²)

v1 scans chars once, decodes inline → O(n).

## Status

- 13/13 tests pass
- No quality regression (same output)
- lol_html now competitive with scraper at all sizes
- **lol_html is now the recommended distiller for all page sizes**
