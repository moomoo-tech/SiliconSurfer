"""Level 3: LLM-as-Judge Evaluation.

Uses Gemini to judge whether our distiller output is good enough
for an Agent to complete real tasks.

Tests:
1. Information Extraction — can LLM extract structured data?
2. Actionability — can LLM find the right URL to act on?
3. Comprehension — can LLM summarize the content correctly?
4. Token cost — how many tokens consumed?
"""

import json
import time
import tomllib
import httpx
from pathlib import Path

# Load Gemini config
_config_path = Path(__file__).parent.parent / "config.toml"
with open(_config_path, "rb") as f:
    _config = tomllib.load(f)

GEMINI_API_KEY = _config["gemini"]["api_key"]
GEMINI_MODEL = _config["gemini"]["model"]
GEMINI_URL = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={GEMINI_API_KEY}"

SAMPLES_DIR = Path(__file__).parent / "samples"


def ask_gemini(prompt: str) -> dict:
    """Call Gemini and return response + usage."""
    t = time.perf_counter()
    resp = httpx.post(
        GEMINI_URL,
        json={"contents": [{"parts": [{"text": prompt}]}]},
        timeout=60,
    )
    elapsed = (time.perf_counter() - t) * 1000
    resp.raise_for_status()
    data = resp.json()

    text = data["candidates"][0]["content"]["parts"][0]["text"]
    usage = data.get("usageMetadata", {})
    return {
        "text": text,
        "prompt_tokens": usage.get("promptTokenCount", 0),
        "output_tokens": usage.get("candidatesTokenCount", 0),
        "elapsed_ms": elapsed,
    }


# ---- Test Cases ----

TESTS = [
    # --- Faithfulness tests (strict substring match) ---
    {
        "name": "HN: Extract top 3 URLs (strict)",
        "site": "hn",
        "prompt": """Extract the URLs of the top 3 stories from this Hacker News page.
CRITICAL: You MUST only return URLs that EXACTLY appear in the text below. Do NOT guess or construct URLs.
If a URL is not present in the text, write "MISSING".
Return ONLY a JSON array of 3 URL strings.

Content:
{content}""",
        "validate": "strict_urls_in_source",
        "validate_desc": "3 URLs, each must appear in source markdown (anti-hallucination)",
    },
    {
        "name": "HN: Find comment URL (strict)",
        "site": "hn",
        "prompt": """Find the URL to view comments for the #1 ranked story on this page.
CRITICAL: The URL MUST exactly appear in the provided text. Do NOT guess or construct URLs.
If no comment URL exists in the text, return exactly: MISSING
Return ONLY the URL or MISSING, nothing else.

Content:
{content}""",
        "validate": "strict_url_in_source",
        "validate_desc": "URL must exist in source (catches LLM hallucination)",
    },
    # --- Comprehension tests ---
    {
        "name": "Example: Summarize page",
        "site": "example",
        "prompt": """Summarize this webpage in exactly one sentence.

Content:
{content}""",
        "validate": "example_summary",
        "validate_desc": "Mentions 'example' and 'domain'",
    },
    # --- Extraction tests ---
    {
        "name": "GitHub: Extract repo info",
        "site": "github",
        "prompt": """From this GitHub repository page, extract:
1. Repository name
2. Programming language
3. Description (first sentence)

Return as JSON: {{"name": "...", "language": "...", "description": "..."}}
Only include information that appears in the provided text.

Content:
{content}""",
        "validate": "github_repo",
        "validate_desc": "Valid JSON with 'nickel' in content",
    },
    {
        "name": "MDN: Extract function syntax",
        "site": "mdn",
        "prompt": """From this MDN documentation page, extract one code example of a function declaration.
Return as JSON: {{"example": "..."}}
The code must appear in the provided text.

Content:
{content}""",
        "validate": "mdn_function",
        "validate_desc": "Valid JSON with function code from source",
    },
    {
        "name": "Python.org: Extract event",
        "site": "python_org",
        "prompt": """From this Python.org page, extract any upcoming event mentioned.
Return as JSON: {{"event": "...", "url": "..."}}
Only include URLs that appear in the provided text.

Content:
{content}""",
        "validate": "python_event",
        "validate_desc": "Valid JSON with event info",
    },
]


def _extract_json(text: str) -> str:
    """Extract JSON from possible markdown code block."""
    cleaned = text.strip()
    if cleaned.startswith("```"):
        cleaned = cleaned.split("\n", 1)[1].rsplit("```", 1)[0].strip()
    # Find first [ or {
    for i, ch in enumerate(cleaned):
        if ch in "[{":
            return cleaned[i:]
    return cleaned


def _is_valid_json_array(text: str, min_items: int = 1) -> bool:
    try:
        # Extract JSON from possible markdown code block
        cleaned = text.strip()
        if cleaned.startswith("```"):
            cleaned = cleaned.split("\n", 1)[1].rsplit("```", 1)[0].strip()
        data = json.loads(cleaned)
        return isinstance(data, list) and len(data) >= min_items
    except Exception:
        return False


def _is_valid_json_obj(text: str) -> bool:
    try:
        cleaned = text.strip()
        if cleaned.startswith("```"):
            cleaned = cleaned.split("\n", 1)[1].rsplit("```", 1)[0].strip()
        # Find first { and last }
        start = cleaned.find("{")
        end = cleaned.rfind("}") + 1
        if start >= 0 and end > start:
            json.loads(cleaned[start:end])
            return True
        return False
    except Exception:
        return False


def validate_result(test: dict, response_text: str, source_content: str) -> dict:
    """Multi-dimensional validation with all metrics."""
    scores = {
        "task_pass": False,       # Did LLM complete the task?
        "faithfulness": 1.0,      # Are facts grounded in source? (0-1)
        "hallucination": False,   # Did LLM fabricate data?
        "actionability": 0.0,     # Are URLs actionable? (0-1)
        "detail": "",
    }

    result = response_text.strip()
    validate_type = test["validate"]

    if validate_type == "strict_urls_in_source":
        try:
            cleaned = _extract_json(result)
            urls = json.loads(cleaned)
            if not isinstance(urls, list) or len(urls) < 3:
                scores["detail"] = f"Expected 3+ URLs, got {len(urls) if isinstance(urls, list) else 'non-list'}"
                return scores
            scores["task_pass"] = True
            # Faithfulness: each URL must be in source
            grounded = 0
            hallucinated = []
            for url in urls[:3]:
                if isinstance(url, str) and url in source_content:
                    grounded += 1
                else:
                    hallucinated.append(str(url)[:60])
            scores["faithfulness"] = grounded / 3
            scores["hallucination"] = len(hallucinated) > 0
            # Actionability: URLs must be absolute
            actionable = sum(1 for u in urls[:3] if isinstance(u, str) and u.startswith("http"))
            scores["actionability"] = actionable / 3
            scores["detail"] = f"grounded={grounded}/3 actionable={actionable}/3 hallucinated={hallucinated}"
        except Exception as e:
            scores["detail"] = f"JSON parse error: {e}"

    elif validate_type == "strict_url_in_source":
        if result == "MISSING" or "MISSING" in result.upper():
            scores["task_pass"] = True  # Honest about missing data
            scores["faithfulness"] = 1.0
            scores["detail"] = "Correctly reported MISSING"
        elif result.startswith("http") and result in source_content:
            scores["task_pass"] = True
            scores["faithfulness"] = 1.0
            scores["actionability"] = 1.0
            scores["detail"] = "URL found in source and is absolute"
        elif "item?id=" in result and result in source_content:
            scores["task_pass"] = True
            scores["faithfulness"] = 1.0
            scores["actionability"] = 0.5  # Relative, not directly actionable
            scores["detail"] = "URL found but relative"
        else:
            # Check if LLM hallucinated
            if result not in source_content and len(result) > 10:
                scores["hallucination"] = True
                scores["faithfulness"] = 0.0
                scores["detail"] = f"HALLUCINATION: '{result[:60]}' not in source"
            else:
                scores["detail"] = f"Unexpected output: {result[:60]}"

    elif validate_type == "example_summary":
        scores["task_pass"] = "example" in result.lower() and "domain" in result.lower()
        scores["faithfulness"] = 1.0 if scores["task_pass"] else 0.5
        scores["detail"] = "Summary contains key terms" if scores["task_pass"] else "Missing key terms"

    elif validate_type == "github_repo":
        is_json = _is_valid_json_obj(result)
        has_nickel = "nickel" in result.lower()
        scores["task_pass"] = is_json and has_nickel
        # Check if extracted values are in source
        if is_json:
            obj = json.loads(_extract_json(result))
            grounded = sum(1 for v in obj.values() if isinstance(v, str) and v.lower() in source_content.lower())
            scores["faithfulness"] = grounded / max(len(obj), 1)
        scores["detail"] = f"json={is_json} nickel={has_nickel}"

    elif validate_type == "mdn_function":
        is_json = _is_valid_json_obj(result)
        has_function = "function" in result.lower()
        scores["task_pass"] = is_json and has_function
        # Check if code example is from source
        if is_json:
            try:
                obj = json.loads(_extract_json(result))
                example = obj.get("example", "")
                # Check first meaningful line of code
                first_line = example.strip().split("\n")[0].strip() if example else ""
                scores["faithfulness"] = 1.0 if first_line and first_line in source_content else 0.5
            except Exception:
                scores["faithfulness"] = 0.5
        scores["detail"] = f"json={is_json} function={has_function}"

    elif validate_type == "python_event":
        is_json = _is_valid_json_obj(result)
        scores["task_pass"] = is_json
        if is_json:
            try:
                obj = json.loads(_extract_json(result))
                url = obj.get("url", "")
                if url and url in source_content:
                    scores["faithfulness"] = 1.0
                    scores["actionability"] = 1.0 if url.startswith("http") else 0.5
                elif url:
                    scores["hallucination"] = url not in source_content and url.startswith("http")
                    scores["faithfulness"] = 0.0 if scores["hallucination"] else 0.5
            except Exception:
                pass
        scores["detail"] = f"json={is_json}"

    return scores


def run_test(test: dict, tool_name: str, content: str) -> dict:
    """Run a single test with given content, return multi-dimensional metrics."""
    prompt = test["prompt"].format(content=content[:8000])

    try:
        response = ask_gemini(prompt)
        scores = validate_result(test, response["text"], content)
    except Exception as e:
        return {
            "test": test["name"],
            "tool": tool_name,
            "task_pass": False,
            "faithfulness": 0.0,
            "hallucination": False,
            "actionability": 0.0,
            "error": str(e),
            "prompt_tokens": 0,
            "output_tokens": 0,
            "elapsed_ms": 0,
        }

    return {
        "test": test["name"],
        "tool": tool_name,
        "task_pass": scores["task_pass"],
        "faithfulness": scores["faithfulness"],
        "hallucination": scores["hallucination"],
        "actionability": scores["actionability"],
        "detail": scores["detail"],
        "prompt_tokens": response["prompt_tokens"],
        "output_tokens": response["output_tokens"],
        "elapsed_ms": response["elapsed_ms"],
        "response_preview": response["text"][:200],
    }


def main():
    tools = {
        "our_scraper": "our_scraper.md",
        "our_lol_html": "our_lol_html.md",
        "trafilatura": "trafilatura.md",
        "playwright": "playwright_innertext.txt",
    }

    all_results = []

    print("=" * 90)
    print("  LLM-AS-JUDGE EVALUATION (Gemini)")
    print("=" * 90)

    for test in TESTS:
        print(f"\n  Test: {test['name']}")
        print(f"  Validate: {test['validate_desc']}")
        print(f"  {'Tool':<20s}  {'Task':>5s}  {'Faith':>6s}  {'Hallu':>6s}  {'Action':>7s}  {'Tokens':>7s}  {'ms':>6s}  Detail")
        print(f"  {'─'*20}  {'─'*5}  {'─'*6}  {'─'*6}  {'─'*7}  {'─'*7}  {'─'*6}  {'─'*30}")

        for tool_name, filename in tools.items():
            filepath = SAMPLES_DIR / test["site"] / filename
            if not filepath.exists():
                continue
            content = filepath.read_text(errors="replace")
            if not content.strip():
                print(f"  {tool_name:<20s}  SKIP")
                continue

            result = run_test(test, tool_name, content)
            all_results.append(result)

            tp = "✓" if result["task_pass"] else "✗"
            faith = f"{result['faithfulness']:.0%}"
            hallu = "⚠" if result["hallucination"] else "ok"
            action = f"{result['actionability']:.0%}"
            tokens = result["prompt_tokens"]
            ms = result["elapsed_ms"]
            detail = result.get("detail", "")[:40]
            print(f"  {tool_name:<20s}  {tp:>5s}  {faith:>6s}  {hallu:>6s}  {action:>7s}  {tokens:>7d}  {ms:>5.0f}ms  {detail}")

            time.sleep(0.5)

    # Summary by tool — all dimensions
    print()
    print("=" * 100)
    print("  MULTI-DIMENSIONAL SUMMARY")
    print("=" * 100)

    tool_metrics = {}
    for r in all_results:
        tool = r["tool"]
        if tool not in tool_metrics:
            tool_metrics[tool] = {"pass": 0, "total": 0, "faith_sum": 0.0,
                                  "hallu_count": 0, "action_sum": 0.0, "tokens": 0}
        m = tool_metrics[tool]
        m["total"] += 1
        if r["task_pass"]: m["pass"] += 1
        m["faith_sum"] += r["faithfulness"]
        if r["hallucination"]: m["hallu_count"] += 1
        m["action_sum"] += r["actionability"]
        m["tokens"] += r["prompt_tokens"]

    print(f"\n  {'Tool':<20s}  {'TaskPass':>9s}  {'Faithful':>9s}  {'Hallu':>6s}  {'Action':>7s}  {'Tokens':>8s}")
    print(f"  {'─'*20}  {'─'*9}  {'─'*9}  {'─'*6}  {'─'*7}  {'─'*8}")
    for tool in tools:
        if tool in tool_metrics:
            m = tool_metrics[tool]
            t = m["total"]
            pass_rate = m["pass"] / t if t > 0 else 0
            faith_avg = m["faith_sum"] / t if t > 0 else 0
            hallu = m["hallu_count"]
            action_avg = m["action_sum"] / t if t > 0 else 0
            print(f"  {tool:<20s}  {m['pass']}/{t} {pass_rate:>4.0%}  {faith_avg:>8.0%}  {hallu:>5d}  {action_avg:>6.0%}  {m['tokens']:>8,}")

    # Save results
    output_path = SAMPLES_DIR / "llm_judge_results.json"
    output_path.write_text(json.dumps(all_results, indent=2, ensure_ascii=False))
    print(f"\n  Results saved to: {output_path}")


if __name__ == "__main__":
    main()
