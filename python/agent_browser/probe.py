"""Probe API — Agent's development logic probe.

Fast smoke tests: check HTTP status, DOM elements, text presence.
"""

from typing import Optional


def smoke_test(
    url: str,
    checks: list[dict] | None = None,
    contains: list[str] | None = None,
    snapshot: bool = False,
    server: str = "http://localhost:9883",
) -> dict:
    """Run a smoke test on a URL.

    Args:
        url: Target URL (e.g. http://localhost:3000/dashboard)
        checks: List of DOM checks, each with:
            - selector: CSS selector (e.g. "#app", "h1", "div.chart")
            - contains_text: Optional expected text substring
            - attr: Optional attribute name to check
            - attr_value: Optional expected attribute value
        contains: List of text strings the page should contain
        snapshot: If True, return DOM structure snapshot
        server: Agent Browser server URL

    Returns:
        dict with: ok, status, elapsed_ms, checks, contains, summary

    Example:
        result = smoke_test(
            "http://localhost:3000",
            checks=[
                {"selector": "#app"},
                {"selector": "h1", "contains_text": "Dashboard"},
                {"selector": "div.chart", "attr": "data-loaded", "attr_value": "true"},
            ],
            contains=["Welcome"],
        )
        if result["ok"]:
            print("All good!")
        else:
            print(result["summary"])
    """
    import httpx

    payload = {
        "url": url,
        "checks": checks or [],
        "contains": contains or [],
        "snapshot": snapshot,
    }

    resp = httpx.post(f"{server}/probe", json=payload, timeout=30)
    resp.raise_for_status()
    return resp.json()


def assert_page(
    url: str,
    status: int = 200,
    selectors: list[str] | None = None,
    contains: list[str] | None = None,
    server: str = "http://localhost:9883",
) -> dict:
    """Simplified probe — assert page is healthy.

    Args:
        url: Target URL
        status: Expected HTTP status code
        selectors: CSS selectors that must exist
        contains: Text strings that must be present

    Returns:
        dict with ok, summary

    Raises:
        AssertionError if any check fails
    """
    checks = [{"selector": s} for s in (selectors or [])]
    result = smoke_test(url, checks=checks, contains=contains, server=server)

    if result["status"] != status:
        raise AssertionError(
            f"Expected HTTP {status}, got {result['status']}\n{result['summary']}"
        )

    if not result["ok"]:
        raise AssertionError(f"Probe failed:\n{result['summary']}")

    return result


def snapshot(
    url: str,
    server: str = "http://localhost:9883",
) -> list[dict]:
    """Get DOM structure snapshot for diff comparison.

    Returns list of top-level body elements with tag, id, class, text preview.
    """
    result = smoke_test(url, snapshot=True, server=server)
    return result.get("snapshot", [])


def diff_snapshots(before: list[dict], after: list[dict]) -> dict:
    """Compare two DOM snapshots.

    Returns:
        dict with: changed (bool), added, removed, modified
    """
    def key(node):
        return (node["tag"], node.get("id", ""), node.get("class", ""))

    before_keys = {key(n): n for n in before}
    after_keys = {key(n): n for n in after}

    before_set = set(before_keys.keys())
    after_set = set(after_keys.keys())

    added = [after_keys[k] for k in after_set - before_set]
    removed = [before_keys[k] for k in before_set - after_set]

    modified = []
    for k in before_set & after_set:
        b, a = before_keys[k], after_keys[k]
        if b.get("text") != a.get("text") or b.get("children_count") != a.get("children_count"):
            modified.append({"before": b, "after": a})

    return {
        "changed": bool(added or removed or modified),
        "added": added,
        "removed": removed,
        "modified": modified,
    }


# Tool definitions for LLM function calling
PROBE_TOOL_DEFINITIONS = [
    {
        "name": "smoke_test",
        "description": "Run a smoke test on a URL to check if the page is working. Checks HTTP status, DOM elements existence, and text presence. Use after writing code to verify it works.",
        "parameters": {
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "URL to test"},
                "checks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "selector": {"type": "string"},
                            "contains_text": {"type": "string"},
                        },
                        "required": ["selector"],
                    },
                    "description": "DOM element checks",
                },
                "contains": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Text strings the page should contain",
                },
            },
            "required": ["url"],
        },
    },
    {
        "name": "assert_page",
        "description": "Assert a page is healthy — checks HTTP status, required DOM elements, and required text. Raises error if anything fails.",
        "parameters": {
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "status": {"type": "integer", "description": "Expected HTTP status (default 200)"},
                "selectors": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "CSS selectors that must exist on the page",
                },
                "contains": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Text that must appear on the page",
                },
            },
            "required": ["url"],
        },
    },
]
