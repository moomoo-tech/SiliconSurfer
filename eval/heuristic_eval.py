"""Level 1: Heuristic evaluation of distiller output quality.

Metrics:
- Token compression ratio
- Link retention rate
- Noise keyword density
- Code block integrity
- Structure score (headings, lists)
"""

import json
import re
from pathlib import Path


def count_tokens(text: str) -> int:
    """Rough token count (~4 chars per token for English)."""
    return max(1, len(text) // 4)


def count_links(text: str) -> int:
    """Count markdown links [text](url)."""
    return len(re.findall(r'\[([^\]]+)\]\(([^)]+)\)', text))


def count_html_links(html: str) -> int:
    """Count <a href> in HTML."""
    return len(re.findall(r'<a\s[^>]*href=["\']([^"\']+)["\']', html))


def noise_keywords(text: str) -> dict:
    """Count UI noise keywords that shouldn't be in LLM-friendly output."""
    keywords = ['login', 'sign in', 'sign up', 'cookie', 'privacy policy',
                'terms of service', 'subscribe', 'advertisement', '| hide |',
                'skip to content', 'skip to main']
    found = {}
    text_lower = text.lower()
    for kw in keywords:
        count = text_lower.count(kw)
        if count > 0:
            found[kw] = count
    return found


def code_block_check(text: str) -> dict:
    """Check code block integrity."""
    fences = text.count('```')
    # Should be even (open + close)
    balanced = fences % 2 == 0
    # Check if code inside fences has preserved indentation
    blocks = re.findall(r'```\n?(.*?)```', text, re.DOTALL)
    has_indentation = any('\n  ' in b or '\n\t' in b for b in blocks)
    return {
        "fence_count": fences,
        "balanced": balanced,
        "block_count": fences // 2,
        "has_indentation": has_indentation,
    }


def structure_score(text: str) -> dict:
    """Evaluate markdown structural elements."""
    h1 = len(re.findall(r'^# ', text, re.MULTILINE))
    h2 = len(re.findall(r'^## ', text, re.MULTILINE))
    h3 = len(re.findall(r'^### ', text, re.MULTILINE))
    lists = len(re.findall(r'^- ', text, re.MULTILINE))
    bold = len(re.findall(r'\*\*[^*]+\*\*', text))
    italic = len(re.findall(r'_[^_]+_', text))
    return {
        "headings": h1 + h2 + h3,
        "h1": h1, "h2": h2, "h3": h3,
        "list_items": lists,
        "bold": bold,
        "italic": italic,
    }


def evaluate_sample(name: str, sample_dir: Path) -> dict:
    """Evaluate all distiller outputs for one sample."""
    raw_html = (sample_dir / "raw.html").read_text(errors="replace")

    tools = {
        "our_scraper": "our_scraper.md",
        "our_lol_html": "our_lol_html.md",
        "trafilatura": "trafilatura.md",
        "readability_js": "readability_js.txt",
        "playwright": "playwright_innertext.txt",
    }

    html_links = count_html_links(raw_html)
    html_tokens = count_tokens(raw_html)

    results = {"name": name, "html_chars": len(raw_html), "html_tokens": html_tokens, "html_links": html_links}

    for tool_name, filename in tools.items():
        filepath = sample_dir / filename
        if not filepath.exists():
            continue
        text = filepath.read_text(errors="replace")

        tokens = count_tokens(text)
        links = count_links(text)
        noise = noise_keywords(text)
        code = code_block_check(text)
        struct = structure_score(text)

        compression = tokens / html_tokens if html_tokens > 0 else 0
        link_retention = links / html_links if html_links > 0 else 0

        results[tool_name] = {
            "chars": len(text),
            "tokens": tokens,
            "compression_ratio": round(compression, 3),
            "links": links,
            "link_retention": round(link_retention, 3),
            "noise_keywords": noise,
            "noise_count": sum(noise.values()),
            "code_blocks": code,
            "structure": struct,
        }

    return results


def main():
    samples_dir = Path("eval/samples")
    all_results = []

    for sample_dir in sorted(samples_dir.iterdir()):
        if not sample_dir.is_dir():
            continue
        result = evaluate_sample(sample_dir.name, sample_dir)
        all_results.append(result)

    # Save full results
    (samples_dir / "eval_results.json").write_text(
        json.dumps(all_results, indent=2, ensure_ascii=False)
    )

    # Print summary
    tools = ["our_scraper", "our_lol_html", "trafilatura", "readability_js", "playwright"]

    print("=" * 120)
    print("  HEURISTIC EVALUATION — Level 1")
    print("=" * 120)

    # Compression ratio
    print(f"\n  TOKEN COMPRESSION RATIO (lower = more compressed, less noise)")
    print(f"  {'Site':<15s}", end="")
    for t in tools:
        print(f"  {t:>15s}", end="")
    print()
    print(f"  {'─'*15}", end="")
    for _ in tools:
        print(f"  {'─'*15}", end="")
    print()
    for r in all_results:
        print(f"  {r['name']:<15s}", end="")
        for t in tools:
            if t in r:
                val = r[t]["compression_ratio"]
                print(f"  {val:>14.1%}", end="")
            else:
                print(f"  {'N/A':>15s}", end="")
        print()

    # Link retention
    print(f"\n  LINK RETENTION (higher = more links preserved)")
    print(f"  {'Site':<15s}  {'HTML links':>10s}", end="")
    for t in tools:
        print(f"  {t:>15s}", end="")
    print()
    print(f"  {'─'*15}  {'─'*10}", end="")
    for _ in tools:
        print(f"  {'─'*15}", end="")
    print()
    for r in all_results:
        print(f"  {r['name']:<15s}  {r['html_links']:>10d}", end="")
        for t in tools:
            if t in r:
                links = r[t]["links"]
                retention = r[t]["link_retention"]
                print(f"  {links:>5d} ({retention:>5.0%})", end="")
            else:
                print(f"  {'N/A':>15s}", end="")
        print()

    # Noise
    print(f"\n  NOISE KEYWORD COUNT (lower = cleaner)")
    print(f"  {'Site':<15s}", end="")
    for t in tools:
        print(f"  {t:>15s}", end="")
    print()
    print(f"  {'─'*15}", end="")
    for _ in tools:
        print(f"  {'─'*15}", end="")
    print()
    for r in all_results:
        print(f"  {r['name']:<15s}", end="")
        for t in tools:
            if t in r:
                print(f"  {r[t]['noise_count']:>15d}", end="")
            else:
                print(f"  {'N/A':>15s}", end="")
        print()

    # Structure
    print(f"\n  STRUCTURE SCORE (headings + lists + code blocks)")
    print(f"  {'Site':<15s}", end="")
    for t in tools:
        print(f"  {t:>15s}", end="")
    print()
    print(f"  {'─'*15}", end="")
    for _ in tools:
        print(f"  {'─'*15}", end="")
    print()
    for r in all_results:
        print(f"  {r['name']:<15s}", end="")
        for t in tools:
            if t in r:
                s = r[t]["structure"]
                cb = r[t]["code_blocks"]["block_count"]
                score = s["headings"] + s["list_items"] + cb
                print(f"  {score:>15d}", end="")
            else:
                print(f"  {'N/A':>15s}", end="")
        print()

    print(f"\n  Results saved to: eval/samples/eval_results.json")


if __name__ == "__main__":
    main()
