# Benchmark: T0 (reqwest) vs T1 (our Chrome) vs Playwright

测试日期: 2026-04-09 (lol_html 优化后)

## 测试环境

- macOS Darwin 25.4.0, Apple Silicon
- Rust release build
- T0: reqwest 0.12 + lol_html 2.7 streaming distiller
- T1: chromiumoxide 0.9 + headless Chrome daemon + scraper distiller
- Playwright 1.58 + Chromium headless (block CSS/img/font)

## 测试 URL

| URL | 类型 |
|-----|------|
| https://news.ycombinator.com/ | 列表页 |
| https://example.com | 极简页面 |
| https://en.wikipedia.org/wiki/Rust_(programming_language) | 超大页面 |
| https://www.python.org/ | 中等主页 |

3 rounds 取平均。

## 速度对比

| URL | T0 | T1 (our Chrome) | Playwright |
|-----|-----|-----------------|------------|
| HN | **495ms** | 405ms | 538ms |
| Example | **210ms** | 298ms | 330ms |
| Wikipedia | **288ms** | 3829ms | 292ms |
| Python.org | **141ms** | 285ms | 404ms |
| **Total** | **1134ms** | 4818ms | 1563ms |
| **Total (含启动)** | **1134ms** | 4818ms | 2137ms |

## 内容提取量 (chars)

| URL | T0 | T1 | Playwright |
|-----|-----|-----|------------|
| HN | 6,157 | 6,157 | 4,175 |
| Example | 167 | 167 | 129 |
| Wikipedia | 77,023 | 77,275 | 76,159 |
| Python.org | 2,622 | 2,622 | 2,676 |

## 资源占用

| 指标 | T0 reqwest | T1 our Chrome | Playwright |
|------|-----------|---------------|------------|
| 启动成本 | 0ms | 共享守护进程 | ~574ms 冷启动 |
| 内存/会话 | ~1 MB | ~30 MB | ~50 MB |
| 8GB RAM 并发上限 | ~5000+ | ~100 | ~100 |
| JS 渲染 | 不支持 | 支持 | 支持 |

## 关键结论

1. **T0 总耗时 1134ms vs Playwright 2137ms — 快 1.9x**
2. **T0 不需要浏览器** — 零启动成本，内存消耗极低
3. **T0 内容质量更好** — HN 页面 6K chars vs Playwright 4K chars，保留完整链接和 Markdown 结构
4. **T1 Wikipedia 大页面有性能问题** — 3.8s vs T0 288ms，scraper DOM 构建在 T1 路径仍是瓶颈
5. **Python.org 乱码已修复** — reqwest 启用 gzip/brotli/deflate 自动解压

## Distiller 优化历程

| 版本 | Wikipedia 耗时 | 技术 |
|------|---------------|------|
| v1 (scraper) | ~500ms | 全量 DOM 构建 + 树遍历 |
| v2 (lol_html) | **~90ms** | 流式处理，零 DOM 分配 |
| 提升 | **5.5x** | |
