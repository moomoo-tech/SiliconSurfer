# Benchmark v9 — 2026-04-09 — SiliconSurfer vs browser-use E2E

Git: `6ee8433`

## Setup

- SiliconSurfer: Rust distiller + Gemini + Playwright (hands)
- browser-use: Python + OpenAI GPT-5.4-nano + built-in Playwright
- Same 5 goals on test-friendly sites

## Results

| Goal | SiliconSurfer | browser-use |
|------|--------------|-------------|
| Find login form fields | **✓ 3.1s, 1 step** | ✗ 49.9s |
| Find cheapest book | **✓ 18.9s, 10 steps** | ✗ 120.4s (timeout) |
| Collect quotes | **✓ 2.9s, 1 step** | ✗ 8.5s |
| Navigate to Travel books | **✓ 17.3s, 10 steps** | ✗ 40.4s |
| Login to secure area | **✓ 20.3s, 10 steps** | ✗ 16.0s |
| **TOTAL** | **5/5 — 62.5s** | **0/5 — 235.2s** |

## Key Architectural Wins

1. **see_page()**: Distill Playwright's current page (with session cookies) instead of independent HTTP fetch. Fixed the secure login loop.

2. **Multi-mode vision**: Spider for link discovery, Operator for form fields, Reader for content. browser-use has one fixed DOM extraction mode.

3. **@e element references**: Every interactive element gets `@e1`, `@e2` etc. Agent says "fill @e3" instead of guessing CSS selectors.

4. **/distill endpoint**: Agent can send any HTML to our Rust distiller. Decouples "fetching" from "understanding".

## Speed Comparison

| Metric | SiliconSurfer | browser-use |
|--------|--------------|-------------|
| Total time | **62.5s** | 235.2s |
| Fastest task | **2.9s** (quotes) | 8.5s |
| Slowest task | **20.3s** (login) | 120.4s (timeout) |
| Speedup | **3.7x faster** | baseline |
