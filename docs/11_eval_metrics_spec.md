# 11 LLM Eval Metrics Specification

日期: 2026-04-09

## 三维评估体系

### Dimension 1: Faithfulness (忠实度)

| Metric | 定义 | 如何测 |
|--------|------|--------|
| Strict Substring Match | LLM 输出的 URL/价格/实体必须 `in raw_markdown` | `assert output_url in markdown_text` |
| Hallucination Rate | LLM 输出中无法被原文支撑的比例 | 交叉验证 or 规则检查 |
| Information Recall | 原文 N 个实体，LLM 提取到了几个 | `extracted / total_in_golden` |

**核心教训：** HN comment URL 测试中 LLM 猜出了 URL 格式，导致假阳性。必须加 Strict Substring Match。

### Dimension 2: Actionability (可执行性)

| Metric | 定义 | 如何测 |
|--------|------|--------|
| Valid Action Path Retention | 任务必需的 URL 是否保留 | 检查 `[text](url)` 中 url 的完整性 |
| DOM Structure Determinism | LLM 能否区分按钮/正文/输入框 | 提取结构化元素的准确率 |

### Dimension 3: Cost (成本)

| Metric | 定义 | 目标 |
|--------|------|------|
| Token Compression | MD tokens / HTML tokens | < 15% |
| SNR | 有效 tokens / (有效 + 噪音 tokens) | > 90% |
| E2E Latency | 抓取 → LLM 首 token | < 500ms |

## 评估规则

```
Score = 0 if:
  - LLM output is empty/FAILED
  - LLM output contains URL not present in source markdown (hallucination)
  
Score = 0.5 if:
  - URL is relative (not directly actionable)
  - Structure not clearly marked

Score = 1.0 if:
  - URL exactly matches source markdown
  - Full absolute URL
  - Structural elements preserved
```
