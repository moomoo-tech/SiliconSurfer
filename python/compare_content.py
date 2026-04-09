"""Compare actual content: T0 vs Playwright on HN"""

import httpx
from playwright.sync_api import sync_playwright

URL = "https://news.ycombinator.com/"
SERVER = "http://localhost:9881"

# T0
resp = httpx.post(f"{SERVER}/fetch", json={"url": URL, "mode": "t0"}, timeout=30)
t0_content = resp.json()["content"]

# Playwright
with sync_playwright() as p:
    browser = p.chromium.launch(headless=True, args=["--blink-settings=imagesEnabled=false"])
    page = browser.new_page()
    page.route("**/*", lambda r: r.abort() if r.request.resource_type in ["stylesheet","image","font","media"] else r.continue_())
    page.goto(URL, wait_until="domcontentloaded")
    pw_content = page.evaluate("""() => {
        ['nav','footer','header','script','style','iframe','noscript'].forEach(t=>document.querySelectorAll(t).forEach(e=>e.remove()));
        let m = document.querySelector('article') || document.querySelector('main') || document.body;
        return m ? m.innerText : '';
    }""")
    browser.close()

print(f"T0: {len(t0_content)} chars")
print(f"PW: {len(pw_content)} chars")
print()

print("=" * 70)
print("T0 FIRST 2000 CHARS:")
print("=" * 70)
print(t0_content[:2000])
print()

print("=" * 70)
print("PLAYWRIGHT FIRST 2000 CHARS:")
print("=" * 70)
print(pw_content[:2000])
print()

# Check what's missing
t0_lines = set(t0_content.split('\n'))
pw_lines = set(pw_content.split('\n'))
only_t0 = [l.strip() for l in t0_lines - pw_lines if l.strip() and len(l.strip()) > 20]
only_pw = [l.strip() for l in pw_lines - t0_lines if l.strip() and len(l.strip()) > 20]

print("=" * 70)
print(f"Lines ONLY in T0 (sample, {len(only_t0)} total):")
print("=" * 70)
for l in sorted(only_t0)[:15]:
    print(f"  {l[:100]}")

print()
print("=" * 70)
print(f"Lines ONLY in Playwright (sample, {len(only_pw)} total):")
print("=" * 70)
for l in sorted(only_pw)[:15]:
    print(f"  {l[:100]}")
