# Agent Browser

专为 AI Agent 打造的高并发网页抓取与交互引擎，Rust 实现。

## 定位

AI Agent 需要从互联网获取信息和执行操作，但不需要 CSS 渲染、可视化调试等面向人类的功能。现有方案（Playwright/Puppeteer）太重，无法支撑大规模并发。

Agent Browser 只做一件事：**让 AI 高效地访问和理解网页**。

## 双层架构

根据目标网页的复杂度，自动选择最优抓取策略：

```
             Agent / LLM 请求抓取
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
- 配合 `reqwest-impersonate` 可伪装 TLS 指纹，穿透基础反爬

### T1：无头浏览器层 — Chromium + CDP（动态页面）

启动极度阉割的 Chromium 内核，只保留 JS 引擎和 DOM 解析。适用于：

- SPA 单页应用（React/Vue/Angular）
- 数据通过 AJAX/WebSocket 异步加载的页面（如雪球、微博）
- 需要登录、点击、表单交互的场景
- 需要执行 JS 才能拿到内容的页面

特点：
- 全局唯一守护进程，毫秒级创建/销毁隔离 Context
- 拦截 CSS/图片/字体/媒体，只保留 JS 执行和 DOM 构建
- 单实例内存 ~40-50MB（对比完整浏览器 ~80-100MB）
- 支持交互：导航、点击、表单填写、滚动

### 路由决策

```rust
// 伪代码
async fn fetch(url: &str, hints: &FetchHints) -> CleanContent {
    if hints.needs_js || hints.needs_interaction {
        // T1: 动态渲染
        browser_pool.fetch_and_render(url).await
    } else {
        // T0: 极速 HTTP
        http_client.fetch(url).await
    }
    // 无论哪条路径，最终都经过提炼器
    |> distiller.to_markdown()
}
```

Agent 或调度器可通过 hints 指定策略，也可由引擎根据首次请求结果自动升级（T0 拿到空内容 → 自动回退到 T1）。

## 提炼器 (Distiller)

两层架构的公共出口。无论 HTML 从哪条路径获取，都经过同一套清洗管线：

1. **DOM 降噪**：剔除 `<nav>`、`<footer>`、`<script>`、`<style>`、广告容器
2. **内容定位**：锁定 `<article>`、`<main>` 或主要内容 `<div>`
3. **格式转换**：输出干净的 Markdown 或结构化 JSON
4. **Token 压缩**：将数十万字符的原始 HTML 压缩为数百~数千 Token 的高密度文本

技术实现：Rust 原生库 `scraper`（HTML 解析）+ `html2md`（Markdown 转换），在 Rust 进程内完成，不依赖浏览器。

## 协议层反爬

不同于 JS 层的 stealth 插件（修改 `navigator.webdriver` 等表层属性），Agent Browser 在网络协议底层做伪装：

- **TLS 指纹伪装**：操控 TLS 握手特征（JA3/JA4），模拟真实浏览器指纹
- **HTTP/2 帧控制**：伪装并发流状态机、SETTINGS 帧参数
- **TCP 特征模拟**：窗口大小、MSS 等参数匹配真实浏览器
- **请求头排序**：Header 顺序与真实 Chrome 一致
- **Cookie 与会话管理**：自动维护登录态，支持凭证复用

T0 层通过 `reqwest-impersonate` 实现协议伪装。T1 层天然拥有真实浏览器指纹。

## 两大核心用途

### 1. 生产力工具：Agent 的信息摄取与执行

Agent 的"眼睛"和"手"——从互联网获取数据、执行操作：

- **读 (Read)**：搜索折扣、抓取新闻、采集价格、调研竞品
- **写 (Write)**：登录网站、填写表单、下单购买、触发操作
- 携带 Session Cookie 空降目标页面，绕过登录流程直接执行

### 2. 开发工具：Agent 的逻辑探针

Agent 写完代码后的极速验证（Sanity Check），不是 UI 测试：

- 服务能启动吗？访问 `/api/health` 返回 200？
- 核心 DOM 节点存在吗？`<div id="app">` 有内容？
- API 返回的 JSON 结构正确吗？

毫秒级完成验证，不需要渲染。重型 E2E 测试交给 CI/CD 中的 Playwright。

## 技术选型

| 组件 | 选型 | 理由 |
|------|------|------|
| 语言 | Rust | 内存安全、零成本抽象、极致并发性能 |
| 异步运行时 | tokio | Rust 生态事实标准 |
| T0 HTTP 客户端 | reqwest + reqwest-impersonate | 极速请求 + TLS 指纹伪装 |
| T1 CDP 通信 | chromiumoxide | Rust 生态最成熟的 CDP 封装 |
| HTML 解析 | scraper | Rust 原生极速 DOM 解析 |
| Markdown 转换 | html2md | HTML → Markdown |
| 序列化 | serde | 高性能 JSON/结构化输出 |
| Python 桥接 | PyO3 + pyo3-asyncio | Rust async ↔ Python asyncio 无缝互通 |
| API 层 | tonic (gRPC) + axum (HTTP) | 高性能服务端框架 |

## 与同类方案对比

| | requests | Playwright | Crawl4AI | Browserbase | **Agent Browser** |
|---|---|---|---|---|---|
| 目标用户 | 通用开发者 | 测试工程师 | Python 开发者 | 应用开发者 | AI Agent 基础设施 |
| 语言 | Python | Node/Python | Python | 云服务 | Rust |
| 动态页面 | 不支持 | 支持 | 支持 | 支持 | 支持（T1 层） |
| 并发能力 | 低（GIL） | 中 | 中 | 高（付费） | 极高（T0）/ 高（T1） |
| 反爬层级 | 无 | JS 层 | JS 层 | JS 层 | 协议层 |
| 输出格式 | 原始 HTML | DOM/HTML | Markdown | HTML | Markdown/JSON |
| 资源占用 | 极低 | 高 | 高 | N/A | 极低（T0）/ 中低（T1） |
| LLM 适配 | 无 | 无 | 有 | 无 | 原生（提炼器） |

## 路线图

### Phase 1：MVP — T0 层 + 提炼器

- [ ] reqwest HTTP 抓取
- [ ] scraper + html2md 提炼管线
- [ ] PyO3 异步桥接，暴露给 Python 调度器
- [ ] 跑通静态页面抓取 → 清洗 → 喂给 LLM 的完整链路

### Phase 2：T1 层 — 无头浏览器

- [ ] chromiumoxide 集成，全局守护进程
- [ ] 极度阉割启动参数（禁 GPU/图片/CSS/字体）
- [ ] 毫秒级 Context 创建/销毁
- [ ] 资源拦截器
- [ ] 基础交互（导航、点击、表单、滚动）
- [ ] T0 → T1 自动回退机制

### Phase 3：协议层反爬

- [ ] reqwest-impersonate 集成（T0 层 TLS 伪装）
- [ ] HTTP/2 帧特征伪装
- [ ] Header 顺序与真实 Chrome 对齐
- [ ] Cookie/会话自动管理与凭证复用

### Phase 4：Agent 集成与规模化

- [ ] gRPC / HTTP API
- [ ] 结构化 JSON 输出（适配 LLM function calling）
- [ ] 会话快照与恢复
- [ ] 分布式调度（多节点浏览器池）
- [ ] Serverless 部署支持
- [ ] 监控与可观测性
- [ ] SDK（Python / TypeScript / Rust）

## 快速开始

```bash
# 构建
cargo build --release

# 启动服务
./target/release/agent-browser serve --port 8080

# T0 抓取静态页面（返回 Markdown）
curl http://localhost:8080/fetch \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "output": "markdown"}'

# T1 抓取动态页面（自动启用无头浏览器）
curl http://localhost:8080/fetch \
  -H "Content-Type: application/json" \
  -d '{"url": "https://xueqiu.com", "output": "markdown", "render_js": true}'
```

## License

MIT
