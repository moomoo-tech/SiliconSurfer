# 05 Architecture: Two Scenarios on One Engine

日期: 2026-04-09

## 引擎底座（已完成）

```
              ┌─────────────────┐  ┌──────────────────┐
              │ 场景1: 执行官     │  │ 场景2: 逻辑探针    │
              │ Cookie注入       │  │ 冒烟测试          │
              │ CDP点击/表单     │  │ DOM快照对比        │
              │ 反爬伪装         │  │ 回归扫描          │
              └────────┬────────┘  └────────┬─────────┘
                       │                     │
              ┌────────┴─────────────────────┴─────────┐
              │         Agent Browser Core              │
              │  T0 (reqwest + lol_html)   ← 已完成     │
              │  T1 (Chrome daemon + CDP)  ← 已完成     │
              │  Router (auto/t0/t1)       ← 已完成     │
              │  Distiller (scraper + lol_html) ← 已完成 │
              └─────────────────────────────────────────┘
```

## 场景 1：执行官（生产力工具）

Agent 不只是看网页，还要操作网页——登录、下单、抢 Deal。

### 1.1 状态注入：跳过登录墙

不要让 Agent 每次走输入账号密码流程。

- 人工或专用登录模块获取一次 Cookie / Storage State
- 序列化为 JSON，持久化存储
- Rust 启动 Context 时注入 Cookie，页面打开即登录态
- 跳过 80% 繁琐步骤

### 1.2 CDP 交互：降维打击

Agent 不需要找像素坐标，直接操作 DOM。

- lol_html 清洗后的 Markdown 里找到按钮的 selector
- Rust 通过 CDP 发送 `Runtime.evaluate` 或 `Input.dispatchMouseEvent`
- 直接在内存里触发 JS 函数，不需要渲染
- 比 Playwright "等待渲染→找坐标→点击" 快一个量级

### 1.3 逻辑验证闭环

- 点击"下单"后，页面跳转
- Rust 引擎瞬间抓取新页面 DOM
- lol_html 扫描到 "Order Confirmed" / "订单号：#XXXX" → 成功
- 扫到 "Out of Stock" / "Payment Failed" → 反馈给 LLM 重试

### 1.4 反爬对抗（必须解决）

登录下单场景必须面对风控：

- **TLS 指纹伪装**：reqwest-impersonate，模拟 Chrome/Safari 握手特征
- **HTTP/2 帧特征**：伪装 SETTINGS、窗口大小
- **验证码**：集成第三方打码平台 或 人工介入
- **行为模拟**：随机延迟、鼠标轨迹（通过 CDP Input domain）

### 1.5 性能对比

| 维度 | Playwright/Selenium | Pyre Rust Engine |
|------|--------------------|--------------------|
| 内存/实例 | 200-500MB | 20-50MB |
| 启动耗时 | 500ms-2s | < 50ms (Context) |
| 网络流量 | 全站资源 | 仅 HTML/JSON |
| 并发能力 | 几十个 | 数千个 |

## 场景 2：逻辑探针（开发工具）

Agent 写完代码后的极速反馈回路。不是 QA，是研发自测。

### 2.1 冒烟测试（Sanity Check）

Agent 写完一个功能，只需确认：

- **服务没挂**：HTTP 200?
- **组件挂载了**：DOM 里有 `<div id="stats-chart">`?
- **数据注入了**：`window.__INITIAL_STATE__` 值正确?

Rust 层 10-30ms 完成，Agent 可以秒级迭代。

### 2.2 状态即真相（State as Truth）

Agent 看不见像素，不需要 Vision 模型。

- 直接读 DOM 结构，比截图+OCR 快 100x
- 检查 `data-timestamp` 属性是否正确更新
- 测的是"逻辑对不对"，不是"看起来对不对"

### 2.3 逻辑快照对比（Zero Regression）

Agent 维护一个 DOM 逻辑快照库：

- 改代码前：扫描关键页面的 DOM 结构，存快照
- 改代码后：再扫一遍，diff 对比
- 导航栏节点丢了？瞬间发现并撤回 CL

### 2.4 地毯式扫描

当前 T0 速度 ~90ms/页（lol_html）：

- 50 个页面全站扫描 < 5 秒
- 别人的 Agent 战战兢兢提交 PR
- 我们的 Agent 提交前已经地毯式验证过全站

### 2.5 开发流理想形态

```
Agent 写代码 → 自动编译/热更新 → 探针出击 (T0, ~90ms)
    → lol_html 清洗 → LLM 验证逻辑 → 通过 → 提交 PR
                                     → 失败 → 自动修复 → 再来
    → 视觉测试交给 CI/CD 里的 Playwright
```

## 两个场景共享的技术栈

| 组件 | 场景1 | 场景2 | 状态 |
|------|-------|-------|------|
| T0 reqwest + lol_html | 抓取页面/API | 冒烟测试 | ✅ 已完成 |
| T1 Chrome daemon + CDP | 交互操作 | JS 渲染页面验证 | ✅ 已完成 |
| Distiller | 提取下单结果 | DOM 快照 | ✅ 已完成 |
| Cookie 注入 | 跳过登录 | 登录态页面测试 | 🔲 待实现 |
| CDP 交互 (click/form) | 下单/抢票 | - | 🔲 待实现 |
| reqwest-impersonate | 反爬伪装 | - | 🔲 待实现 |
| DOM 快照 diff | - | 回归检测 | 🔲 待实现 |
| Python Agent API | tool calling | tool calling | ✅ 已完成 |
