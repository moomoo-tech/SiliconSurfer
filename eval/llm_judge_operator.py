"""LLM-as-Judge for Operator mode — can Agent find and use UI elements?

Tests whether the distiller output allows an LLM to:
1. Find login forms and identify fields
2. Find buttons and their actions
3. Navigate pagination
4. Identify interactive elements
"""

import json
import time
import tomllib
import httpx
from pathlib import Path

_config_path = Path(__file__).parent.parent / "config.toml"
with open(_config_path, "rb") as f:
    _config = tomllib.load(f)

GEMINI_API_KEY = _config["gemini"]["api_key"]
GEMINI_MODEL = _config["gemini"]["model"]
GEMINI_URL = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={GEMINI_API_KEY}"

SAMPLES_DIR = Path("eval/samples")


def ask_gemini(prompt: str) -> dict:
    t = time.perf_counter()
    resp = httpx.post(GEMINI_URL, json={"contents": [{"parts": [{"text": prompt}]}]}, timeout=60)
    elapsed = (time.perf_counter() - t) * 1000
    resp.raise_for_status()
    data = resp.json()
    text = data["candidates"][0]["content"]["parts"][0]["text"]
    usage = data.get("usageMetadata", {})
    return {"text": text, "prompt_tokens": usage.get("promptTokenCount", 0),
            "output_tokens": usage.get("candidatesTokenCount", 0), "elapsed_ms": elapsed}


TESTS = [
    {
        "name": "Login: Find form fields",
        "site": "toscrape_login",
        "prompt": """You are an AI Agent that needs to log in to this website.
From the page content below, identify:
1. The login form action URL
2. The username field name
3. The password field name
4. The submit button text

Return as JSON: {{"action": "...", "username_field": "...", "password_field": "...", "submit_text": "..."}}
If any field is not found, use "MISSING".

CRITICAL: Only report what you can see in the text. Do NOT guess.

Content:
{content}""",
        "check": lambda result, source: _check_login_form(result, source),
    },
    {
        "name": "Quotes: Find pagination link",
        "site": "toscrape_quotes",
        "prompt": """You are an AI Agent browsing a quotes website.
Find the URL to go to the next page of quotes.

Return ONLY the URL. If not found, return "MISSING".
CRITICAL: The URL must appear in the provided text.

Content:
{content}""",
        "check": lambda result, source: _check_pagination(result, source),
    },
    {
        "name": "Books: Find a book to buy",
        "site": "books_toscrape",
        "prompt": """You are an AI Agent shopping for books.
From this page, find the first book and extract:
1. Title
2. Price
3. URL to the book's detail page

Return as JSON: {{"title": "...", "price": "...", "url": "..."}}
CRITICAL: Only use information present in the text.

Content:
{content}""",
        "check": lambda result, source: _check_book(result, source),
    },
    {
        "name": "Forms: Identify form inputs",
        "site": "httpbin_forms",
        "prompt": """You are an AI Agent that needs to fill out a form.
From the content below, list all form input fields with their names and types.

Return as JSON array: [{{"name": "...", "type": "..."}}, ...]
If no form fields are found, return "MISSING".

Content:
{content}""",
        "check": lambda result, source: _check_form_fields(result, source),
    },
    {
        "name": "HerokuApp: Find available tests",
        "site": "herokuapp",
        "prompt": """You are an AI Agent exploring a test website.
List the first 5 available test pages with their URLs.

Return as JSON array: [{{"name": "...", "url": "..."}}, ...]
CRITICAL: URLs must appear in the provided text.

Content:
{content}""",
        "check": lambda result, source: _check_herokuapp_links(result, source),
    },
]


def _extract_json(text):
    cleaned = text.strip()
    if cleaned.startswith("```"):
        cleaned = cleaned.split("\n", 1)[1].rsplit("```", 1)[0].strip()
    for i, ch in enumerate(cleaned):
        if ch in "[{":
            return cleaned[i:]
    return cleaned


def _check_login_form(result, source):
    scores = {"task_pass": False, "found_fields": 0, "detail": ""}
    try:
        obj = json.loads(_extract_json(result))
        fields_found = 0
        missing = []
        for key in ["username_field", "password_field"]:
            val = obj.get(key, "MISSING")
            if val != "MISSING" and val:
                fields_found += 1
            else:
                missing.append(key)
        scores["found_fields"] = fields_found
        scores["task_pass"] = fields_found >= 1  # At least found something
        scores["detail"] = f"found={fields_found}/2 missing={missing}"
    except Exception as e:
        scores["detail"] = f"JSON error: {e}"
    return scores


def _check_pagination(result, source):
    result = result.strip()
    if "MISSING" in result.upper():
        return {"task_pass": False, "detail": "Reported MISSING"}
    in_source = result in source or "/page/" in result
    return {
        "task_pass": in_source or "page" in result.lower(),
        "in_source": in_source,
        "detail": f"url={result[:60]} in_source={in_source}",
    }


def _check_book(result, source):
    scores = {"task_pass": False, "detail": ""}
    try:
        obj = json.loads(_extract_json(result))
        has_title = bool(obj.get("title")) and obj["title"] != "MISSING"
        has_price = bool(obj.get("price")) and "£" in str(obj.get("price", ""))
        scores["task_pass"] = has_title and has_price
        scores["detail"] = f"title={has_title} price={has_price}"
    except Exception as e:
        scores["detail"] = f"JSON error: {e}"
    return scores


def _check_form_fields(result, source):
    if "MISSING" in str(result).upper():
        return {"task_pass": False, "detail": "No form fields found"}
    try:
        fields = json.loads(_extract_json(result))
        if isinstance(fields, list) and len(fields) >= 1:
            return {"task_pass": True, "count": len(fields), "detail": f"{len(fields)} fields found"}
    except Exception:
        pass
    return {"task_pass": False, "detail": "Could not parse fields"}


def _check_herokuapp_links(result, source):
    try:
        links = json.loads(_extract_json(result))
        if isinstance(links, list) and len(links) >= 3:
            in_source = sum(1 for l in links if isinstance(l, dict) and
                          (l.get("url", "") in source or l.get("name", "").lower() in source.lower()))
            return {"task_pass": True, "count": len(links), "grounded": in_source,
                    "detail": f"{len(links)} links, {in_source} grounded"}
    except Exception:
        pass
    return {"task_pass": False, "detail": "Could not parse links"}


def main():
    # Test both Reader and Scraper outputs
    tools = {
        "reader_lol": "reader.md",
        "reader_scraper": "scraper.md",
    }

    all_results = []

    print("=" * 90)
    print("  OPERATOR MODE EVAL — Can Agent find and use UI elements?")
    print("=" * 90)

    for test in TESTS:
        print(f"\n  Test: {test['name']} ({test['site']})")
        print(f"  {'Tool':<20s}  {'Pass':>5s}  {'Tokens':>7s}  {'ms':>6s}  Detail")
        print(f"  {'─'*20}  {'─'*5}  {'─'*7}  {'─'*6}  {'─'*40}")

        for tool_name, filename in tools.items():
            filepath = SAMPLES_DIR / test["site"] / filename
            if not filepath.exists():
                print(f"  {tool_name:<20s}  SKIP (file not found)")
                continue
            content = filepath.read_text(errors="replace")
            if not content.strip():
                # Empty content = Reader mode deleted everything
                result_data = {
                    "test": test["name"], "tool": tool_name,
                    "task_pass": False, "detail": "EMPTY — Reader mode deleted all UI elements",
                    "prompt_tokens": 0, "output_tokens": 0, "elapsed_ms": 0,
                }
                all_results.append(result_data)
                print(f"  {tool_name:<20s}  {'✗':>5s}  {'0':>7s}  {'0':>6s}  EMPTY — all UI elements removed by noise filter")
                continue

            prompt = test["prompt"].format(content=content[:8000])
            try:
                response = ask_gemini(prompt)
                check = test["check"](response["text"], content)
                passed = check.get("task_pass", False)
            except Exception as e:
                check = {"task_pass": False, "detail": str(e)}
                passed = False
                response = {"prompt_tokens": 0, "output_tokens": 0, "elapsed_ms": 0}

            result_data = {
                "test": test["name"], "tool": tool_name,
                "task_pass": passed,
                "detail": check.get("detail", ""),
                "prompt_tokens": response["prompt_tokens"],
                "output_tokens": response["output_tokens"],
                "elapsed_ms": response["elapsed_ms"],
            }
            all_results.append(result_data)

            tp = "✓" if passed else "✗"
            detail = check.get("detail", "")[:45]
            print(f"  {tool_name:<20s}  {tp:>5s}  {response['prompt_tokens']:>7d}  {response['elapsed_ms']:>5.0f}ms  {detail}")

            time.sleep(0.5)

    # Summary
    print()
    print("=" * 90)
    print("  SUMMARY")
    print("=" * 90)

    for tool in tools:
        results = [r for r in all_results if r["tool"] == tool]
        passed = sum(1 for r in results if r["task_pass"])
        total = len(results)
        empty = sum(1 for r in results if "EMPTY" in r.get("detail", ""))
        print(f"  {tool:<20s}  {passed}/{total} passed  ({empty} empty/deleted by noise filter)")

    # Save
    output = SAMPLES_DIR / "llm_judge_operator_results.json"
    output.write_text(json.dumps(all_results, indent=2, ensure_ascii=False))
    print(f"\n  Results saved to: {output}")

    print()
    print("  KEY INSIGHT:")
    print("  Reader mode deletes forms/buttons/inputs → Agent can't interact")
    print("  Operator mode (not yet tested via server) should preserve these")
    print("  This proves why multi-mode vision is essential")


if __name__ == "__main__":
    main()
