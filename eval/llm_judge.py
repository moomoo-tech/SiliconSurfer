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
    {
        "name": "HN: Extract top 3 stories",
        "site": "hn",
        "prompt": """Based on the following Hacker News page content, extract the top 3 stories.
Return ONLY a JSON array with objects containing: title, url, points, comments_count.
No explanation, just JSON.

Content:
{content}""",
        "validate": lambda result: (
            _is_valid_json_array(result, min_items=3)
            and all("title" in item and "url" in item for item in json.loads(result))
        ),
        "validate_desc": "Valid JSON array with 3+ items, each having title and url",
    },
    {
        "name": "HN: Find comment URL",
        "site": "hn",
        "prompt": """Based on the following Hacker News page content, find the URL to view comments
for the #1 ranked story. Return ONLY the URL, nothing else.

Content:
{content}""",
        "validate": lambda result: "news.ycombinator.com/item" in result or "item?id=" in result,
        "validate_desc": "Contains HN item URL",
    },
    {
        "name": "Example: Summarize page",
        "site": "example",
        "prompt": """Summarize this webpage in exactly one sentence.

Content:
{content}""",
        "validate": lambda result: "example" in result.lower() and "domain" in result.lower(),
        "validate_desc": "Mentions 'example' and 'domain'",
    },
    {
        "name": "GitHub: Extract repo info",
        "site": "github",
        "prompt": """From this GitHub repository page, extract:
1. Repository name
2. Programming language
3. Description (first sentence)

Return as JSON: {{"name": "...", "language": "...", "description": "..."}}

Content:
{content}""",
        "validate": lambda result: "nickel" in result.lower() and _is_valid_json_obj(result),
        "validate_desc": "Valid JSON with 'nickel' in content",
    },
    {
        "name": "MDN: Extract function syntax",
        "site": "mdn",
        "prompt": """From this MDN documentation page, extract:
1. What is a function declaration syntax in JavaScript?
2. Give one code example from the page.

Return as JSON: {{"syntax": "...", "example": "..."}}

Content:
{content}""",
        "validate": lambda result: "function" in result.lower() and _is_valid_json_obj(result),
        "validate_desc": "Valid JSON mentioning 'function'",
    },
    {
        "name": "Python.org: Extract key info",
        "site": "python_org",
        "prompt": """From this Python.org page, extract:
1. Any upcoming event mentioned
2. Any call-to-action URL

Return as JSON: {{"event": "...", "cta_url": "..."}}

Content:
{content}""",
        "validate": lambda result: _is_valid_json_obj(result),
        "validate_desc": "Valid JSON with event info",
    },
]


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


def run_test(test: dict, tool_name: str, content: str) -> dict:
    """Run a single test with given content."""
    prompt = test["prompt"].format(content=content[:8000])  # Truncate to avoid token limits

    try:
        response = ask_gemini(prompt)
        passed = test["validate"](response["text"])
    except Exception as e:
        return {
            "test": test["name"],
            "tool": tool_name,
            "passed": False,
            "error": str(e),
            "prompt_tokens": 0,
            "output_tokens": 0,
            "elapsed_ms": 0,
        }

    return {
        "test": test["name"],
        "tool": tool_name,
        "passed": passed,
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
        print(f"  {'Tool':<20s}  {'Pass':>6s}  {'Prompt':>8s}  {'Output':>8s}  {'Time':>8s}")
        print(f"  {'─'*20}  {'─'*6}  {'─'*8}  {'─'*8}  {'─'*8}")

        for tool_name, filename in tools.items():
            filepath = SAMPLES_DIR / test["site"] / filename
            if not filepath.exists():
                continue
            content = filepath.read_text(errors="replace")
            if not content.strip():
                print(f"  {tool_name:<20s}  {'SKIP':>6s}  {'':>8s}  {'':>8s}  {'':>8s}  (empty)")
                continue

            result = run_test(test, tool_name, content)
            all_results.append(result)

            status = "✓" if result["passed"] else "✗"
            print(f"  {tool_name:<20s}  {status:>6s}  {result['prompt_tokens']:>8d}  {result['output_tokens']:>8d}  {result['elapsed_ms']:>6.0f}ms")

            # Rate limit
            time.sleep(0.5)

    # Summary
    print()
    print("=" * 90)
    print("  SUMMARY")
    print("=" * 90)

    tool_scores = {}
    tool_tokens = {}
    for r in all_results:
        tool = r["tool"]
        if tool not in tool_scores:
            tool_scores[tool] = {"passed": 0, "total": 0}
            tool_tokens[tool] = 0
        tool_scores[tool]["total"] += 1
        if r["passed"]:
            tool_scores[tool]["passed"] += 1
        tool_tokens[tool] += r["prompt_tokens"]

    print(f"\n  {'Tool':<20s}  {'Passed':>8s}  {'Total':>8s}  {'Rate':>8s}  {'Total Tokens':>14s}")
    print(f"  {'─'*20}  {'─'*8}  {'─'*8}  {'─'*8}  {'─'*14}")
    for tool in tools:
        if tool in tool_scores:
            s = tool_scores[tool]
            rate = s["passed"] / s["total"] if s["total"] > 0 else 0
            tokens = tool_tokens[tool]
            print(f"  {tool:<20s}  {s['passed']:>8d}  {s['total']:>8d}  {rate:>7.0%}  {tokens:>14,}")

    # Save results
    output_path = SAMPLES_DIR / "llm_judge_results.json"
    output_path.write_text(json.dumps(all_results, indent=2, ensure_ascii=False))
    print(f"\n  Results saved to: {output_path}")


if __name__ == "__main__":
    main()
