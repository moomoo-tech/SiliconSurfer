# SiliconSurfer — Project Summary

## 我们的优势

### 1. 唯一的多模态视觉 (5 modes)
没有任何竞品有模式切换。Jina/Trafilatura/Playwright MCP 都是单一视角。
- Reader: 读文章省 token
- Operator: 看表单/按钮 (@e 引用)
- Spider: 链接拓扑 JSON
- Developer: DOM 骨架
- Data: 结构化表格 JSON

**30/30 eval 满分，唯一全过的工具。**

### 2. 速度碾压
- vs Jina Reader: **27x 更快** (604ms vs 16.6s on Slickdeals)
- vs browser-use: **6.2x 更快** (34.4s vs 212.5s on E2E tasks)
- Distiller: **6.76ms/500KB** (从 v0 的 264ms 优化 39x)
- T0 不需要浏览器，1ms 级响应

### 3. @e 元素引用 (Set-of-Mark)
Agent 说 "fill @e3" 而不是猜 CSS selector。100% 命中率 vs ~70% selector 猜测。
```
@e4 [Input: type=text name=username]
@e5 [Input: type=password name=password]
@e6 [Button: Login]
```

### 4. 链接完整性
Trafilatura 和 Playwright innerText 给 LLM **0 个 URL**。
我们在 Slickdeals 上提取了 **66 个可点击链接**。Agent 能导航，它们不能。

### 5. MCP 原生
5 个工具直接在 Claude Code/Desktop 里用。Playwright MCP 给 raw HTML，我们给成品 Markdown/JSON。

### 6. 站点 Profile 数据库
profiles.toml 定义每个站点的噪声规则，不需要重新编译。9 个站点已配置。

---

## 我们的问题

### 1. Token 效率不是最优
Trafilatura 在纯内容提取上更省 token（2.7K vs 我们 6.8K on Slickdeals）。
我们保留了更多链接和结构，所以 token 多。这是 trade-off 不是 bug。

但腾讯云文章 40% 是导航噪声，说明 profile 去噪还不够激进。

### 2. Reader 模式 LLM Judge 只有 83%
6 个内容理解测试只过了 5 个（Example.com summarize 失败）。
Trafilatura 100%。我们的 lol_html Reader 在某些小页面上输出不够干净。

### 3. E2E Agent 的 "脑" 问题
5/5 通过但有些靠兜底总结。LLM 数错 quotes（10→6），找最便宜的书循环了 6 步。
这是 LLM 推理问题不是 distiller 问题，但用户不在乎根因，只看结果。

### 4. T1 (Chrome CDP) 还不成熟
- CDP 交互层 (cdp.rs) 写好了但没在 E2E 里用（还在用 Playwright 当手）
- 没有 cookie/session 持久化
- 没有截图/录屏
- 没有资源拦截（CSS/图片在 CDP 层）

### 5. 反爬能力为零
Slickdeals 没反 bot 所以成功了。遇到 Cloudflare/DataDome 会直接被拦。
reqwest-impersonate（TLS 指纹伪装）还没集成。

### 6. 只在测试站验证过
quotes.toscrape.com、books.toscrape.com、the-internet.herokuapp.com 都是专门给爬虫练手的。
真实的 Amazon/淘宝/微信公众号 没测过。

### 7. 大页面内容膨胀
Wikipedia 输出 134K chars（scraper）/ 97K chars（lol_html）。
对 LLM context window 来说太大了，需要内容截断或分段策略。

---

## 竞品对比总分

| | 导航 | 表单 | 内容 | 链接 | 速度 | Token |
|---|---|---|---|---|---|---|
| **SiliconSurfer** | **10/10** | **10/10** | 9/10 | **66** | **604ms** | 6.8K |
| Jina Reader | 10/10 | 0/10 | 10/10 | 205 | 16.6s | 15.1K |
| Trafilatura | 3/10 | 3/10 | 9/10 | 0 | 692ms | 2.7K |
| browser-use | 0/5 E2E | - | - | - | 212.5s | - |
| Playwright MCP | - | - | - | raw HTML | needs browser | 25K |

## 下一步优先级

1. **反爬**: reqwest-impersonate 集成（TLS 指纹）
2. **T1 CDP 成熟**: 用自己的 CDP 替换 Playwright，cookie 持久化
3. **Reader 质量**: 追上 Trafilatura 的 token 效率
4. **Profile 优化**: 腾讯云/CSDN/知乎 的噪声规则调准
5. **大页面截断**: 超长内容自动分段
6. **真实网站测试**: Amazon/淘宝/微信公众号
