"""Collect Operator mode samples from test-friendly sites."""

import json
import httpx
from pathlib import Path

SERVER = "http://localhost:9883"
OUT = Path("eval/samples")

# Test sites with forms, buttons, UI elements — won't block us
OPERATOR_URLS = {
    "toscrape_login": "https://quotes.toscrape.com/login",
    "toscrape_quotes": "https://quotes.toscrape.com/",
    "books_toscrape": "https://books.toscrape.com/",
    "httpbin_forms": "https://httpbin.org/forms/post",
    "herokuapp": "https://the-internet.herokuapp.com/",
}


def fetch_mode(url: str, mode: str = "t0", fast: bool = True) -> dict:
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "mode": mode, "fast": fast}, timeout=60)
    return r.json()


def main():
    c = httpx.Client(timeout=60)

    for name, url in OPERATOR_URLS.items():
        print(f"\n{'='*60}")
        print(f"  {name} ({url})")
        print(f"{'='*60}")

        site_dir = OUT / name
        site_dir.mkdir(parents=True, exist_ok=True)

        # Reader mode (LLM-Friendly)
        print("  Fetching Reader mode...")
        reader = c.post(f"{SERVER}/fetch", json={"url": url, "mode": "t0", "fast": True}).json()
        (site_dir / "reader.md").write_text(reader.get("content", ""), encoding="utf-8")
        print(f"    Reader: {reader.get('content_length', 0):,} chars")

        # Scraper Reader
        print("  Fetching Scraper mode...")
        scraper = c.post(f"{SERVER}/fetch", json={"url": url, "mode": "t0", "fast": False}).json()
        (site_dir / "scraper.md").write_text(scraper.get("content", ""), encoding="utf-8")
        print(f"    Scraper: {scraper.get('content_length', 0):,} chars")

        # Raw HTML
        print("  Fetching raw HTML...")
        raw = httpx.get(url, timeout=30, follow_redirects=True, verify=False).text
        (site_dir / "raw.html").write_text(raw, encoding="utf-8")
        print(f"    Raw HTML: {len(raw):,} chars")

    c.close()
    print("\nDone. Check eval/samples/ for operator test sites.")


if __name__ == "__main__":
    main()
