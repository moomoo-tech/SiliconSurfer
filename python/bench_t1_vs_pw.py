"""T1 (our headless Chrome) vs Playwright — head to head"""

import time
import httpx
from playwright.sync_api import sync_playwright

URLS = [
    ("HN", "https://news.ycombinator.com/"),
    ("Example", "https://example.com"),
    ("Wikipedia", "https://en.wikipedia.org/wiki/Rust_(programming_language)"),
    ("Python.org", "https://www.python.org/"),
]

ROUNDS = 3
SERVER = "http://localhost:9883"


def bench_t1(urls):
    client = httpx.Client(timeout=120)
    # warmup
    client.post(f"{SERVER}/fetch", json={"url": "https://example.com", "mode": "t1"})

    results = {}
    for name, url in urls:
        times = []
        for _ in range(ROUNDS):
            t = time.perf_counter()
            r = client.post(f"{SERVER}/fetch", json={"url": url, "mode": "t1"})
            times.append((time.perf_counter() - t) * 1000)
        d = r.json()
        results[name] = {"ms": sum(times) / ROUNDS, "chars": d.get("content_length", 0),
                         "preview": d.get("content", "")[:200]}
    client.close()
    return results


def bench_playwright(urls):
    startup_t = time.perf_counter()
    results = {}
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True,
            args=["--disable-gpu", "--disable-software-rasterizer",
                  "--disable-dev-shm-usage", "--disable-extensions",
                  "--mute-audio", "--blink-settings=imagesEnabled=false"])
        startup_ms = (time.perf_counter() - startup_t) * 1000

        ctx = browser.new_context()
        page = ctx.new_page()
        page.route("**/*", lambda r: r.abort()
                   if r.request.resource_type in ["stylesheet", "image", "font", "media"]
                   else r.continue_())

        for name, url in urls:
            times = []
            for _ in range(ROUNDS):
                t = time.perf_counter()
                page.goto(url, wait_until="domcontentloaded", timeout=30000)
                content = page.evaluate("""() => {
                    ['nav','footer','header','script','style','iframe','noscript'].forEach(t=>
                        document.querySelectorAll(t).forEach(e=>e.remove()));
                    let m = document.querySelector('article')
                        || document.querySelector('main')
                        || document.body;
                    return m ? m.innerText : '';
                }""")
                times.append((time.perf_counter() - t) * 1000)
            results[name] = {"ms": sum(times) / ROUNDS, "chars": len(content),
                             "preview": content[:200]}

        browser.close()
    return results, startup_ms


def main():
    print("Running T1...")
    t1 = bench_t1(URLS)

    print("Running Playwright...")
    pw, pw_startup = bench_playwright(URLS)

    print()
    print("=" * 85)
    print(f"  T1 (our Chrome daemon + Rust distiller) vs Playwright")
    print(f"  Playwright startup: {pw_startup:.0f}ms | T1 startup: 0ms (daemon)")
    print("=" * 85)
    print()

    # Speed table
    print(f"  {'Page':<15s}  {'T1':>8s}  {'PW':>8s}  {'winner':>12s}  {'T1 chars':>10s}  {'PW chars':>10s}")
    print(f"  {'─'*15}  {'─'*8}  {'─'*8}  {'─'*12}  {'─'*10}  {'─'*10}")

    t1_total, pw_total = 0, 0
    for name, _ in URLS:
        t = t1[name]
        p = pw[name]
        t1_total += t["ms"]
        pw_total += p["ms"]
        if t["ms"] < p["ms"]:
            winner = f"T1 {p['ms']/t['ms']:.1f}x"
        else:
            winner = f"PW {t['ms']/p['ms']:.1f}x"
        print(f"  {name:<15s}  {t['ms']:>6.0f}ms  {p['ms']:>6.0f}ms  {winner:>12s}  {t['chars']:>10,}  {p['chars']:>10,}")

    print(f"  {'─'*15}  {'─'*8}  {'─'*8}  {'─'*12}  {'─'*10}  {'─'*10}")
    print(f"  {'TOTAL':<15s}  {t1_total:>6.0f}ms  {pw_total:>6.0f}ms")
    pw_total_startup = pw_total + pw_startup
    print(f"  {'TOTAL+startup':<15s}          {pw_total_startup:>6.0f}ms")

    print()
    print(f"  Summary:")
    print(f"    T1 vs PW (fetch only):     {'T1' if t1_total < pw_total else 'PW'} is {max(t1_total,pw_total)/min(t1_total,pw_total):.1f}x faster")
    print(f"    T1 vs PW (incl startup):   T1 is {pw_total_startup/t1_total:.1f}x faster")
    print()

    # Content comparison
    print("  Content quality:")
    print(f"    T1: Rust distiller → Markdown with links, headings, structure")
    print(f"    PW: JS innerText → plain text, no links, no structure")
    print()
    for name, _ in URLS:
        t1_more = t1[name]["chars"] > pw[name]["chars"]
        diff = abs(t1[name]["chars"] - pw[name]["chars"])
        who = "T1" if t1_more else "PW"
        print(f"    {name:<15s}  T1={t1[name]['chars']:>8,}  PW={pw[name]['chars']:>8,}  ({who} +{diff:,})")


if __name__ == "__main__":
    main()
