"""Comprehensive LLM Judge — 30 tests across 3 categories.

Category 1: Navigation (10 tests) — Can Agent find its way around?
Category 2: Forms & Login (10 tests) — Can Agent interact with UI?
Category 3: Content Extraction (10 tests) — Can Agent understand the page?

Tests against: Our Operator, Our Reader, Playwright innerText
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

SERVER = "http://localhost:9883"


def ask_gemini(prompt: str) -> dict:
    t = time.perf_counter()
    resp = httpx.post(GEMINI_URL, json={"contents": [{"parts": [{"text": prompt}]}]}, timeout=60)
    elapsed = (time.perf_counter() - t) * 1000
    resp.raise_for_status()
    data = resp.json()
    text = data["candidates"][0]["content"]["parts"][0]["text"]
    usage = data.get("usageMetadata", {})
    return {"text": text, "tokens": usage.get("promptTokenCount", 0), "ms": elapsed}


def fetch_page(url, distill="operator", fast=True):
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "fast": fast, "distill": distill}, timeout=60)
    return r.json().get("content", "")


def fetch_playwright(url):
    """Fetch via Playwright for comparison."""
    from playwright.sync_api import sync_playwright
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--disable-gpu"])
        page = browser.new_page()
        page.goto(url, wait_until="domcontentloaded", timeout=30000)
        content = page.evaluate("() => document.body.innerText")
        browser.close()
    return content


# ==========================================
# TEST DEFINITIONS
# ==========================================

NAVIGATION_TESTS = [
    {
        "name": "Nav-1: Find login page URL",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Find the URL to the login page. Return ONLY the URL. If not found, return MISSING.",
        "check": lambda r, s: ("login" in r.lower()) and (r.strip() in s or "login" in s.lower()),
    },
    {
        "name": "Nav-2: Find next page URL",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Find the URL to go to page 2. Return ONLY the URL. If not found, return MISSING.",
        "check": lambda r, s: "page/2" in r or "page=2" in r,
    },
    {
        "name": "Nav-3: Find category link",
        "url": "https://books.toscrape.com/",
        "prompt": "Find the URL to the 'Travel' book category. Return ONLY the URL. If not found, return MISSING.",
        "check": lambda r, s: "travel" in r.lower() and "MISSING" not in r.upper(),
    },
    {
        "name": "Nav-4: Count navigation links",
        "url": "https://the-internet.herokuapp.com/",
        "prompt": "Count how many test page links are listed on this page. Return ONLY the number.",
        "check": lambda r, s: any(str(n) in r for n in range(20, 50)),  # ~44 links
    },
    {
        "name": "Nav-5: Find specific link by text",
        "url": "https://the-internet.herokuapp.com/",
        "prompt": "Find the URL to the 'Form Authentication' test page. Return ONLY the URL. If not found, return MISSING.",
        "check": lambda r, s: "login" in r.lower() or "form" in r.lower(),
    },
    {
        "name": "Nav-6: Find author page",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Find the URL to view more about Albert Einstein. Return ONLY the URL. If not found, return MISSING.",
        "check": lambda r, s: "einstein" in r.lower() or "author" in r.lower(),
    },
    {
        "name": "Nav-7: Find tag page",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Find the URL for the 'life' tag. Return ONLY the URL. If not found, return MISSING.",
        "check": lambda r, s: "tag/life" in r.lower() or "life" in r.lower(),
    },
    {
        "name": "Nav-8: Find footer link",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Find the URL to GoodReads.com mentioned in the footer. Return ONLY the URL.",
        "check": lambda r, s: "goodreads" in r.lower(),
    },
    {
        "name": "Nav-9: Find last page number",
        "url": "https://books.toscrape.com/",
        "prompt": "What is the total number of pages of books? Look for pagination info. Return ONLY the number.",
        "check": lambda r, s: "50" in r or "page" in s.lower(),
    },
    {
        "name": "Nav-10: Find external link",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Find any external URL (starting with http) that links to a website other than quotes.toscrape.com. Return ONLY the URL.",
        "check": lambda r, s: r.startswith("http") and "toscrape" not in r,
    },
]

FORM_TESTS = [
    {
        "name": "Form-1: Login field names",
        "url": "https://quotes.toscrape.com/login",
        "prompt": "What are the form field names for username and password? Return as JSON: {{\"username_field\": \"...\", \"password_field\": \"...\"}}. If not found, use MISSING.",
        "check": lambda r, s: "username" in r.lower() and "password" in r.lower() and "MISSING" not in r.upper(),
    },
    {
        "name": "Form-2: Form action URL",
        "url": "https://quotes.toscrape.com/login",
        "prompt": "What is the form's action URL and HTTP method? Return as JSON: {{\"action\": \"...\", \"method\": \"...\"}}",
        "check": lambda r, s: "login" in r.lower() and ("post" in r.lower() or "POST" in r),
    },
    {
        "name": "Form-3: Hidden CSRF token",
        "url": "https://quotes.toscrape.com/login",
        "prompt": "Is there a hidden CSRF token field? What is its field name? Return as JSON: {{\"has_csrf\": true/false, \"field_name\": \"...\"}}",
        "check": lambda r, s: "csrf" in r.lower() and "true" in r.lower(),
    },
    {
        "name": "Form-4: Pizza order fields",
        "url": "https://httpbin.org/forms/post",
        "prompt": "List all form input field names on this page. Return as JSON array of strings.",
        "check": lambda r, s: "custname" in r and "custtel" in r,
    },
    {
        "name": "Form-5: Input types",
        "url": "https://httpbin.org/forms/post",
        "prompt": "What input types are used in this form? (text, email, radio, etc.) Return as JSON array of unique types.",
        "check": lambda r, s: "text" in r and "radio" in r,
    },
    {
        "name": "Form-6: Submit button text",
        "url": "https://httpbin.org/forms/post",
        "prompt": "What is the text on the submit button? Return ONLY the button text.",
        "check": lambda r, s: "submit" in r.lower() or "order" in r.lower(),
    },
    {
        "name": "Form-7: Radio options",
        "url": "https://httpbin.org/forms/post",
        "prompt": "What are the pizza size options (radio buttons)? Return as JSON array.",
        "check": lambda r, s: "small" in r.lower() and "medium" in r.lower() and "large" in r.lower(),
    },
    {
        "name": "Form-8: Checkbox options",
        "url": "https://httpbin.org/forms/post",
        "prompt": "What pizza toppings are available as checkboxes? Return as JSON array.",
        "check": lambda r, s: "bacon" in r.lower() and "cheese" in r.lower(),
    },
    {
        "name": "Form-9: Form POST target",
        "url": "https://httpbin.org/forms/post",
        "prompt": "What URL does the form POST to? Return ONLY the URL.",
        "check": lambda r, s: "/post" in r,
    },
    {
        "name": "Form-10: Email field",
        "url": "https://httpbin.org/forms/post",
        "prompt": "What is the name of the email input field? Return ONLY the field name.",
        "check": lambda r, s: "custemail" in r,
    },
]

CONTENT_TESTS = [
    {
        "name": "Content-1: First quote text",
        "url": "https://quotes.toscrape.com/",
        "prompt": "What is the first quote on the page? Return ONLY the quote text (without author).",
        "check": lambda r, s: len(r) > 20 and r.strip('"').strip("'")[:30].lower() in s.lower(),
    },
    {
        "name": "Content-2: First quote author",
        "url": "https://quotes.toscrape.com/",
        "prompt": "Who is the author of the first quote on this page? Return ONLY the name.",
        "check": lambda r, s: r.strip().lower() in s.lower() and len(r.strip()) > 3,
    },
    {
        "name": "Content-3: Count quotes on page",
        "url": "https://quotes.toscrape.com/",
        "prompt": "How many quotes are displayed on this page? Return ONLY the number.",
        "check": lambda r, s: "10" in r,
    },
    {
        "name": "Content-4: First book title",
        "url": "https://books.toscrape.com/",
        "prompt": "What is the title of the first book displayed? Return ONLY the title.",
        "check": lambda r, s: len(r.strip()) > 3 and r.strip().lower()[:15] in s.lower(),
    },
    {
        "name": "Content-5: First book price",
        "url": "https://books.toscrape.com/",
        "prompt": "What is the price of the first book? Return ONLY the price (e.g. £51.77).",
        "check": lambda r, s: "£" in r or "51" in r,
    },
    {
        "name": "Content-6: Count books on page",
        "url": "https://books.toscrape.com/",
        "prompt": "How many books are displayed on the first page? Return ONLY the number.",
        "check": lambda r, s: "20" in r,
    },
    {
        "name": "Content-7: Tags on first quote",
        "url": "https://quotes.toscrape.com/",
        "prompt": "What tags are associated with the first quote? Return as JSON array.",
        "check": lambda r, s: "[" in r and len(r) > 5,
    },
    {
        "name": "Content-8: HerokuApp page title",
        "url": "https://the-internet.herokuapp.com/",
        "prompt": "What is the main heading on this page? Return ONLY the heading text.",
        "check": lambda r, s: "welcome" in r.lower() or "internet" in r.lower(),
    },
    {
        "name": "Content-9: Book rating",
        "url": "https://books.toscrape.com/",
        "prompt": "What star rating does the first book have? (One/Two/Three/Four/Five) Return ONLY the rating word.",
        "check": lambda r, s: r.strip().lower() in ["one", "two", "three", "four", "five"],
    },
    {
        "name": "Content-10: In-stock status",
        "url": "https://books.toscrape.com/",
        "prompt": "Are books on this page in stock or out of stock? Return ONLY 'In stock' or 'Out of stock'.",
        "check": lambda r, s: "in stock" in r.lower(),
    },
]


def run_category(name, tests, tools_content):
    """Run a category of tests against all tools."""
    results = {}
    for tool_name in tools_content:
        results[tool_name] = {"pass": 0, "total": 0, "details": []}

    print(f"\n{'='*90}")
    print(f"  {name} ({len(tests)} tests)")
    print(f"{'='*90}")

    for test in tests:
        print(f"\n  {test['name']}")
        for tool_name, content_fn in tools_content.items():
            content = content_fn(test["url"])
            if not content or not content.strip():
                results[tool_name]["total"] += 1
                results[tool_name]["details"].append({"test": test["name"], "pass": False, "reason": "EMPTY"})
                print(f"    {tool_name:<16s}  ✗  EMPTY")
                continue

            prompt = f"{test['prompt']}\n\nCRITICAL: Only use info from the text below. Do NOT guess.\n\nContent:\n{content[:6000]}"
            try:
                resp = ask_gemini(prompt)
                passed = test["check"](resp["text"], content)
            except Exception as e:
                passed = False
                resp = {"text": str(e), "tokens": 0, "ms": 0}

            results[tool_name]["total"] += 1
            if passed:
                results[tool_name]["pass"] += 1
            results[tool_name]["details"].append({
                "test": test["name"], "pass": passed,
                "response": resp["text"][:80], "tokens": resp["tokens"],
            })
            tp = "✓" if passed else "✗"
            print(f"    {tool_name:<16s}  {tp}  {resp['text'][:60].strip()}")
            time.sleep(0.3)

    return results


def main():
    # Define tools
    tools = {
        "operator": lambda url: fetch_page(url, "operator"),
        "reader": lambda url: fetch_page(url, "reader"),
        "playwright": lambda url: fetch_playwright(url),
    }

    all_results = {}

    # Run all categories
    nav_results = run_category("NAVIGATION (Can Agent find its way?)", NAVIGATION_TESTS, tools)
    form_results = run_category("FORMS & LOGIN (Can Agent interact?)", FORM_TESTS, tools)
    content_results = run_category("CONTENT EXTRACTION (Can Agent understand?)", CONTENT_TESTS, tools)

    # Grand summary
    print(f"\n{'='*90}")
    print(f"  GRAND SUMMARY — 30 tests total")
    print(f"{'='*90}")

    categories = [
        ("Navigation", nav_results),
        ("Forms/Login", form_results),
        ("Content", content_results),
    ]

    print(f"\n  {'Category':<16s}", end="")
    for tool in tools:
        print(f"  {tool:>16s}", end="")
    print()
    print(f"  {'─'*16}", end="")
    for _ in tools:
        print(f"  {'─'*16}", end="")
    print()

    grand = {t: {"pass": 0, "total": 0} for t in tools}
    for cat_name, cat_results in categories:
        print(f"  {cat_name:<16s}", end="")
        for tool in tools:
            p = cat_results[tool]["pass"]
            t = cat_results[tool]["total"]
            grand[tool]["pass"] += p
            grand[tool]["total"] += t
            rate = p / t * 100 if t > 0 else 0
            print(f"  {p:>3d}/{t:<2d} ({rate:>4.0f}%)", end="")
        print()

    print(f"  {'─'*16}", end="")
    for _ in tools:
        print(f"  {'─'*16}", end="")
    print()
    print(f"  {'TOTAL':<16s}", end="")
    for tool in tools:
        p = grand[tool]["pass"]
        t = grand[tool]["total"]
        rate = p / t * 100 if t > 0 else 0
        print(f"  {p:>3d}/{t:<2d} ({rate:>4.0f}%)", end="")
    print()

    # Save
    all_data = {"nav": nav_results, "forms": form_results, "content": content_results, "grand": grand}
    Path("eval/samples/comprehensive_results.json").write_text(json.dumps(all_data, indent=2, default=str))
    print(f"\n  Saved to: eval/samples/comprehensive_results.json")


if __name__ == "__main__":
    main()
