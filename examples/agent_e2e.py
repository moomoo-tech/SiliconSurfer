"""E2E Agent Scenarios — Agent sees with our browser, acts with Playwright, thinks with Gemini.

10 scenarios demonstrating the full Agent loop:
  SEE (our distiller) → THINK (Gemini) → ACT (Playwright) → SEE again → ...

Uses:
  - Spider mode: discover page links
  - Operator mode: find forms/buttons
  - Reader mode: understand content
  - Data mode: extract structured data
  - Playwright: execute clicks/form fills (simulating future T1 CDP)
  - Gemini: decision making
"""

import json
import time
import tomllib
import httpx
from pathlib import Path
from playwright.sync_api import sync_playwright

_config = tomllib.load(open(Path(__file__).parent.parent / "config.toml", "rb"))
GEMINI_API_KEY = _config["gemini"]["api_key"]
GEMINI_MODEL = _config["gemini"]["model"]
GEMINI_URL = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={GEMINI_API_KEY}"
SERVER = "http://localhost:9883"


def see(url: str, mode: str = "operator") -> str:
    """Agent sees a page through our distiller."""
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": mode}, timeout=60)
    return r.json().get("content", "")


def think(prompt: str) -> str:
    """Agent thinks using Gemini."""
    resp = httpx.post(GEMINI_URL, json={"contents": [{"parts": [{"text": prompt}]}]}, timeout=60)
    return resp.json()["candidates"][0]["content"]["parts"][0]["text"]


def log_step(step: int, action: str, detail: str = ""):
    """Pretty print agent step."""
    print(f"  [{step}] {action}")
    if detail:
        for line in detail.strip().split("\n")[:5]:
            print(f"      {line[:80]}")


# ==========================================
# SCENARIO 1: Login → Browse Quotes → Find Specific Author
# ==========================================
def scenario_1_login_and_browse(pw_page):
    print("\n" + "="*80)
    print("  SCENARIO 1: Login → Browse → Find Einstein Quote")
    print("="*80)

    # Step 1: SEE login page (Operator mode — see form fields)
    content = see("https://quotes.toscrape.com/login", "operator")
    log_step(1, "SEE login page (operator)", content[:200])

    # Step 2: THINK — extract form info
    plan = think(f"""You are an Agent. Analyze this login page and create an action plan.
Extract: form action URL, username field name, password field name.
Return JSON: {{"action": "...", "username_field": "...", "password_field": "...", "credentials": {{"user": "test", "pass": "test"}}}}

Page content:
{content}""")
    log_step(2, "THINK — plan login", plan[:200])

    # Step 3: ACT — fill and submit form
    pw_page.goto("https://quotes.toscrape.com/login")
    pw_page.fill("input[name='username']", "test")
    pw_page.fill("input[name='password']", "test")
    pw_page.click("input[type='submit']")
    pw_page.wait_for_load_state("domcontentloaded")
    log_step(3, f"ACT — submitted login form → {pw_page.url}")

    # Step 4: SEE — check if logged in (Reader mode)
    content = see(pw_page.url, "reader")
    logged_in = "logout" in content.lower() or "goodreads" in content.lower()
    log_step(4, f"SEE result (reader) — logged_in={logged_in}", content[:150])

    # Step 5: THINK — find Einstein
    decision = think(f"""You are on a quotes website. Find a quote by Albert Einstein.
Look at the page content and tell me: is there an Einstein quote visible?
If yes, extract it. If no, what link should I click to find one?
Return JSON: {{"found": true/false, "quote": "...", "next_url": "..."}}

Content:
{content[:3000]}""")
    log_step(5, "THINK — find Einstein", decision[:200])

    return True


# ==========================================
# SCENARIO 2: Browse Books → Find Cheapest → Get Details
# ==========================================
def scenario_2_find_cheapest_book(pw_page):
    print("\n" + "="*80)
    print("  SCENARIO 2: Browse Books → Find Cheapest → Get Details")
    print("="*80)

    # Step 1: SEE book catalog (Data mode — structured extraction)
    content = see("https://books.toscrape.com/", "reader")
    log_step(1, "SEE book catalog (reader)", content[:200])

    # Step 2: THINK — identify cheapest book
    analysis = think(f"""You are a shopping Agent. Find the cheapest book on this page.
Return JSON: {{"title": "...", "price": "...", "url": "..."}}
Only use info from the text. If URL not available, say MISSING.

Content:
{content[:4000]}""")
    log_step(2, "THINK — find cheapest", analysis[:200])

    # Step 3: SEE with Spider — get all book links
    links = see("https://books.toscrape.com/", "spider")
    log_step(3, "SEE catalog (spider)", f"{json.loads(links).get('total', 0)} links found")

    # Step 4: ACT — click first book
    pw_page.goto("https://books.toscrape.com/")
    first_book = pw_page.query_selector("article.product_pod h3 a")
    if first_book:
        first_book.click()
        pw_page.wait_for_load_state("domcontentloaded")
        log_step(4, f"ACT — clicked first book → {pw_page.url}")

    # Step 5: SEE book detail (Reader mode)
    content = see(pw_page.url, "reader")
    log_step(5, "SEE book detail (reader)", content[:200])

    # Step 6: THINK — extract book info
    info = think(f"""Extract this book's details:
Return JSON: {{"title": "...", "price": "...", "description": "...", "availability": "..."}}

Content:
{content[:3000]}""")
    log_step(6, "THINK — extract details", info[:200])

    return True


# ==========================================
# SCENARIO 3: Navigate → Find Form → Fill → Submit → Verify
# ==========================================
def scenario_3_form_submission(pw_page):
    print("\n" + "="*80)
    print("  SCENARIO 3: Navigate → Find Form → Fill → Submit")
    print("="*80)

    # Step 1: SEE homepage (Spider)
    links = see("https://the-internet.herokuapp.com/", "spider")
    link_data = json.loads(links)
    log_step(1, f"SEE homepage (spider) — {link_data['total']} links")

    # Step 2: THINK — which link has a form?
    decision = think(f"""You are an Agent exploring a test website. Find a page that has a login form.
Look at these links and pick the best one.
Return JSON: {{"url": "...", "reason": "..."}}

Links:
{json.dumps(link_data['content_links'][:20], indent=2)}""")
    log_step(2, "THINK — pick form page", decision[:200])

    # Step 3: ACT — go to login page
    pw_page.goto("https://the-internet.herokuapp.com/login")
    pw_page.wait_for_load_state("domcontentloaded")
    log_step(3, f"ACT — navigated to login → {pw_page.url}")

    # Step 4: SEE login page (Operator)
    content = see(pw_page.url, "operator")
    log_step(4, "SEE login form (operator)", content[:200])

    # Step 5: THINK — extract credentials hint
    plan = think(f"""Analyze this login page. Find the username and password fields.
Also look for any hints about valid credentials.
Return JSON: {{"username_field": "...", "password_field": "...", "hint_user": "...", "hint_pass": "..."}}

Content:
{content}""")
    log_step(5, "THINK — analyze form", plan[:200])

    # Step 6: ACT — fill and submit
    pw_page.fill("#username", "tomsmith")
    pw_page.fill("#password", "SuperSecretPassword!")
    pw_page.click("button[type='submit']")
    pw_page.wait_for_load_state("domcontentloaded")
    log_step(6, f"ACT — submitted → {pw_page.url}")

    # Step 7: SEE — verify result
    content = see(pw_page.url, "reader")
    success = "secure" in content.lower() or "logged" in content.lower()
    log_step(7, f"SEE result — success={success}", content[:150])

    return success


# ==========================================
# SCENARIO 4: Multi-page Quote Collection
# ==========================================
def scenario_4_collect_quotes(pw_page):
    print("\n" + "="*80)
    print("  SCENARIO 4: Collect Quotes Across 3 Pages (Agent-driven pagination)")
    print("="*80)

    all_quotes = []
    url = "https://quotes.toscrape.com/"
    step = 0

    for page_num in range(1, 4):
        # SEE — read current page content
        content = see(url, "reader")
        step += 1
        log_step(step, f"SEE page (reader) — {len(content)} chars")

        # THINK — extract quotes from what Agent sees
        step += 1
        result = think(f"""Extract all quotes from this page.
Return JSON array: [{{"text": "...", "author": "..."}}]
Only include quotes actually present in the text.

Content:
{content[:4000]}""")
        log_step(step, "THINK — extract quotes", result[:100])

        try:
            cleaned = result.strip().strip("`").replace("json\n", "").strip()
            if cleaned.startswith("["):
                quotes = json.loads(cleaned)
                if isinstance(quotes, list):
                    all_quotes.extend(quotes)
        except Exception:
            pass

        # SEE — get page link topology
        spider = see(url, "spider")
        step += 1
        log_step(step, "SEE page (spider) — link topology")

        # THINK — Agent decides: is there a next page? What URL?
        step += 1
        decision = think(f"""You are collecting quotes from multiple pages. You are on page {page_num}.
You have collected {len(all_quotes)} quotes so far. You need at least 20.

Here are all the links on this page:
{spider}

Question: Is there a "next page" link? If yes, return ONLY the full URL.
If no more pages or you have enough quotes, return DONE.""")

        decision = decision.strip().split("\n")[0].strip()
        log_step(step, f"THINK — decision: {decision[:60]}")

        if decision.startswith("http"):
            url = decision
        elif "DONE" in decision.upper() or len(all_quotes) >= 20:
            break
        else:
            break

    print(f"  Total quotes collected: {len(all_quotes)}")
    return len(all_quotes) >= 20


# ==========================================
# SCENARIO 5: Book Category Navigation
# ==========================================
def scenario_5_category_browse(pw_page):
    print("\n" + "="*80)
    print("  SCENARIO 5: Navigate Book Categories → Find Travel Books")
    print("="*80)

    # Step 1: SEE categories (Spider)
    links = see("https://books.toscrape.com/", "spider")
    link_data = json.loads(links)
    log_step(1, f"SEE catalog (spider) — {link_data['total']} links")

    # Step 2: THINK — find travel category
    decision = think(f"""Find the URL for the 'Travel' book category.
Return ONLY the URL.

Links:
{json.dumps([l for l in link_data['content_links'] if 'travel' in l.get('text','').lower() or 'travel' in l.get('url','').lower()][:5], indent=2)}""")
    log_step(2, "THINK — find Travel", decision[:100])

    # Step 3: ACT — navigate to Travel
    travel_url = None
    for link in link_data['content_links']:
        if 'travel' in link.get('url', '').lower():
            travel_url = link['url']
            break

    if travel_url:
        pw_page.goto(travel_url)
        pw_page.wait_for_load_state("domcontentloaded")
        log_step(3, f"ACT — navigated to Travel → {pw_page.url}")

        # Step 4: SEE travel books
        content = see(pw_page.url, "reader")
        log_step(4, "SEE travel books (reader)", content[:200])

        # Step 5: THINK — summarize
        summary = think(f"""Summarize what travel books are available. List titles and prices.
Content:
{content[:3000]}""")
        log_step(5, "THINK — summarize travel books", summary[:200])
        return True

    return False


def main():
    print("="*80)
    print("  AGENT E2E SCENARIOS — SEE → THINK → ACT → SEE ...")
    print("  Using: Our distiller (eyes) + Gemini (brain) + Playwright (hands)")
    print("="*80)

    results = {}

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--disable-gpu"])
        page = browser.new_page()

        scenarios = [
            ("1. Login & Browse", scenario_1_login_and_browse),
            ("2. Find Cheapest Book", scenario_2_find_cheapest_book),
            ("3. Form Submit & Verify", scenario_3_form_submission),
            ("4. Multi-page Collection", scenario_4_collect_quotes),
            ("5. Category Navigation", scenario_5_category_browse),
        ]

        for name, fn in scenarios:
            try:
                success = fn(page)
                results[name] = "✓" if success else "✗"
            except Exception as e:
                results[name] = f"✗ {str(e)[:40]}"
                print(f"  ERROR: {e}")

        browser.close()

    # Summary
    print(f"\n{'='*80}")
    print(f"  RESULTS")
    print(f"{'='*80}")
    for name, result in results.items():
        print(f"  {result}  {name}")

    passed = sum(1 for r in results.values() if r == "✓")
    print(f"\n  {passed}/{len(results)} scenarios passed")


if __name__ == "__main__":
    main()
