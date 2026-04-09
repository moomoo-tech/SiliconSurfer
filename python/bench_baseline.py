"""Baseline: Our Distiller vs Readability.js vs raw innerText
Compare content quality and speed across different page types."""

import time
import json
import httpx
from playwright.sync_api import sync_playwright

# Mix of page types
URLS = {
    "HN (list)":       "https://news.ycombinator.com/",
    "Example (tiny)":   "https://example.com",
    "Wikipedia (huge)": "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    "Python.org (mid)": "https://www.python.org/",
    "Blog post":        "https://aphyr.com/posts/411-the-future-of-everything-is-lies-i-guess",
}

# Readability.js CDN (inject into page)
READABILITY_JS_URL = "https://cdn.jsdelivr.net/npm/@mozilla/readability@0.5.0/Readability.min.js"


def our_distiller(urls: dict) -> dict:
    """Our T0: reqwest + Rust distiller"""
    client = httpx.Client(timeout=60)
    results = {}
    for name, url in urls.items():
        t = time.perf_counter()
        try:
            resp = client.post("http://localhost:9883/fetch",
                             json={"url": url, "mode": "t0"})
            data = resp.json()
            ms = (time.perf_counter() - t) * 1000
            results[name] = {
                "ms": ms,
                "chars": data.get("content_length", 0),
                "preview": data.get("content", "")[:300],
                "error": data.get("error"),
            }
        except Exception as e:
            ms = (time.perf_counter() - t) * 1000
            results[name] = {"ms": ms, "chars": 0, "preview": "", "error": str(e)}
    client.close()
    return results


def readability_js(urls: dict) -> dict:
    """Readability.js injected via Playwright"""
    results = {}
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True,
            args=["--disable-gpu", "--blink-settings=imagesEnabled=false"])
        page = browser.new_page()
        page.route("**/*", lambda r: r.abort()
                    if r.request.resource_type in ["image", "font", "media"]
                    else r.continue_())

        # Pre-fetch Readability.js source
        import httpx as _httpx
        readability_src = _httpx.get(READABILITY_JS_URL, verify=False, timeout=15).text

        for name, url in urls.items():
            t = time.perf_counter()
            try:
                page.goto(url, wait_until="domcontentloaded", timeout=30000)
                # Inject Readability.js directly (avoid CDN fetch inside page)
                page.evaluate(readability_src)
                # Extract with Readability
                result = page.evaluate("""
                    () => {
                        try {
                            const doc = document.cloneNode(true);
                            const reader = new Readability(doc);
                            const article = reader.parse();
                            if (article) {
                                return {content: article.textContent, title: article.title, ok: true};
                            }
                            return {content: document.body.innerText, title: document.title, ok: false};
                        } catch(e) {
                            return {content: document.body.innerText, title: document.title, ok: false};
                        }
                    }
                """)
                ms = (time.perf_counter() - t) * 1000
                content = result.get("content", "")
                results[name] = {
                    "ms": ms,
                    "chars": len(content),
                    "preview": content[:300],
                    "readability_ok": result.get("ok", False),
                    "error": None,
                }
            except Exception as e:
                ms = (time.perf_counter() - t) * 1000
                results[name] = {"ms": ms, "chars": 0, "preview": "", "error": str(e)}

        browser.close()
    return results


def raw_innertext(urls: dict) -> dict:
    """Plain innerText via Playwright (baseline)"""
    results = {}
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True,
            args=["--disable-gpu", "--blink-settings=imagesEnabled=false"])
        page = browser.new_page()
        page.route("**/*", lambda r: r.abort()
                    if r.request.resource_type in ["stylesheet", "image", "font", "media"]
                    else r.continue_())

        for name, url in urls.items():
            t = time.perf_counter()
            try:
                page.goto(url, wait_until="domcontentloaded", timeout=30000)
                content = page.evaluate("() => document.body.innerText")
                ms = (time.perf_counter() - t) * 1000
                results[name] = {
                    "ms": ms,
                    "chars": len(content),
                    "preview": content[:300],
                    "error": None,
                }
            except Exception as e:
                ms = (time.perf_counter() - t) * 1000
                results[name] = {"ms": ms, "chars": 0, "preview": "", "error": str(e)}

        browser.close()
    return results


def main():
    print("=" * 90)
    print("  BASELINE: Our Distiller vs Readability.js vs Raw innerText")
    print("=" * 90)
    print()

    print("  Running Our Distiller (T0)...")
    ours = our_distiller(URLS)

    print("  Running Readability.js...")
    readability = readability_js(URLS)

    print("  Running Raw innerText...")
    raw = raw_innertext(URLS)

    # --- Speed comparison ---
    print()
    print("  SPEED (ms)")
    print(f"  {'Page':<20s}  {'Ours':>8s}  {'Readability':>12s}  {'innerText':>10s}")
    print(f"  {'─'*20}  {'─'*8}  {'─'*12}  {'─'*10}")
    for name in URLS:
        o = ours.get(name, {})
        r = readability.get(name, {})
        i = raw.get(name, {})
        o_s = f"{o['ms']:.0f}ms" if not o.get("error") else "ERR"
        r_s = f"{r['ms']:.0f}ms" if not r.get("error") else "ERR"
        i_s = f"{i['ms']:.0f}ms" if not i.get("error") else "ERR"
        print(f"  {name:<20s}  {o_s:>8s}  {r_s:>12s}  {i_s:>10s}")

    # --- Content size ---
    print()
    print("  CONTENT SIZE (chars)")
    print(f"  {'Page':<20s}  {'Ours':>8s}  {'Readability':>12s}  {'innerText':>10s}")
    print(f"  {'─'*20}  {'─'*8}  {'─'*12}  {'─'*10}")
    for name in URLS:
        o = ours.get(name, {})
        r = readability.get(name, {})
        i = raw.get(name, {})
        print(f"  {name:<20s}  {o.get('chars',0):>8,}  {r.get('chars',0):>12,}  {i.get('chars',0):>10,}")

    # --- Content preview ---
    print()
    print("=" * 90)
    print("  CONTENT PREVIEW (first 200 chars)")
    print("=" * 90)
    for name in URLS:
        print(f"\n  --- {name} ---")
        o = ours.get(name, {})
        r = readability.get(name, {})
        i = raw.get(name, {})
        rdbl = " [Readability OK]" if r.get("readability_ok") else " [fallback innerText]"

        print(f"  [Ours]        {o.get('preview','ERR: '+str(o.get('error','')))[:200]}")
        print(f"  [Readability]{rdbl} {r.get('preview','ERR: '+str(r.get('error','')))[:200]}")
        print(f"  [innerText]   {i.get('preview','ERR: '+str(i.get('error','')))[:200]}")


if __name__ == "__main__":
    main()
