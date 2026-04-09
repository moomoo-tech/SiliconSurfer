# 10 Multi-mode Vision Architecture

日期: 2026-04-09

## 核心洞察

不同 Agent 任务需要不同的"视界"：
- 总结新闻 → 极致去噪（Reader）
- 点赞/下单 → 保留 UI 按钮（Operator）
- 探索站点 → 只要链接图（Spider）
- 写爬虫脚本 → 要 DOM 骨架（Developer）
- 提取表格数据 → 要结构化输出（Data）

## 5 种模式

### 1. Reader 📖 (已实现 = 当前 distiller)
- 极致信噪比，省 Token
- 杀掉 nav/footer/ads/UI 按钮
- 只保留标题、正文、链接、表格
- 适用: "总结文章"、"提取产品参数"

### 2. Operator 🕹️
- 保留所有可操作锚点
- 不丢弃 UI 节点，用标注区分
- 输出: `[Button: hide](js:...) | [283 comments](/item)`
- 适用: "注册账号"、"加购物车"、"点赞"

### 3. Spider 🕸️
- 只提取链接拓扑图
- 按区域分类: nav_links / content_links / footer_links
- 输出 JSON 数组
- 适用: "找 About Us 页面"、"全站文章链接"
- Token: 极低

### 4. Developer 🛠️
- DOM 骨架 + 属性 (id/class/role/data-*)
- 清空 script/style 内部代码，保留标签结构
- 适用: "写 Playwright 脚本"、"验证 #app 是否渲染"

### 5. Data 📊
- 表格/列表 → CSV 或 JSON
- 抛弃文本段落
- 适用: "提取股票数据"、"对比产品参数"

## 实现策略

一套 lol_html 代码，通过 `enum Mode` 切换 noise selector 和输出格式:

```rust
enum FetchMode {
    Reader,    // 当前默认
    Operator,  // 保留 UI，标注可操作元素
    Spider,    // 只提取链接
    Developer, // DOM 骨架
    Data,      // 结构化表格
}
```

不同模式共享同一个 `rewrite_combined()`，只是:
- Reader: noise selectors 最激进
- Operator: noise selectors 最宽松，加 `[Action: xxx]` 标注
- Spider: 只处理 `<a href>`, 忽略所有文本
- Developer: 保留标签结构，清空 script/style 内容
- Data: 只处理 `<table>` 和列表

## MCP Agent 工作流示例

```
1. Spider 模式 → 扫首页，找到登录链接
2. Developer 模式 → 查看登录表单 id/class
3. T1 CDP 交互 → 填表单登录
4. Reader 模式 → 抓取内部知识库文章
5. Data 模式 → 提取报表数据
```

## 优先级

1. Spider — 最小工作量，最大价值（Agent 导航基础能力）
2. Data — 结构化输出直接对接 function calling
3. Operator — Agent 交互的前置条件
4. Developer — 开发场景的深度支持
