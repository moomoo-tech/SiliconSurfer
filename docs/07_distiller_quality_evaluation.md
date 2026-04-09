# 07 Distiller Quality Evaluation Framework

日期: 2026-04-09

## Bug 修复

### MDN 页面内容全丢（0 chars）

**根因：** noise selector `[class*='banner']` 和 `[class*='sidebar']` 子串匹配太激进。

MDN 的 HTML 结构：
```html
<div class="page-layout__banner">      ← 被 [class*='banner'] 匹配！
  <div class="layout__2-sidebars-inline"> ← 被 [class*='sidebar'] 匹配！
    <main id="content">                   ← 主内容在这里面，被连带删除
```

**修复：** 缩小 noise selector 匹配范围
- `[class*='banner']` → `[class*='-banner'][class*='ad']`
- `[class*='sidebar']` → 移除（太多布局容器包含 sidebar 命名）
- `[role='banner']` → 移除（MDN 用 banner role 包裹整页）

**结果：** MDN 从 0 chars → 23,943 chars (scraper) / 24,129 chars (lol_html)

## 评估三大法则（来自架构讨论）

### 1. Token 密度与信噪比 (SNR)

评估 Markdown 有多少有用信息 vs 噪声。

噪声类型：
- UI 交互元素残留（`| hide |`、`login`、`submit`）
- 导航栏（`new | past | comments | ask | show | jobs`）
- 页脚（`Guidelines | FAQ | Lists`）

### 2. 结构确定性

LLM 能否可靠解析？

- `1. [Title](url)` — LLM 预训练语料中极常见，解析成功率 ~100%
- 原始 HTML — 需要 LLM 自己理解标签，成功率不稳定
- 纯文本 — 丢失结构，LLM 无法区分标题和正文

### 3. 可行动性 (Actionability)

Agent 能否基于结果采取行动？

- 完整 URL — Agent 可以直接跳转
- 相对路径 — Agent 需要拼接，容易出错
- 无链接（Playwright innerText） — Agent 完全失去导航能力

## 当前评分：90/100

已有：
- ✓ 完整 URL 保留（绝对路径）
- ✓ Markdown 结构化（`# Title`、`[text](url)`、列表）
- ✓ HTML entity 解码
- ✓ 行级去重
- ✓ layout table 展平
- ✓ 噪声标签剔除

扣分项：
- 顶部导航未完全去除（`new | past | comments | ...`）
- `| hide |` 按钮残留
- 底部 footer 链接残留
- 评论区链接丢失（只保留了文章原文链接）

## 理想输出格式（99 分目标）

```markdown
1. [LittleSnitch for Linux](https://obdev.at/products/littlesnitch-linux/index.html)
   832 points by pluc | 11 hours ago | [283 comments](https://news.ycombinator.com/item?id=...)

2. [Help Keep Thunderbird Alive](https://updates.thunderbird.net/...)
   138 points by playfultones | 4 hours ago | [73 comments](https://news.ycombinator.com/item?id=...)
```

## 评估 Pipeline 三级体系

### Level 1: 启发式指标（无需 LLM，Python 脚本）

- **链接密度**：`[]()` 字符占比（>80% = 导航栏没清干净）
- **Token 压缩率**：原始 HTML tokens / Markdown tokens
- **代码块/正文比**：检测内联 CSS/JS 乱码泄漏
- **噪声关键词**：`hide`、`login`、`submit`、`cookie` 出现次数

### Level 2: 黄金基准对比（Trafilatura 作为 gold standard）

- 100 个测试 URL
- 对比 ROUGE score / difflib 相似度
- 如果我们的引擎与 Trafilatura 重合度 >90% 且速度快 10x → 质量过关

### Level 3: LLM-as-Judge（终极检验）

- 用 Promptfoo 配置评估任务
- 把我们的 Markdown 喂给 LLM，检查能否正确提取关键信息
- 成功率 >95% → 量产质量

## 已采集的 Samples

路径：`eval/samples/`

每个站点 6 个文件：
- `raw.html` — 原始 HTML
- `our_scraper.md` — T0 scraper distiller
- `our_lol_html.md` — T0 lol_html fast distiller
- `trafilatura.md` — Trafilatura (Python gold standard)
- `readability_js.txt` — Mozilla Readability.js
- `playwright_innertext.txt` — 原始 innerText

站点：example, hn, wikipedia, python_org, blog, github, mdn
