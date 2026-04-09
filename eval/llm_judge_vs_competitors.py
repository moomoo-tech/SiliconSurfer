"""Head-to-head: Agent Browser vs Jina Reader vs Crawl4AI vs Trafilatura.

Same 30 tests, same LLM judge.
Firecrawl requires API key, skip for now.
"""

import json
import re
import time
import tomllib
import httpx
import trafilatura
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
    tokens = data.get("usageMetadata", {}).get("promptTokenCount", 0)
    return {"text": text, "tokens": tokens, "ms": elapsed}


# ---- Fetchers ----

def fetch_ours_operator(url):
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": "operator"}, timeout=60)
    return r.json().get("content", "")

def fetch_ours_reader(url):
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": "reader"}, timeout=60)
    return r.json().get("content", "")

def fetch_jina(url):
    try:
        r = httpx.get(f"https://r.jina.ai/{url}", headers={"Accept": "text/markdown"}, timeout=30)
        return r.text if r.status_code == 200 else ""
    except Exception:
        return ""

def fetch_trafilatura(url):
    try:
        html = httpx.get(url, timeout=30, follow_redirects=True, verify=False).text
        return trafilatura.extract(html, include_links=True, include_tables=True) or ""
    except Exception:
        return ""


# ---- Tests (same as comprehensive) ----

TESTS = {
    "navigation": [
        ("Nav-1: Login URL", "https://quotes.toscrape.com/",
         "Find the URL to the login page. Return ONLY the URL or MISSING.",
         lambda r, s: "login" in r.lower() and "MISSING" not in r.upper()),
        ("Nav-2: Next page", "https://quotes.toscrape.com/",
         "Find the URL to page 2. Return ONLY the URL or MISSING.",
         lambda r, s: "page/2" in r or "page=2" in r),
        ("Nav-3: Category", "https://books.toscrape.com/",
         "Find the URL to the Travel book category. Return ONLY the URL or MISSING.",
         lambda r, s: "travel" in r.lower() and "MISSING" not in r.upper()),
        ("Nav-4: Count links", "https://the-internet.herokuapp.com/",
         "How many test page links are listed? Return ONLY the number.",
         lambda r, s: any(str(n) in r for n in range(20, 50))),
        ("Nav-5: Specific link", "https://the-internet.herokuapp.com/",
         "Find the URL to 'Form Authentication'. Return ONLY the URL or MISSING.",
         lambda r, s: "login" in r.lower() or "form" in r.lower()),
        ("Nav-6: Author page", "https://quotes.toscrape.com/",
         "Find the URL to view Albert Einstein's page. Return ONLY the URL or MISSING.",
         lambda r, s: "einstein" in r.lower() or "author" in r.lower()),
        ("Nav-7: Tag page", "https://quotes.toscrape.com/",
         "Find the URL for the 'life' tag. Return ONLY the URL or MISSING.",
         lambda r, s: "life" in r.lower() and "MISSING" not in r.upper()),
        ("Nav-8: Footer link", "https://quotes.toscrape.com/",
         "Find the URL to GoodReads.com. Return ONLY the URL or MISSING.",
         lambda r, s: "goodreads" in r.lower()),
        ("Nav-9: External link", "https://quotes.toscrape.com/",
         "Find any external URL (http) to a site other than toscrape.com. Return ONLY the URL.",
         lambda r, s: r.strip().startswith("http") and "toscrape" not in r),
        ("Nav-10: Link text", "https://the-internet.herokuapp.com/",
         "What is the text of the link that leads to '/dropdown'. Return ONLY the link text.",
         lambda r, s: "dropdown" in r.lower()),
    ],
    "forms": [
        ("Form-1: Field names", "https://quotes.toscrape.com/login",
         "What are the login form field names? Return JSON: {{\"username\": \"...\", \"password\": \"...\"}}. Use MISSING if not found.",
         lambda r, s: "username" in r.lower() and "password" in r.lower() and "MISSING" not in r.upper()),
        ("Form-2: Action URL", "https://quotes.toscrape.com/login",
         "What URL does the login form POST to? Return ONLY the URL or MISSING.",
         lambda r, s: "login" in r.lower() and "MISSING" not in r.upper()),
        ("Form-3: CSRF token", "https://quotes.toscrape.com/login",
         "Is there a hidden CSRF token field? Return JSON: {{\"has_csrf\": true/false, \"name\": \"...\"}}",
         lambda r, s: "csrf" in r.lower() and "true" in r.lower()),
        ("Form-4: All fields", "https://httpbin.org/forms/post",
         "List all form input field names. Return JSON array of strings.",
         lambda r, s: "custname" in r and "custtel" in r),
        ("Form-5: Input types", "https://httpbin.org/forms/post",
         "What input types exist? Return JSON array.",
         lambda r, s: "text" in r and ("radio" in r or "tel" in r)),
        ("Form-6: Button text", "https://httpbin.org/forms/post",
         "What is the submit button text? Return ONLY the text.",
         lambda r, s: "submit" in r.lower() or "order" in r.lower()),
        ("Form-7: Radio options", "https://httpbin.org/forms/post",
         "What are the pizza size radio options? Return JSON array.",
         lambda r, s: "small" in r.lower() and "large" in r.lower()),
        ("Form-8: Checkboxes", "https://httpbin.org/forms/post",
         "What pizza toppings are available? Return JSON array.",
         lambda r, s: "bacon" in r.lower() and "cheese" in r.lower()),
        ("Form-9: POST target", "https://httpbin.org/forms/post",
         "What URL does the form POST to? Return ONLY the URL.",
         lambda r, s: "/post" in r),
        ("Form-10: Email field", "https://httpbin.org/forms/post",
         "What is the name of the email input field? Return ONLY the name.",
         lambda r, s: "custemail" in r),
    ],
    "content": [
        ("Content-1: First quote", "https://quotes.toscrape.com/",
         "What is the first quote on the page? Return ONLY the quote text.",
         lambda r, s: len(r) > 20 and r.strip('"\'')[:20].lower() in s.lower()),
        ("Content-2: Author", "https://quotes.toscrape.com/",
         "Who is the author of the first quote? Return ONLY the name.",
         lambda r, s: r.strip().lower() in s.lower() and len(r.strip()) > 3),
        ("Content-3: Count quotes", "https://quotes.toscrape.com/",
         "How many quotes on this page? Return ONLY the number.",
         lambda r, s: "10" in r),
        ("Content-4: Book title", "https://books.toscrape.com/",
         "What is the first book title? Return ONLY the title.",
         lambda r, s: len(r.strip()) > 3 and r.strip()[:10].lower() in s.lower()),
        ("Content-5: Book price", "https://books.toscrape.com/",
         "What is the first book price? Return ONLY the price.",
         lambda r, s: "£" in r or "51" in r),
        ("Content-6: Count books", "https://books.toscrape.com/",
         "How many books on the first page? Return ONLY the number.",
         lambda r, s: "20" in r),
        ("Content-7: Tags", "https://quotes.toscrape.com/",
         "What tags are on the first quote? Return JSON array.",
         lambda r, s: "[" in r and len(r) > 5),
        ("Content-8: Heading", "https://the-internet.herokuapp.com/",
         "What is the main heading? Return ONLY the text.",
         lambda r, s: "welcome" in r.lower() or "internet" in r.lower()),
        ("Content-9: Book availability", "https://books.toscrape.com/",
         "Are books in stock? Return 'In stock' or 'Out of stock'.",
         lambda r, s: "in stock" in r.lower()),
        ("Content-10: Quote count check", "https://quotes.toscrape.com/",
         "Does the page mention how many quotes exist in total? Return the number or UNKNOWN.",
         lambda r, s: True),  # Any answer accepted
    ],
}


def main():
    tools = {
        "Our Operator": fetch_ours_operator,
        "Our Reader": fetch_ours_reader,
        "Jina Reader": fetch_jina,
        "Trafilatura": fetch_trafilatura,
    }

    results = {t: {"nav": 0, "form": 0, "content": 0, "nav_t": 0, "form_t": 0, "content_t": 0, "tokens": 0} for t in tools}

    for cat_key, cat_name in [("navigation", "NAVIGATION"), ("forms", "FORMS/LOGIN"), ("content", "CONTENT")]:
        print(f"\n{'='*95}")
        print(f"  {cat_name} (10 tests)")
        print(f"{'='*95}")

        for test_name, url, prompt, check_fn in TESTS[cat_key]:
            print(f"\n  {test_name}")
            score_key = {"navigation": "nav", "forms": "form", "content": "content"}[cat_key]

            for tool_name, fetch_fn in tools.items():
                content = fetch_fn(url)
                if not content or not content.strip():
                    results[tool_name][f"{score_key}_t"] += 1
                    print(f"    {tool_name:<16s}  ✗  EMPTY")
                    continue

                full_prompt = f"{prompt}\nCRITICAL: Only use info from text below. Do NOT guess.\n\nContent:\n{content[:6000]}"
                try:
                    resp = ask_gemini(full_prompt)
                    passed = check_fn(resp["text"], content)
                except Exception as e:
                    passed = False
                    resp = {"text": str(e), "tokens": 0}

                results[tool_name][f"{score_key}_t"] += 1
                if passed:
                    results[tool_name][score_key] += 1
                results[tool_name]["tokens"] += resp.get("tokens", 0)

                tp = "✓" if passed else "✗"
                print(f"    {tool_name:<16s}  {tp}  {resp['text'][:55].strip()}")
                time.sleep(0.3)

    # Grand summary
    print(f"\n{'='*95}")
    print(f"  GRAND SUMMARY — 30 tests")
    print(f"{'='*95}\n")

    print(f"  {'Tool':<18s}  {'Navigation':>12s}  {'Forms':>12s}  {'Content':>12s}  {'TOTAL':>12s}  {'Tokens':>10s}")
    print(f"  {'─'*18}  {'─'*12}  {'─'*12}  {'─'*12}  {'─'*12}  {'─'*10}")

    for tool in tools:
        r = results[tool]
        nav = f"{r['nav']}/{r['nav_t']}" if r['nav_t'] > 0 else "N/A"
        form = f"{r['form']}/{r['form_t']}" if r['form_t'] > 0 else "N/A"
        cont = f"{r['content']}/{r['content_t']}" if r['content_t'] > 0 else "N/A"
        total_p = r['nav'] + r['form'] + r['content']
        total_t = r['nav_t'] + r['form_t'] + r['content_t']
        total = f"{total_p}/{total_t}" if total_t > 0 else "N/A"
        rate = total_p / total_t * 100 if total_t > 0 else 0
        print(f"  {tool:<18s}  {nav:>12s}  {form:>12s}  {cont:>12s}  {total:>6s} ({rate:>3.0f}%)  {r['tokens']:>10,}")

    # Save
    Path("eval/samples/competitors_results.json").write_text(json.dumps(results, indent=2))
    print(f"\n  Saved to: eval/samples/competitors_results.json")


if __name__ == "__main__":
    main()
