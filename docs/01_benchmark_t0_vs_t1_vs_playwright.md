# 01 Benchmark: T0 vs T1 vs Playwright (scraper distiller)

日期: 2026-04-09
Distiller: scraper 0.22 (全量 DOM)

## 背景

第一版 distiller 用 scraper 库构建完整 DOM 树再遍历提取。
这是 T0/T1/Playwright 三方对比的首次测试。

## 结果

### 速度

| URL | T0 reqwest | T1 our Chrome | Playwright |
|-----|-----------|---------------|------------|
| HN | **450ms** | 474ms | 509ms |
| Example | **137ms** | 253ms | 306ms |
| Wikipedia | 495ms | 432ms | **216ms** |
| Python.org | **212ms** | 430ms | 360ms |
| Blog | **348ms** | 533ms | 506ms |

### 内容量 (chars)

| URL | T0 | T1 | Playwright |
|-----|-----|-----|------------|
| HN | 14,218 | 14,218 | 4,159 |
| Example | 169 | 169 | 129 |
| Wikipedia | 99,248 | 99,652 | 76,159 |
| Python.org | 乱码 (gzip 未解压) | 2,916 | 2,676 |
| Blog | 20,838 | - | 21,774 |

### 资源

| 指标 | T0 | T1 | Playwright |
|------|-----|-----|------------|
| 启动成本 | 0ms | 共享守护进程 | ~1800ms |
| 内存/会话 | ~1 MB | ~30 MB | ~50 MB |
| 并发上限 (8GB) | ~5000+ | ~100 | ~100 |

## 发现的问题

1. **Python.org 乱码** — reqwest 未启用 gzip/brotli 解压
2. **HN 14K chars 虚高** — table 布局被渲染成 markdown table，导航重复 3 遍
3. **Wikipedia T0 慢于 Playwright** — scraper 全量 DOM 构建在大页面上性能差
4. **Playwright 内容少不是丢数据** — innerText 更紧凑，我们的 markdown table 标记膨胀
