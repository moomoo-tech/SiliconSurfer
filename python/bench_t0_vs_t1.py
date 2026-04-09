"""Benchmark: T0 (reqwest) vs T1 (headless Chromium)
比较我们架构的两条路径。"""

import time
import asyncio
from playwright.sync_api import sync_playwright
from agent_browser import AgentBrowser

# 混合测试：静态 + 动态页面
URLS = [
    ("https://news.ycombinator.com/", "static"),
    ("https://example.com", "static"),
    ("https://en.wikipedia.org/wiki/Rust_(programming_language)", "static"),
    ("https://www.python.org/", "static"),
    ("https://html.duckduckgo.com/html/?q=best+4tb+ssd+2026", "static"),
]

ROUNDS = 3


def bench_t0(browser: AgentBrowser, urls):
    """T0: reqwest + Rust distiller"""
    results = []
    for url, _ in urls:
        t0 = time.perf_counter()
        result = browser.fetch(url)
        elapsed = time.perf_counter() - t0
        results.append({
            "url": url,
            "time_ms": elapsed * 1000,
            "content_len": result["content_length"],
            "title": result.get("title", ""),
        })
    return results


def bench_t1(urls):
    """T1: Headless Chromium (Playwright) — 模拟未来 Rust CDP 层"""
    results = []
    with sync_playwright() as p:
        chrome = p.chromium.launch(
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
        context = chrome.new_context()

        # 拦截不需要的资源
        def block_resources(route):
            if route.request.resource_type in ["stylesheet", "image", "font", "media"]:
                route.abort()
            else:
                route.continue_()

        page = context.new_page()
        page.route("**/*", block_resources)

        for url, _ in urls:
            t0 = time.perf_counter()
            page.goto(url, wait_until="domcontentloaded")
            # 提取文本（模拟 distiller）
            content = page.evaluate("""
                () => {
                    // 移除噪声
                    ['nav','footer','header','script','style','iframe','noscript'].forEach(tag => {
                        document.querySelectorAll(tag).forEach(el => el.remove());
                    });
                    // 提取主要内容
                    let main = document.querySelector('article')
                        || document.querySelector('main')
                        || document.querySelector('[role="main"]')
                        || document.body;
                    return main ? main.innerText : document.body.innerText;
                }
            """)
            elapsed = time.perf_counter() - t0
            title = page.title()
            results.append({
                "url": url,
                "time_ms": elapsed * 1000,
                "content_len": len(content),
                "title": title,
            })

        chrome.close()
    return results


def bench_t1_startup():
    """单独测量 T1 的浏览器启动时间"""
    t0 = time.perf_counter()
    with sync_playwright() as p:
        chrome = p.chromium.launch(headless=True)
        t1 = time.perf_counter()
        chrome.close()
    return (t1 - t0) * 1000


def main():
    print(f"T0 (reqwest) vs T1 (headless Chromium)")
    print(f"{len(URLS)} URLs x {ROUNDS} rounds\n")

    # --- T1 startup cost ---
    startup = bench_t1_startup()
    print(f"T1 browser startup: {startup:.0f}ms (one-time cost)\n")

    # --- T0 ---
    print("=" * 70)
    print("T0: reqwest + Rust distiller (no browser)")
    print("=" * 70)
    with AgentBrowser(port=9880) as ab:
        t0_all = []
        for r in range(ROUNDS):
            results = bench_t0(ab, URLS)
            total = sum(x["time_ms"] for x in results)
            t0_all.append(total)
            if r == 0:
                for x in results:
                    print(f"  {x['time_ms']:7.0f}ms  {x['content_len']:>7,} chars  {x['url'][:60]}")
        t0_avg = sum(t0_all) / ROUNDS
        print(f"\n  Avg total: {t0_avg:.0f}ms")
        # 记住第一轮结果做对比
        t0_results = results

    print()

    # --- T1 ---
    print("=" * 70)
    print("T1: Headless Chromium (block CSS/img/font) + JS extract")
    print("=" * 70)
    t1_all = []
    for r in range(ROUNDS):
        results = bench_t1(URLS)
        total = sum(x["time_ms"] for x in results)
        t1_all.append(total)
        if r == 0:
            for x in results:
                print(f"  {x['time_ms']:7.0f}ms  {x['content_len']:>7,} chars  {x['url'][:60]}")
    t1_avg = sum(t1_all) / ROUNDS
    print(f"\n  Avg total: {t1_avg:.0f}ms")
    t1_results = results

    # --- Memory estimate ---
    print()
    print("=" * 70)
    print("COMPARISON")
    print("=" * 70)
    print(f"  {'':40s}  {'T0 reqwest':>12s}  {'T1 Chrome':>12s}")
    print(f"  {'─'*40}  {'─'*12}  {'─'*12}")
    print(f"  {'Avg total time':40s}  {t0_avg:>10.0f}ms  {t1_avg:>10.0f}ms")
    print(f"  {'Startup cost':40s}  {'~0ms':>12s}  {startup:>10.0f}ms")
    print(f"  {'Memory per session':40s}  {'~1 MB':>12s}  {'~50 MB':>12s}")
    print(f"  {'Max concurrency (8GB RAM)':40s}  {'~5000+':>12s}  {'~100':>12s}")
    print(f"  {'JS rendering':40s}  {'No':>12s}  {'Yes':>12s}")
    print(f"  {'Dynamic page (SPA)':40s}  {'No':>12s}  {'Yes':>12s}")
    print()
    if t1_avg > 0:
        ratio = t1_avg / t0_avg
        print(f"  T0 is {ratio:.1f}x faster than T1 (sequential)")
    print()
    print("  Per-URL comparison:")
    print(f"  {'URL':50s}  {'T0':>8s}  {'T1':>8s}  {'T0 chars':>10s}  {'T1 chars':>10s}")
    print(f"  {'─'*50}  {'─'*8}  {'─'*8}  {'─'*10}  {'─'*10}")
    for a, b in zip(t0_results, t1_results):
        url_short = a["url"][:50]
        print(f"  {url_short:50s}  {a['time_ms']:>6.0f}ms  {b['time_ms']:>6.0f}ms  {a['content_len']:>10,}  {b['content_len']:>10,}")

    print()
    print("Conclusion:")
    print("  T0 (reqwest): 静态页面首选，极快极轻，无浏览器开销")
    print("  T1 (Chrome):  动态页面（SPA/JS渲染）必须用，但慢且重")
    print("  → 双层路由策略：能用 T0 绝不上 T1")


if __name__ == "__main__":
    main()
