"""E2E Agent Scenarios — Our multi-mode vs Jina Reader vs Trafilatura (single mode).

Same 5 scenarios, same Gemini brain. Different eyes.
Tests whether single-mode tools can complete multi-step Agent tasks.
"""

import json
import time
import tomllib
import httpx
import trafilatura
from pathlib import Path
from playwright.sync_api import sync_playwright

_config = tomllib.load(open(Path(__file__).parent.parent / "config.toml", "rb"))
GEMINI_API_KEY = _config["gemini"]["api_key"]
GEMINI_MODEL = _config["gemini"]["model"]
GEMINI_URL = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={GEMINI_API_KEY}"
SERVER = "http://localhost:9883"


def think(prompt: str) -> str:
    resp = httpx.post(GEMINI_URL, json={"contents": [{"parts": [{"text": prompt}]}]}, timeout=60)
    return resp.json()["candidates"][0]["content"]["parts"][0]["text"]


# ---- Three different "eyes" ----

def see_ours(url: str, mode: str = "operator") -> str:
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": mode}, timeout=60)
    return r.json().get("content", "")


def see_jina(url: str) -> str:
    try:
        r = httpx.get(f"https://r.jina.ai/{url}", headers={"Accept": "text/markdown"}, timeout=30)
        return r.text if r.status_code == 200 else ""
    except Exception:
        return ""


def see_trafilatura(url: str) -> str:
    try:
        html = httpx.get(url, timeout=30, follow_redirects=True, verify=False).text
        return trafilatura.extract(html, include_links=True, include_tables=True) or ""
    except Exception:
        return ""


# ---- Scenarios ----

def test_login_form(see_fn, tool_name):
    """Can agent find login form fields and plan a login?"""
    content = see_fn("https://quotes.toscrape.com/login")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""You need to log in. Find the form field names for username and password.
Return JSON: {{"username_field": "...", "password_field": "..."}}.
Use MISSING if not found. Only use info from the text.

Content:
{content[:4000]}""")

    found_user = "username" in result.lower() and "MISSING" not in result.upper()
    found_pass = "password" in result.lower() and "MISSING" not in result.upper()
    return found_user and found_pass, f"user={'✓' if found_user else '✗'} pass={'✓' if found_pass else '✗'}"


def test_find_pagination(see_fn, tool_name):
    """Can agent find the next page URL?"""
    content = see_fn("https://quotes.toscrape.com/")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""Find the URL to go to page 2. Return ONLY the URL. If not found, return MISSING.
Only use URLs from the text.

Content:
{content[:4000]}""")

    found = "page/2" in result and "MISSING" not in result.upper()
    return found, result.strip()[:60]


def test_find_category(see_fn, tool_name):
    """Can agent navigate to a book category?"""
    content = see_fn("https://books.toscrape.com/")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""Find the URL to the 'Travel' book category. Return ONLY the URL or MISSING.

Content:
{content[:4000]}""")

    found = "travel" in result.lower() and "MISSING" not in result.upper()
    return found, result.strip()[:60]


def test_extract_book_price(see_fn, tool_name):
    """Can agent extract structured data from a product page?"""
    content = see_fn("https://books.toscrape.com/")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""Find the first book's title and price. Return JSON: {{"title": "...", "price": "..."}}

Content:
{content[:4000]}""")

    has_price = "£" in result or "51" in result
    has_title = "light" in result.lower() or len(result) > 30
    return has_price, f"price={'✓' if has_price else '✗'} title={'✓' if has_title else '✗'}"


def test_find_form_fields(see_fn, tool_name):
    """Can agent find all form input fields?"""
    content = see_fn("https://httpbin.org/forms/post")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""List all form input field names. Return as JSON array of strings.

Content:
{content[:4000]}""")

    has_custname = "custname" in result
    has_custtel = "custtel" in result
    return has_custname and has_custtel, f"custname={'✓' if has_custname else '✗'} custtel={'✓' if has_custtel else '✗'}"


def test_find_csrf(see_fn, tool_name):
    """Can agent find hidden CSRF tokens?"""
    content = see_fn("https://quotes.toscrape.com/login")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""Is there a hidden CSRF token field on this page? What is its name?
Return JSON: {{"has_csrf": true/false, "name": "..."}}

Content:
{content[:4000]}""")

    return "csrf" in result.lower() and "true" in result.lower(), result.strip()[:60]


def test_find_author_link(see_fn, tool_name):
    """Can agent find a specific author's page URL?"""
    content = see_fn("https://quotes.toscrape.com/")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""Find the URL to Albert Einstein's author page. Return ONLY the URL or MISSING.

Content:
{content[:4000]}""")

    return "einstein" in result.lower() and "MISSING" not in result.upper(), result.strip()[:60]


def test_count_items(see_fn, tool_name):
    """Can agent count items on a page?"""
    content = see_fn("https://the-internet.herokuapp.com/")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""How many test page links are on this page? Return ONLY the number.

Content:
{content[:4000]}""")

    return any(str(n) in result for n in range(30, 50)), result.strip()[:20]


def test_radio_options(see_fn, tool_name):
    """Can agent identify radio button options?"""
    content = see_fn("https://httpbin.org/forms/post")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""What are the pizza size radio button options? Return as JSON array.

Content:
{content[:4000]}""")

    return "small" in result.lower() and "large" in result.lower(), result.strip()[:60]


def test_external_link(see_fn, tool_name):
    """Can agent find external URLs?"""
    content = see_fn("https://quotes.toscrape.com/")
    if not content.strip():
        return False, "EMPTY page"

    result = think(f"""Find any external URL (starting with http) linking to a site other than toscrape.com.
Return ONLY the URL or MISSING.

Content:
{content[:4000]}""")

    return result.strip().startswith("http") and "toscrape" not in result, result.strip()[:60]


def main():
    tools = {
        "Our Multi-mode": lambda url: see_ours(url, "operator"),
        "Jina Reader": see_jina,
        "Trafilatura": see_trafilatura,
    }

    tests = [
        ("Login form fields", test_login_form),
        ("Find pagination URL", test_find_pagination),
        ("Find category URL", test_find_category),
        ("Extract book price", test_extract_book_price),
        ("Find form field names", test_find_form_fields),
        ("Find CSRF token", test_find_csrf),
        ("Find author link", test_find_author_link),
        ("Count page items", test_count_items),
        ("Radio button options", test_radio_options),
        ("Find external link", test_external_link),
    ]

    results = {t: {"pass": 0, "total": 0, "details": []} for t in tools}

    print("=" * 95)
    print("  E2E AGENT SCENARIOS — Our Multi-mode vs Jina vs Trafilatura")
    print("  Same brain (Gemini), different eyes. 10 tasks.")
    print("=" * 95)

    for test_name, test_fn in tests:
        print(f"\n  {test_name}")
        for tool_name, see_fn in tools.items():
            passed, detail = test_fn(see_fn, tool_name)
            results[tool_name]["total"] += 1
            if passed:
                results[tool_name]["pass"] += 1
            results[tool_name]["details"].append({"test": test_name, "pass": passed, "detail": detail})

            tp = "✓" if passed else "✗"
            print(f"    {tool_name:<18s}  {tp}  {detail[:55]}")
            time.sleep(0.3)

    # Summary
    print(f"\n{'='*95}")
    print(f"  SUMMARY — 10 Agent Tasks")
    print(f"{'='*95}\n")

    print(f"  {'Tool':<20s}  {'Passed':>8s}  {'Total':>7s}  {'Rate':>7s}")
    print(f"  {'─'*20}  {'─'*8}  {'─'*7}  {'─'*7}")
    for tool in tools:
        r = results[tool]
        rate = r["pass"] / r["total"] * 100 if r["total"] > 0 else 0
        print(f"  {tool:<20s}  {r['pass']:>8d}  {r['total']:>7d}  {rate:>5.0f}%")

    # Per-test comparison
    print(f"\n  {'Test':<25s}", end="")
    for tool in tools:
        print(f"  {tool:>18s}", end="")
    print()
    print(f"  {'─'*25}", end="")
    for _ in tools:
        print(f"  {'─'*18}", end="")
    print()
    for i, (test_name, _) in enumerate(tests):
        print(f"  {test_name:<25s}", end="")
        for tool in tools:
            d = results[tool]["details"][i]
            tp = "✓" if d["pass"] else "✗"
            print(f"  {tp:>18s}", end="")
        print()

    Path("eval/samples/e2e_competitors.json").write_text(json.dumps(results, indent=2, default=str))
    print(f"\n  Saved to: eval/samples/e2e_competitors.json")


if __name__ == "__main__":
    main()
