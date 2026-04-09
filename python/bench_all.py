"""Benchmark: T0 (reqwest) vs T1 (our headless Chrome) vs Playwright"""

import time
import httpx
from playwright.sync_api import sync_playwright

URLS = [
    "https://news.ycombinator.com/",
    "https://example.com",
    "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    "https://www.python.org/",
    "https://html.duckduckgo.com/html/?q=best+4tb+ssd+2026",
]

ROUNDS = 3
SERVER = "http://localhost:9881"


def bench_t0(urls):
    """Our T0: reqwest + Rust distiller"""
    client = httpx.Client(timeout=60)
    results = []
    for url in urls:
        t0 = time.perf_counter()
        resp = client.post(f"{SERVER}/fetch", json={"url": url, "mode": "t0"})
        elapsed = time.perf_counter() - t0
        data = resp.json()
        results.append({
            "url": url,
            "ms": elapsed * 1000,
            "chars": data["content_length"],
        })
    client.close()
    return results


def bench_t1(urls):
    """Our T1: Rust headless Chrome + Rust distiller"""
    client = httpx.Client(timeout=60)
    results = []
    for url in urls:
        t0 = time.perf_counter()
        resp = client.post(f"{SERVER}/fetch", json={"url": url, "mode": "t1"})
        elapsed = time.perf_counter() - t0
        data = resp.json()
        results.append({
            "url": url,
            "ms": elapsed * 1000,
            "chars": data["content_length"],
        })
    client.close()
    return results


def bench_playwright(urls):
    """Playwright: full headless Chrome (block CSS/img/font) + JS extract"""
    results = []
    with sync_playwright() as p:
        browser = p.chromium.launch(
            headless=True,
            args=[
                "--disable-gpu",
                "--disable-software-rasterizer",
                "--disable-dev-shm-usage",
                "--disable-extensions",
                "--mute-audio",
                "--blink-settings=imagesEnabled=false",
            ],
        )
        context = browser.new_context()
        page = context.new_page()

        def block_resources(route):
            if route.request.resource_type in ["stylesheet", "image", "font", "media"]:
                route.abort()
            else:
                route.continue_()

        page.route("**/*", block_resources)

        for url in urls:
            t0 = time.perf_counter()
            page.goto(url, wait_until="domcontentloaded")
            content = page.evaluate("""
                () => {
                    ['nav','footer','header','script','style','iframe','noscript'].forEach(tag => {
                        document.querySelectorAll(tag).forEach(el => el.remove());
                    });
                    let main = document.querySelector('article')
                        || document.querySelector('main')
                        || document.querySelector('[role="main"]')
                        || document.body;
                    return main ? main.innerText : '';
                }
            """)
            elapsed = time.perf_counter() - t0
            results.append({
                "url": url,
                "ms": elapsed * 1000,
                "chars": len(content),
            })

        browser.close()
    return results


def shorten(url, maxlen=45):
    return url[:maxlen] + "..." if len(url) > maxlen else url


def main():
    print("=" * 90)
    print("  T0 (reqwest)  vs  T1 (our Chrome)  vs  Playwright")
    print(f"  {len(URLS)} URLs x {ROUNDS} rounds")
    print("=" * 90)

    # Warm up server
    httpx.Client(timeout=10).post(f"{SERVER}/fetch", json={"url": "https://example.com", "mode": "t0"}).close

    all_t0, all_t1, all_pw = [], [], []

    for r in range(ROUNDS):
        t0_r = bench_t0(URLS)
        t1_r = bench_t1(URLS)
        pw_r = bench_playwright(URLS)
        all_t0.append(t0_r)
        all_t1.append(t1_r)
        all_pw.append(pw_r)

    # Average per-URL
    def avg_results(all_rounds):
        avgs = []
        for i in range(len(URLS)):
            ms = sum(all_rounds[r][i]["ms"] for r in range(ROUNDS)) / ROUNDS
            chars = all_rounds[0][i]["chars"]
            avgs.append({"url": URLS[i], "ms": ms, "chars": chars})
        return avgs

    t0_avg = avg_results(all_t0)
    t1_avg = avg_results(all_t1)
    pw_avg = avg_results(all_pw)

    # Per-URL table
    print()
    print(f"  {'URL':<47s}  {'T0':>8s}  {'T1':>8s}  {'PW':>8s}  {'T0 ch':>8s}  {'T1 ch':>8s}  {'PW ch':>8s}")
    print(f"  {'─'*47}  {'─'*8}  {'─'*8}  {'─'*8}  {'─'*8}  {'─'*8}  {'─'*8}")
    for i in range(len(URLS)):
        url = shorten(URLS[i])
        print(f"  {url:<47s}  {t0_avg[i]['ms']:>6.0f}ms  {t1_avg[i]['ms']:>6.0f}ms  {pw_avg[i]['ms']:>6.0f}ms  {t0_avg[i]['chars']:>7,}  {t1_avg[i]['chars']:>7,}  {pw_avg[i]['chars']:>7,}")

    # Totals
    t0_total = sum(x["ms"] for x in t0_avg)
    t1_total = sum(x["ms"] for x in t1_avg)
    pw_total = sum(x["ms"] for x in pw_avg)

    print(f"  {'─'*47}  {'─'*8}  {'─'*8}  {'─'*8}")
    print(f"  {'TOTAL':<47s}  {t0_total:>6.0f}ms  {t1_total:>6.0f}ms  {pw_total:>6.0f}ms")

    print()
    print("=" * 90)
    print("  SUMMARY")
    print("=" * 90)
    print(f"  {'Metric':<40s}  {'T0 reqwest':>12s}  {'T1 our Chrome':>14s}  {'Playwright':>12s}")
    print(f"  {'─'*40}  {'─'*12}  {'─'*14}  {'─'*12}")
    print(f"  {'Total time (sequential)':40s}  {t0_total:>10.0f}ms  {t1_total:>12.0f}ms  {pw_total:>10.0f}ms")
    print(f"  {'Browser startup':40s}  {'0ms':>12s}  {'shared':>14s}  {'~1800ms':>12s}")
    print(f"  {'Distiller':40s}  {'Rust':>12s}  {'Rust':>14s}  {'JS inject':>12s}")
    print(f"  {'Memory per session':40s}  {'~1 MB':>12s}  {'~30 MB':>14s}  {'~50 MB':>12s}")
    print(f"  {'JS rendering':40s}  {'No':>12s}  {'Yes':>14s}  {'Yes':>12s}")

    print()
    fastest = min(t0_total, t1_total, pw_total)
    print(f"  T0 vs Playwright:     T0 is {pw_total/t0_total:.1f}x faster")
    print(f"  T1 vs Playwright:     T1 is {pw_total/t1_total:.1f}x faster")
    print(f"  T0 vs T1:             T0 is {t1_total/t0_total:.1f}x faster")


if __name__ == "__main__":
    main()
