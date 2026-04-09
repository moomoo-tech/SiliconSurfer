"""Final Benchmark: T0 vs T1 vs Playwright — with detailed output"""

import time
from playwright.sync_api import sync_playwright
import httpx

URLS = [
    "https://news.ycombinator.com/",
    "https://example.com",
    "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    "https://www.python.org/",
]

SERVER = "http://localhost:9883"


def run_t0(urls):
    client = httpx.Client(timeout=60)
    results = []
    for url in urls:
        t = time.perf_counter()
        resp = client.post(f"{SERVER}/fetch", json={"url": url, "mode": "t0"})
        ms = (time.perf_counter() - t) * 1000
        data = resp.json()
        results.append({"url": url, "ms": ms, "chars": data["content_length"], "title": data.get("title",""), "error": None})
    client.close()
    return results


def run_t1(urls):
    client = httpx.Client(timeout=60)
    results = []
    for url in urls:
        t = time.perf_counter()
        resp = client.post(f"{SERVER}/fetch", json={"url": url, "mode": "t1"})
        ms = (time.perf_counter() - t) * 1000
        data = resp.json()
        err = data.get("error")
        results.append({"url": url, "ms": ms, "chars": data.get("content_length", 0), "title": data.get("title",""), "error": err})
    client.close()
    return results


def run_playwright(urls):
    results = []
    startup_t = time.perf_counter()
    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            args=["--disable-gpu", "--disable-software-rasterizer",
                  "--disable-dev-shm-usage", "--disable-extensions",
                  "--mute-audio", "--blink-settings=imagesEnabled=false"],
        )
        startup_ms = (time.perf_counter() - startup_t) * 1000
        print(f"  Playwright startup: {startup_ms:.0f}ms")

        ctx = browser.new_context()
        page = ctx.new_page()
        page.route("**/*", lambda route: route.abort() if route.request.resource_type in ["stylesheet","image","font","media"] else route.continue_())

        for url in urls:
            err = None
            chars = 0
            title = ""
            t = time.perf_counter()
            try:
                page.goto(url, wait_until="domcontentloaded", timeout=30000)
                title = page.title()
                content = page.evaluate("""() => {
                    ['nav','footer','header','script','style','iframe','noscript'].forEach(t=>document.querySelectorAll(t).forEach(e=>e.remove()));
                    let m = document.querySelector('article') || document.querySelector('main') || document.body;
                    return m ? m.innerText : '';
                }""")
                chars = len(content)
            except Exception as e:
                err = str(e)
            ms = (time.perf_counter() - t) * 1000
            results.append({"url": url, "ms": ms, "chars": chars, "title": title, "error": err})

        browser.close()
    return results, startup_ms


def short(url, n=50):
    return url[:n] + "..." if len(url) > n else url


def main():
    print("=" * 95)
    print("  BENCHMARK: T0 (reqwest) vs T1 (our Chrome) vs Playwright")
    print("=" * 95)
    print()

    # --- T0 ---
    print("  [T0] reqwest + Rust distiller")
    t0 = run_t0(URLS)
    for r in t0:
        status = f"{r['chars']:>7,} chars" if not r["error"] else f"ERROR: {r['error'][:40]}"
        print(f"    {r['ms']:>7.0f}ms  {status:>20s}  {short(r['url'])}")
    t0_total = sum(x["ms"] for x in t0)
    print(f"    TOTAL: {t0_total:.0f}ms\n")

    # --- T1 ---
    print("  [T1] Our headless Chrome (CDP) + Rust distiller")
    t1 = run_t1(URLS)
    for r in t1:
        status = f"{r['chars']:>7,} chars" if not r["error"] else f"ERROR: {r['error'][:40]}"
        print(f"    {r['ms']:>7.0f}ms  {status:>20s}  {short(r['url'])}")
    t1_total = sum(x["ms"] for x in t1)
    print(f"    TOTAL: {t1_total:.0f}ms\n")

    # --- Playwright ---
    print("  [PW] Playwright headless Chrome + JS extract")
    pw, pw_startup = run_playwright(URLS)
    for r in pw:
        status = f"{r['chars']:>7,} chars" if not r["error"] else f"ERROR: {r['error'][:40]}"
        print(f"    {r['ms']:>7.0f}ms  {status:>20s}  {short(r['url'])}")
    pw_total = sum(x["ms"] for x in pw)
    pw_total_with_startup = pw_total + pw_startup
    print(f"    TOTAL: {pw_total:.0f}ms (+ {pw_startup:.0f}ms startup = {pw_total_with_startup:.0f}ms)\n")

    # --- Final comparison ---
    print("=" * 95)
    print("  FINAL COMPARISON")
    print("=" * 95)
    print()
    print(f"  {'':50s}  {'T0':>10s}  {'T1':>10s}  {'Playwright':>12s}")
    print(f"  {'─'*50}  {'─'*10}  {'─'*10}  {'─'*12}")
    for i in range(len(URLS)):
        url = short(URLS[i])
        t0_s = f"{t0[i]['ms']:.0f}ms" if not t0[i]["error"] else "ERR"
        t1_s = f"{t1[i]['ms']:.0f}ms" if not t1[i]["error"] else "ERR"
        pw_s = f"{pw[i]['ms']:.0f}ms" if not pw[i]["error"] else "ERR"
        print(f"  {url:50s}  {t0_s:>10s}  {t1_s:>10s}  {pw_s:>12s}")

    print(f"  {'─'*50}  {'─'*10}  {'─'*10}  {'─'*12}")
    print(f"  {'Total (fetch only)':50s}  {t0_total:>8.0f}ms  {t1_total:>8.0f}ms  {pw_total:>10.0f}ms")
    print(f"  {'Total (incl startup)':50s}  {t0_total:>8.0f}ms  {t1_total:>8.0f}ms  {pw_total_with_startup:>10.0f}ms")
    print()

    print(f"  {'Content extracted (chars):':50s}")
    for i in range(len(URLS)):
        url = short(URLS[i])
        print(f"    {url:48s}  {t0[i]['chars']:>8,}  {t1[i]['chars']:>8,}  {pw[i]['chars']:>10,}")

    print()
    print(f"  T0 vs Playwright (incl startup):  T0 is {pw_total_with_startup/t0_total:.1f}x faster")
    print(f"  T1 vs Playwright (incl startup):  T1 is {pw_total_with_startup/t1_total:.1f}x faster")
    print(f"  T0 vs T1:                         T0 is {t1_total/t0_total:.1f}x faster")


if __name__ == "__main__":
    main()
