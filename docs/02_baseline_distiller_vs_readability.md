# Baseline: Our Distiller vs Readability.js vs Raw innerText

测试日期: 2026-04-09

## 测试环境

- macOS Darwin 25.4.0, Apple Silicon
- Our Distiller: Rust (reqwest 0.12 + scraper 0.22), T0 模式
- Readability.js: Mozilla Readability 0.5.0, 通过 Playwright 注入
- Raw innerText: Playwright headless Chrome, `document.body.innerText`
- 所有 Playwright 测试均禁用图片/字体/媒体加载

## 测试页面

| 页面 | 类型 | URL |
|------|------|-----|
| HN | 列表页 | https://news.ycombinator.com/ |
| Example | 极简页面 | https://example.com |
| Wikipedia | 超大页面 | https://en.wikipedia.org/wiki/Rust_(programming_language) |
| Python.org | 中等主页 | https://www.python.org/ |
| Blog | 博客文章 | https://aphyr.com/posts/411-the-future-of-everything-is-lies-i-guess |

## 速度对比 (ms)

| Page | Ours (T0) | Readability.js | innerText |
|---|---|---|---|
| HN (list) | **450ms** | 607ms | 474ms |
| Example (tiny) | **137ms** | 253ms | 306ms |
| Wikipedia (huge) | 495ms | 432ms | **216ms** |
| Python.org (mid) | **212ms** | 430ms | 360ms |
| Blog post | **348ms** | 533ms | 506ms |

我们 4/5 最快。唯一输的 Wikipedia 大页面（scraper 解析开销大，后续可换 lol_html 流式解析器优化）。

## 内容大小 (chars)

| Page | Ours | Readability.js | innerText |
|---|---|---|---|
| HN (list) | 6,184 | 3,839 | 4,182 |
| Example (tiny) | 167 | 111 | 129 |
| Wikipedia (huge) | 77,023 | 71,095 | 78,994 |
| Python.org (mid) | 2,622 | 2,657 | 6,982 |
| Blog post | 20,838 | 15,905 | 21,774 |

## 内容质量逐页分析

### HN (列表页) — 我们赢

| | Ours | Readability.js | innerText |
|---|---|---|---|
| 链接 | 完整保留 `[Title](url)` | 无链接 | 无链接 |
| 格式 | 编号 + 结构化 | 文字挤一行 | tab 分隔 |
| 信息量 | 6K (最多) | 4K | 4K |

```
[Ours]        1. [LittleSnitch for Linux](https://obdev.at/...) ( obdev.at )
              301 points by pluc 3 hours ago | hide | 115 comments

[Readability] 1.LittleSnitch for Linux (obdev.at)301 points by pluc 3 hours ago | hide | 115 comments

[innerText]   1.  LittleSnitch for Linux (obdev.at)
              301 points by pluc 3 hours ago | hide | 115 comments
```

### Example (极简) — 我们赢

| | Ours | Readability.js | innerText |
|---|---|---|---|
| 标题 | `# Example Domain` | 丢失 | 纯文本 |
| 链接 | `[Learn more](url)` | 丢失 | 纯文本 |

### Wikipedia (大页面) — 我们赢

| | Ours | Readability.js | innerText |
|---|---|---|---|
| 结构 | 表格 + 标题 + 段落 | 文字挤一起无空格 | 含导航噪声 |
| 信息表 | Markdown table | 全部连成一串 | 正常 |

```
[Ours]        | Paradigms | Concurrent functional generic imperative structured |
[Readability] ParadigmsConcurrentfunctionalgenericimperativestructured
[innerText]   Jump to content Main menu Main menu move to sidebar hide...
```

### Python.org (主页) — 我们赢

| | Ours | Readability.js | innerText |
|---|---|---|---|
| 内容 | 干净正文 + 链接 | 含 "JS disabled" 噪声 | 7K 含大量噪声 |
| 大小 | 2.6K (最紧凑) | 2.7K | 7K |

```
[Ours]        ## Get Started
              Whether you're new to programming or an experienced developer...

[Readability] Notice: This page displays a fallback because interactive
              scripts did not run...

[innerText]   Notice: This page displays a fallback because interactive
              scripts did not run... Skip to content ▼ Close Python PS...
```

### Blog (文章) — 我们赢

| | Ours | Readability.js | innerText |
|---|---|---|---|
| 标题 | `# The Future of Everything is Lies, I Guess` | 丢失 | 含导航噪声 |
| 导航 | 已去除 | 已去除 | 包含 "Aphyr About Blog Photos Code" |
| 目录 | 保留 Table of Contents | 保留 | 保留 |

## 综合评分

| 维度 | Ours | Readability.js | innerText |
|---|---|---|---|
| 速度 | **最快 (4/5)** | 最慢 | 中等 |
| 需要浏览器 | **不需要** | 需要 | 需要 |
| 标题保留 | **Markdown 标题** | 经常丢失 | 纯文本 |
| 链接保留 | **完整 `[text](url)`** | 丢失 | 丢失 |
| 去噪能力 | **强** | 中（文章好，列表/主页差） | 弱 |
| 列表页支持 | **好** | 差（专为文章设计） | 一般 |
| Token 效率 | **高（紧凑 Markdown）** | 中（缺结构挤成一团） | 低（含大量噪声） |

## 结论

1. **Our Distiller 全场最佳** — 速度最快、不需要浏览器、Markdown 结构化输出、链接和标题完整保留
2. **Readability.js 局限明显** — 专为博客/新闻文章设计，列表页（HN）和主页（Python.org）表现差，经常丢标题和链接
3. **Raw innerText 噪声太多** — 包含导航栏、菜单等无用内容，不适合直接喂给 LLM

## 优化方向

1. Wikipedia 大页面速度：换用 `lol_html` 流式解析替代 `scraper` 全量 DOM 构建
2. 考虑 T1 路径注入 Readability.js 作为文章检测辅助（识别正文区域），但最终提取仍走 Rust distiller
