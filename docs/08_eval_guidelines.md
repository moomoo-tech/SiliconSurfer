# 08 Pyre Scraper Benchmark & Eval Guidelines

日期: 2026-04-09

## 三层评估体系

### Layer 1: Micro-Benchmarks (性能指标)

工具: criterion.rs

| 指标 | 目标 | Block 阈值 |
|------|------|-----------|
| Parse Latency P50 | < 50ms | 退化 > 5% |
| Parse Latency P99 | < 200ms | 退化 > 10% |
| Memory Peak (5MB HTML) | < 10MB | 退化 > 20% |
| Throughput | > 20 MB/s | 退化 > 5% |

### Layer 2: Deterministic Eval (结构保真度)

方法: Golden Master Testing (黄金快照)

| 指标 | 公式 | 含义 |
|------|------|------|
| Token 压缩率 | MD tokens / HTML tokens | 越低越好，突然升高 = 漏噪音 |
| 链接留存率 | MD [links] / DOM <a href> | 越高越好，下降 = 丢链接 |
| 代码块保真度 | 缩进/换行完整性 | 回归 = 致命 |
| 表格对齐度 | 列数一致性 | 错位 = 数据污染 |

Regression test cases (已踩过的坑):
- `<pre>` 标签内换行符不能被压缩
- 相对路径 `/foo/bar` 必须拼接 base URL
- `[class*='banner']` 不能误杀布局容器
- layout table 不能渲染成 markdown table
- HTML entity (`&#160;` `&#91;`) 必须解码
- `<title>` 内容不能泄漏到 body

### Layer 3: LLM-as-Judge (业务评估)

工具: promptfoo / RAGAS

| 测试 | Prompt | 断言 |
|------|--------|------|
| 信息提取 | "提取价格和折扣码，JSON 返回" | JSON 100% 匹配 |
| 可行动性 | "第一条新闻的评论链接是什么" | URL 正确 |
| Token 成本 | 记录 prompt tokens | 越少越好 |

## 黄金语料库 (Golden Corpus)

目标: 50 个 HTML + 人工校对的完美 Markdown

分类:
- 列表页 (HN, Reddit, Slickdeals)
- 文章页 (博客, 新闻)
- 文档页 (MDN, Wikipedia, docs.rs)
- 代码仓库 (GitHub, GitLab)
- 商品页 (Amazon, Newegg)
- 主页/Landing (Python.org, Rust-lang.org)
- 高噪音页 (充满广告的资讯站)

## 架构决策记录

### 为什么用 lol_html 流式处理而不是 AST

| | AST (html5ever/scraper) | 流式 (lol_html) |
|---|---|---|
| 内存 | 10-50x HTML 大小 | KB 级恒定 |
| 延迟 | 等全部下载完 | 边下载边处理 |
| 并发 1000 | OOM | 轻松 |
| 上下文感知 | 完整 | 有限 (用状态变量补) |

流式处理的不足用状态变量补偿:
- `is_inside_pre` → 保留换行
- `noise_depth` → 嵌套噪声跟踪
- `base_url` → 相对链接拼接
