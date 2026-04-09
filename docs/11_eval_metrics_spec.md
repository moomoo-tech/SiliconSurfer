# 11 LLM Eval Metrics Specification

日期: 2026-04-09

## 概述

三维评估体系，每个维度包含具体指标和评分细则。
总分 = Faithfulness (40%) + Actionability (30%) + Cost (30%)

---

## Dimension 1: Faithfulness 忠实度 (40%)

衡量 distiller 输出是否忠实保留了原始网页的信息，以及 LLM 是否基于输出做出了正确判断。

### 1.1 Strict Substring Match (SSM)

**定义：** LLM 提取的关键实体（URL/价格/名称）必须在 distiller 输出的原文中逐字存在。

**评分细则：**
```
1.0  — 实体在 distiller 输出中逐字匹配
0.5  — 实体部分匹配（如 URL 路径匹配但 domain 不同）
0.0  — 实体不在输出中（幻觉/捏造）
```

**测试方法：**
```python
# 从 LLM 响应中提取 URL/价格等实体
# 逐个检查: assert entity in distiller_output
score = matched_count / total_entities
```

**反面案例：** HN comment URL `item?id=42700346` — LLM 猜出了格式但原文中不存在，SSM=0。

### 1.2 Hallucination Detection

**定义：** LLM 输出中包含无法被 distiller 原文支撑的事实。

**评分细则：**
```
0 hallucinations  — 1.0 分
1 hallucination   — 0.5 分  
2+ hallucinations — 0.0 分
```

**检测规则：**
- URL 中的 domain 不在原文中 → hallucination
- 数字（价格/分数/日期）不在原文中 → hallucination
- 人名不在原文中 → hallucination
- LLM 回答 "MISSING" / "not found" → NOT hallucination（诚实）

### 1.3 Information Recall

**定义：** 原文中有 N 个目标实体，distiller 输出保留了几个，LLM 成功提取了几个。

**评分细则：**
```
Distiller Recall = 保留实体数 / 原文总实体数
LLM Recall       = LLM 提取数 / 保留实体数
Combined Recall  = Distiller Recall × LLM Recall
```

**测试方法：**
```python
# Golden标注: HN 页面有 30 条新闻
# Distiller 输出包含 28 条 → Distiller Recall = 93%
# LLM 从输出中提取 25 条 → LLM Recall = 89%
# Combined = 93% × 89% = 83%
```

---

## Dimension 2: Actionability 可执行性 (30%)

衡量 Agent 能否基于 distiller 输出采取行动（点击、跳转、填表）。

### 2.1 URL Completeness

**定义：** 输出中的链接是否可直接执行 HTTP 请求。

**评分细则：**
```
1.0  — 绝对 URL (https://example.com/page)
0.5  — 根相对 URL (/page) — 需要拼接 origin
0.25 — 裸相对 URL (page?id=123) — 需要拼接 base path
0.0  — 无 URL / javascript: / #
```

**计算：**
```python
score = sum(url_score(u) for u in extracted_urls) / total_expected_urls
```

### 2.2 Action Path Retention

**定义：** 完成任务必需的交互路径是否保留。

**评分细则：** 按任务类型定义必需路径：

| 任务 | 必需路径 | 评分 |
|------|---------|------|
| "查看评论" | 评论页 URL | 有=1.0, 无=0.0 |
| "下单购买" | 购买按钮 URL + 商品价格 | 都有=1.0, 缺一=0.5, 都无=0.0 |
| "登录" | 登录页 URL + 表单字段 | 都有=1.0 |
| "搜索" | 搜索框 action URL | 有=1.0 |

### 2.3 Structure Determinism

**定义：** LLM 能否准确识别元素类型（标题/正文/按钮/代码）。

**评分细则：**
```
1.0  — Markdown 结构明确: # Title, [Link](url), ```code```, **bold**
0.7  — 有结构但不完整（标题有但代码块丢失）
0.3  — 纯文本有换行分段
0.0  — 纯文本无结构
```

**检测：**
```python
has_headings = bool(re.findall(r'^#{1,6} ', text, re.MULTILINE))
has_links = '[' in text and '](' in text
has_code = '```' in text
has_lists = bool(re.findall(r'^- ', text, re.MULTILINE))
score = (has_headings + has_links + has_code + has_lists) / 4
```

---

## Dimension 3: Cost 成本 (30%)

衡量 distiller 输出的 token 效率和系统延迟。

### 3.1 Token Compression Ratio

**定义：** distiller 输出的 token 数 / 原始 HTML 的 token 数。

**评分细则：**
```
< 5%   — 1.0 分（极致压缩）
5-15%  — 0.8 分（优秀）
15-30% — 0.5 分（一般）
> 30%  — 0.2 分（臃肿）
```

**计算：** `ratio = len(distilled) / len(raw_html) * 4` (粗略 token 估算)

### 3.2 Signal-to-Noise Ratio (SNR)

**定义：** 有效业务 token / 总 token。

**噪声判定规则：**
```
UI 噪声:     login, sign in, hide, submit, cookie, skip to
导航噪声:     new | past | comments | ask | show | jobs
重复噪声:     完全相同的行出现 2+ 次
结构噪声:     空 markdown 标记 ([](), **, __)
```

**评分细则：**
```
> 95%  — 1.0 分
90-95% — 0.8 分
80-90% — 0.5 分
< 80%  — 0.2 分
```

### 3.3 Prompt Token Cost

**定义：** 完成等效任务消耗的 LLM prompt tokens。

**评分细则：** 相对于最佳工具的比值：
```
best_tokens = min(all_tools_tokens)
ratio = my_tokens / best_tokens
< 1.1  — 1.0 分（接近最优）
1.1-1.3 — 0.8 分
1.3-1.5 — 0.5 分
> 1.5  — 0.2 分
```

### 3.4 E2E Latency (TTFT)

**定义：** 从发出抓取请求到 LLM 输出第一个 token 的总耗时。

**组成：** `fetch_latency + distill_latency + llm_latency`

**评分细则：**
```
< 500ms  (distill only) — 1.0 分
500-2000ms              — 0.7 分
> 2000ms                — 0.3 分
```

---

## 总分计算

```python
total = (
    faithfulness_score * 0.40 +  # SSM + Hallucination + Recall
    actionability_score * 0.30 + # URL Completeness + Action Path + Structure
    cost_score * 0.30            # Compression + SNR + Token Cost + Latency
)

# 各维度内部权重
faithfulness_score = SSM * 0.5 + hallucination * 0.3 + recall * 0.2
actionability_score = url_completeness * 0.4 + action_path * 0.3 + structure * 0.3
cost_score = compression * 0.3 + snr * 0.3 + token_cost * 0.2 + latency * 0.2
```

## 阈值

| 等级 | 总分 | 含义 |
|------|------|------|
| A    | > 0.85 | 生产可用 |
| B    | 0.70-0.85 | 需要优化 |
| C    | 0.50-0.70 | 有严重缺陷 |
| F    | < 0.50 | 不可用 |

## Anti-Hallucination Prompt 规范

所有涉及事实提取的 LLM eval prompt 必须包含：

```
CRITICAL: You MUST only return information that EXACTLY appears in the text below.
Do NOT guess, infer, or construct data from your training knowledge.
If the information is not present in the text, return "MISSING".
```

## 已验证的陷阱

| 陷阱 | 描述 | 防御 |
|------|------|------|
| URL 幻觉 | LLM 从训练数据猜出 HN URL 格式 | Strict Substring Match |
| 格式伪阳性 | JSON 格式正确但内容捏造 | 字段值 in source 校验 |
| 过度去噪 | distiller 删了重要数据导致 LLM 说 MISSING | Information Recall 指标 |
| Token 膨胀 | 保留太多链接导致 LLM 注意力分散 | Token Compression 指标 |
