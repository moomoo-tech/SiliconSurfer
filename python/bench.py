"""Benchmark: Agent Browser (Rust) vs Python requests vs httpx."""

import time
import httpx
from agent_browser import AgentBrowser

URLS = [
    "https://news.ycombinator.com/",
    "https://example.com",
    "https://httpbin.org/html",
    "https://en.wikipedia.org/wiki/Rust_(programming_language)",
    "https://www.python.org/",
]

ROUNDS = 3


def bench_requests(urls: list[str]) -> list[float]:
    """Pure Python httpx (no distill)."""
    import httpx
    times = []
    client = httpx.Client(timeout=30, verify=False)
    for url in urls:
        t0 = time.perf_counter()
        resp = client.get(url)
        _ = resp.text
        t1 = time.perf_counter()
        times.append(t1 - t0)
    client.close()
    return times


def bench_agent_browser(browser: AgentBrowser, urls: list[str]) -> list[float]:
    """Rust worker: fetch + distill to markdown."""
    times = []
    for url in urls:
        t0 = time.perf_counter()
        result = browser.fetch(url)
        t1 = time.perf_counter()
        times.append(t1 - t0)
    return times


async def bench_agent_browser_concurrent(browser: AgentBrowser, urls: list[str]) -> float:
    """Rust worker: fetch all URLs concurrently."""
    t0 = time.perf_counter()
    results = await browser.fetch_many_async(urls)
    t1 = time.perf_counter()
    return t1 - t0, results


def main():
    print(f"Benchmark: {len(URLS)} URLs x {ROUNDS} rounds\n")
    print("URLs:")
    for u in URLS:
        print(f"  {u}")
    print()

    # --- httpx (raw, no distill) ---
    print("=" * 60)
    print("httpx (Python, raw HTML, no distill)")
    print("=" * 60)
    httpx_times_all = []
    for r in range(ROUNDS):
        times = bench_requests(URLS)
        httpx_times_all.append(sum(times))
        if r == 0:
            for url, t in zip(URLS, times):
                print(f"  {t*1000:7.1f}ms  {url}")
    httpx_avg = sum(httpx_times_all) / ROUNDS
    print(f"  Avg total (sequential): {httpx_avg*1000:.0f}ms\n")

    # --- Agent Browser (sequential) ---
    print("=" * 60)
    print("Agent Browser (Rust, fetch + distill to markdown)")
    print("=" * 60)
    with AgentBrowser(port=9879) as browser:
        ab_times_all = []
        raw_sizes = []
        clean_sizes = []
        for r in range(ROUNDS):
            times = bench_agent_browser(browser, URLS)
            ab_times_all.append(sum(times))
            if r == 0:
                for url, t in zip(URLS, times):
                    result = browser.fetch(url)
                    clean_sizes.append(result["content_length"])
                    print(f"  {t*1000:7.1f}ms  {url}  → {result['content_length']} chars md")
        ab_avg = sum(ab_times_all) / ROUNDS
        print(f"  Avg total (sequential): {ab_avg*1000:.0f}ms\n")

        # --- Agent Browser (concurrent) ---
        print("=" * 60)
        print("Agent Browser (Rust, concurrent fetch)")
        print("=" * 60)
        import asyncio
        concurrent_times = []
        for r in range(ROUNDS):
            elapsed, results = asyncio.run(bench_agent_browser_concurrent(browser, URLS))
            concurrent_times.append(elapsed)
            if r == 0:
                for res in results:
                    if "error" not in res:
                        print(f"  {res['url']}  → {res['content_length']} chars md")
        concurrent_avg = sum(concurrent_times) / ROUNDS
        print(f"  Avg total (concurrent): {concurrent_avg*1000:.0f}ms\n")

    # --- Comparison ---
    print("=" * 60)
    print("COMPARISON")
    print("=" * 60)

    # Get raw HTML sizes for comparison
    client = httpx.Client(timeout=30, verify=False)
    raw_total = 0
    clean_total = sum(clean_sizes)
    for url in URLS:
        resp = client.get(url)
        raw_total += len(resp.text)
    client.close()

    print(f"  httpx sequential:          {httpx_avg*1000:7.0f}ms  (raw HTML, no cleaning)")
    print(f"  Agent Browser sequential:  {ab_avg*1000:7.0f}ms  (fetch + distill)")
    print(f"  Agent Browser concurrent:  {concurrent_avg*1000:7.0f}ms  (fetch + distill)")
    print()
    print(f"  Raw HTML total:     {raw_total:>8,} chars")
    print(f"  Clean MD total:     {clean_total:>8,} chars")
    print(f"  Compression ratio:  {raw_total/max(clean_total,1):.1f}x")
    print()
    if concurrent_avg > 0:
        print(f"  Concurrent speedup vs httpx seq: {httpx_avg/concurrent_avg:.1f}x")


if __name__ == "__main__":
    main()
