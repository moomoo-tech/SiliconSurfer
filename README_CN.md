# SiliconSurfer 🏄

> 专为硅基生物打造的 MCP 兼容浏览器。

[English](README.md) | 中文

## 为什么不用 Playwright MCP？

Playwright MCP 给 LLM 原始 HTML（25,000 token 的噪声）。SiliconSurfer 给 LLM **成品数据**：

| | Playwright MCP | SiliconSurfer |
|---|---|---|
| 读页面 | 原始 HTML，25K tokens | 干净 Markdown，5K tokens |
| 找表单 | LLM 解析 HTML | `@e3 [Input: name=username]` |
| 获取链接 | LLM 搜索 `<a>` 标签 | `observe(mode="spider")` → JSON |
| 提取表格 | LLM 解析 `<table>` | `observe(mode="data")` → JSON |
| 模式 | 1 种（全部） | 5 种（Reader/Operator/Spider/Developer/Data） |
| 速度 | 每次启动浏览器 | T0 1ms，T1 共享守护进程 |

**评测结果：30/30（Jina 20/30），E2E 5/5（browser-use 0/5），快 6.2 倍。**

## MCP 工具

两个工具，五种视觉模式：

```
observe(url, mode)   # 观察网页
  mode="reader"      → 干净 Markdown（默认）
  mode="operator"    → @e1 @e2 @e3 交互元素引用
  mode="spider"      → JSON 链接拓扑 {nav, content, footer}
  mode="data"        → 结构化 JSON 表格/列表
  mode="developer"   → DOM 骨架 + 属性

act(action, target, value)   # 执行操作
  act("navigate", url)       → 导航到 URL
  act("click", "@e3")        → 点击元素
  act("fill", "@e1", "admin")→ 填写表单
  act("submit", "@e5")       → 提交表单
  act("set_cookies", "", '[{"name":"session","value":"abc","domain":".example.com"}]')
```

工作流：`observe(mode="operator")` → 看到元素 → `act("click", "@e3")` → 再次 `observe`。

## 快速开始

```bash
# 构建 Rust + PyO3 绑定
uv sync --dev

# 配合 Claude Code 使用 — 添加 .mcp.json，重启
uv run python mcp_server.py
```

或以 HTTP 服务器运行：

```bash
cargo build --release -p agent-browser-server
PORT=9883 ./target/release/agent-browser-server
```

## 定位

AI Agent 需要从互联网获取信息和执行操作，但不需要 CSS 渲染、可视化调试等面向人类的功能。

SiliconSurfer 让 AI 用**硅基生物的方式**看网页——5 种视觉模式，@e 元素引用，毫秒级响应。

## 双层架构

根据目标网页的复杂度，自动选择最优抓取策略：

```
             Agent / LLM 请求
                    │
                    ▼
            ┌──────────────┐
            │  路由决策引擎   │
            └──┬────────┬──┘
               │        │
       静态页面 │        │ 动态页面（SPA/JS 渲染/需要交互）
               ▼        ▼
        ┌──────────┐  ┌─────────────────┐
        │ T0: 轻量层 │  │ T1: 无头浏览器层  │
        │ reqwest   │  │ Chromium + CDP  │
        └─────┬────┘  └───────┬─────────┘
              │               │
              ▼               ▼
        ┌─────────────────────────────┐
        │      提炼器 (Distiller)      │
        │  HTML → 干净 Markdown/JSON   │
        └─────────────────────────────┘
                    │
                    ▼
              Agent / LLM
```

### T0：轻量层 — reqwest（静态页面）

纯 HTTP 请求，不启动浏览器。适用于：

- 静态 HTML 页面、博客、文档站
- 开放 API、RSS 源
- 无 JS 渲染依赖的内容页

特点：
- 单机轻松数千并发
- 内存占用极低（KB 级/请求）
- 毫秒级响应

### T1：无头浏览器层 — Chromium + CDP（动态页面）

启动极度阉割的 Chromium 内核，只保留 JS 引擎和 DOM 解析。适用于：

- SPA 单页应用（React/Vue/Angular）
- 数据通过 AJAX/WebSocket 异步加载的页面
- 需要登录、点击、表单交互的场景
- 需要执行 JS 才能拿到内容的页面

特点：
- 全局唯一守护进程，毫秒级创建/销毁隔离 Context
- 拦截 CSS/图片/字体/媒体，只保留 JS 执行和 DOM 构建
- 支持交互：导航、点击、表单填写、提交
- BrowserSession 持久化：observe 和 act 共享同一个 Tab
- Cookie 注入：跳过登录流程，直接空降到认证后页面

### 路由决策

```rust
match mode {
    FetchMode::T0   => fetch_t0(url),      // reqwest → distill
    FetchMode::T1   => fetch_t1(url),      // Chrome → distill
    FetchMode::Auto => {                   // T0 优先，回退 T1
        let result = fetch_t0(url);
        if result.content_length < 100 { fetch_t1(url) }
        else { result }
    }
}
```

## 提炼器 (Distiller)

两层架构的公共出口。无论 HTML 从哪条路径获取，都经过同一套清洗管线：

1. **DOM 降噪**：剔除 `<nav>`、`<footer>`、`<script>`、`<style>`、广告容器
2. **内容定位**：锁定 `<article>`、`<main>` 或主要内容 `<div>`
3. **格式转换**：输出干净的 Markdown 或结构化 JSON
4. **Token 压缩**：将数十万字符的原始 HTML 压缩为数百~数千 Token 的高密度文本

双引擎：`scraper`（DOM AST）用于精确提取，`lol_html`（流式）用于高速批量处理（6.76ms/500KB）。

## 技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| 语言 | Rust | 内存安全、零成本抽象、极致并发性能 |
| 异步运行时 | tokio | Rust 生态事实标准 |
| T0 HTTP 客户端 | reqwest | 极速请求，gzip/brotli/deflate |
| T1 CDP 通信 | chromiumoxide | Rust 生态最成熟的 CDP 封装 |
| HTML 解析 | scraper + lol_html | AST 精确提取 + 流式高速处理 |
| 序列化 | serde + serde_json | 高性能 JSON/结构化输出 |
| Python 桥接 | PyO3 | Rust → Python FFI，零网络开销 |
| HTTP API | axum | 轻量 HTTP 服务端 |

## License

Apache-2.0
