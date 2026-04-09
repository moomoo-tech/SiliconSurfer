"""Collect samples in multiple distill modes + Playwright for comparison."""

import json
import httpx
from pathlib import Path
from playwright.sync_api import sync_playwright

SERVER = "http://localhost:9883"
OUT = Path("eval/samples")

URLS = {
    "toscrape_login": "https://quotes.toscrape.com/login",
    "toscrape_quotes": "https://quotes.toscrape.com/",
    "books_toscrape": "https://books.toscrape.com/",
    "httpbin_forms": "https://httpbin.org/forms/post",
    "herokuapp": "https://the-internet.herokuapp.com/",
}


def main():
    c = httpx.Client(timeout=60)

    for name, url in URLS.items():
        print(f"\n  Collecting: {name}")
        site_dir = OUT / name
        site_dir.mkdir(parents=True, exist_ok=True)

        # Raw HTML
        raw = httpx.get(url, timeout=30, follow_redirects=True, verify=False).text
        (site_dir / "raw.html").write_text(raw, encoding="utf-8")

        # Reader (lol_html, default LLM-friendly)
        r = c.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": "llm_friendly"}).json()
        (site_dir / "reader.md").write_text(r.get("content", ""), encoding="utf-8")
        print(f"    reader: {r.get('content_length', 0):>6,} chars")

        # Operator (lol_html)
        r = c.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": "operator"}).json()
        (site_dir / "operator.md").write_text(r.get("content", ""), encoding="utf-8")
        print(f"    operator: {r.get('content_length', 0):>6,} chars")

        # Spider (lol_html)
        r = c.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": "spider"}).json()
        (site_dir / "spider.json").write_text(r.get("content", ""), encoding="utf-8")
        print(f"    spider: {r.get('content_length', 0):>6,} chars")

        # Scraper (AST reader)
        r = c.post(f"{SERVER}/fetch", json={"url": url, "fast": False}).json()
        (site_dir / "scraper.md").write_text(r.get("content", ""), encoding="utf-8")
        print(f"    scraper: {r.get('content_length', 0):>6,} chars")

    c.close()

    # Playwright innerText
    print("\n  Collecting Playwright...")
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--disable-gpu"])
        page = browser.new_page()
        for name, url in URLS.items():
            page.goto(url, wait_until="domcontentloaded", timeout=30000)
            text = page.evaluate("() => document.body.innerText")
            (OUT / name / "playwright.txt").write_text(text, encoding="utf-8")
            print(f"    {name} playwright: {len(text):>6,} chars")
        browser.close()

    print("\n  Done.")


if __name__ == "__main__":
    main()
