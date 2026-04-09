"""Collect distiller output samples for human review.

Saves raw HTML + all distiller outputs for each URL.
"""

import os
import json
import time
import httpx
import trafilatura
from pathlib import Path
from playwright.sync_api import sync_playwright

URLS = {
    "example":    "https://example.com",
    "hn":         "https://news.ycombinator.com/",
    "wikipedia":  "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    "python_org": "https://www.python.org/",
    "blog":       "https://aphyr.com/posts/411-the-future-of-everything-is-lies-i-guess",
    "github":     "https://github.com/nickel-org/nickel.rs",
    "mdn":        "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions",
}

SERVER = "http://localhost:9883"
OUT = Path("eval/samples")


def fetch_raw_html(url: str) -> str:
    """Fetch raw HTML via httpx."""
    r = httpx.get(url, timeout=30, follow_redirects=True, verify=False)
    return r.text


def fetch_our_scraper(url: str) -> dict:
    """Our T0 scraper distiller."""
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "mode": "t0"}, timeout=60)
    return r.json()


def fetch_our_lol_html(url: str) -> dict:
    """Our T0 lol_html fast distiller."""
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "mode": "t0", "fast": True}, timeout=60)
    return r.json()


def fetch_trafilatura(html: str) -> str:
    """Trafilatura extraction (gold standard)."""
    return trafilatura.extract(html, include_links=True, include_tables=True) or ""


def fetch_readability_js(url: str) -> str:
    """Readability.js via Playwright."""
    readability_src = httpx.get(
        "https://cdn.jsdelivr.net/npm/@mozilla/readability@0.5.0/Readability.min.js",
        verify=False, timeout=15
    ).text

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--disable-gpu"])
        page = browser.new_page()
        page.route("**/*", lambda r: r.abort()
                    if r.request.resource_type in ["image", "font", "media"]
                    else r.continue_())
        page.goto(url, wait_until="domcontentloaded", timeout=30000)
        page.evaluate(readability_src)
        result = page.evaluate("""() => {
            try {
                const doc = document.cloneNode(true);
                const reader = new Readability(doc);
                const article = reader.parse();
                return article ? article.textContent : document.body.innerText;
            } catch(e) {
                return document.body.innerText;
            }
        }""")
        browser.close()
    return result


def fetch_playwright_innertext(url: str) -> str:
    """Raw innerText via Playwright."""
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--disable-gpu", "--blink-settings=imagesEnabled=false"])
        page = browser.new_page()
        page.route("**/*", lambda r: r.abort()
                    if r.request.resource_type in ["stylesheet", "image", "font", "media"]
                    else r.continue_())
        page.goto(url, wait_until="domcontentloaded", timeout=30000)
        content = page.evaluate("() => document.body.innerText")
        browser.close()
    return content


def main():
    summary = {}

    for name, url in URLS.items():
        print(f"\n{'='*60}")
        print(f"  Collecting: {name} ({url})")
        print(f"{'='*60}")

        site_dir = OUT / name
        site_dir.mkdir(parents=True, exist_ok=True)

        # 1. Raw HTML
        print("  Fetching raw HTML...")
        raw_html = fetch_raw_html(url)
        (site_dir / "raw.html").write_text(raw_html, encoding="utf-8")

        # 2. Our scraper
        print("  Running our scraper distiller...")
        scraper = fetch_our_scraper(url)
        (site_dir / "our_scraper.md").write_text(scraper["content"], encoding="utf-8")

        # 3. Our lol_html
        print("  Running our lol_html distiller...")
        lol = fetch_our_lol_html(url)
        (site_dir / "our_lol_html.md").write_text(lol["content"], encoding="utf-8")

        # 4. Trafilatura
        print("  Running trafilatura...")
        traf = fetch_trafilatura(raw_html)
        (site_dir / "trafilatura.md").write_text(traf, encoding="utf-8")

        # 5. Readability.js
        print("  Running Readability.js...")
        try:
            readability = fetch_readability_js(url)
            (site_dir / "readability_js.txt").write_text(readability, encoding="utf-8")
        except Exception as e:
            readability = f"ERROR: {e}"
            (site_dir / "readability_js.txt").write_text(readability, encoding="utf-8")

        # 6. Playwright innerText
        print("  Running Playwright innerText...")
        innertext = fetch_playwright_innertext(url)
        (site_dir / "playwright_innertext.txt").write_text(innertext, encoding="utf-8")

        # Summary
        summary[name] = {
            "url": url,
            "raw_html_chars": len(raw_html),
            "our_scraper_chars": scraper["content_length"],
            "our_lol_html_chars": lol["content_length"],
            "trafilatura_chars": len(traf),
            "readability_js_chars": len(readability),
            "playwright_innertext_chars": len(innertext),
        }

        print(f"  raw={len(raw_html):,} scraper={scraper['content_length']:,} lol={lol['content_length']:,} traf={len(traf):,} rdbl={len(readability):,} inner={len(innertext):,}")

    # Save summary
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")

    # Print summary table
    print(f"\n{'='*100}")
    print(f"  SUMMARY")
    print(f"{'='*100}")
    print(f"  {'Site':<15s}  {'Raw HTML':>10s}  {'Scraper':>10s}  {'lol_html':>10s}  {'Trafil':>10s}  {'Readab':>10s}  {'innerTxt':>10s}")
    print(f"  {'─'*15}  {'─'*10}  {'─'*10}  {'─'*10}  {'─'*10}  {'─'*10}  {'─'*10}")
    for name, s in summary.items():
        print(f"  {name:<15s}  {s['raw_html_chars']:>10,}  {s['our_scraper_chars']:>10,}  {s['our_lol_html_chars']:>10,}  {s['trafilatura_chars']:>10,}  {s['readability_js_chars']:>10,}  {s['playwright_innertext_chars']:>10,}")

    print(f"\n  Files saved to: {OUT.absolute()}")


if __name__ == "__main__":
    main()
