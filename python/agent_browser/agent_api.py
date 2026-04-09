"""High-level API designed for AI Agents / LLM tool calling.

Each function is a self-contained "tool" that an Agent can invoke.
Returns structured data optimized for LLM consumption.
"""

from typing import Optional
from .client import AgentBrowser

_browser: Optional[AgentBrowser] = None


def _get_browser() -> AgentBrowser:
    global _browser
    if _browser is None:
        _browser = AgentBrowser()
        _browser.start()
    return _browser


# ---- Agent Tools ----


def read_webpage(url: str) -> dict:
    """Read a webpage and return clean content for LLM processing.

    Use this when: Agent needs to read/understand a webpage.

    Args:
        url: The webpage URL to read.

    Returns:
        {
            "title": "Page Title",
            "content": "Clean markdown content...",
            "url": "https://...",
            "content_length": 1234
        }
    """
    result = _get_browser().fetch(url, output="markdown")
    return {
        "title": result.get("title"),
        "content": result["content"],
        "url": result["url"],
        "content_length": result["content_length"],
    }


def search_and_read(query: str, engine: str = "duckduckgo") -> dict:
    """Search the web and return results as clean text.

    Use this when: Agent needs to find information online.

    Args:
        query: Search query string.
        engine: Search engine ("duckduckgo").

    Returns:
        {
            "query": "original query",
            "content": "Search results as markdown...",
            "url": "search URL used"
        }
    """
    import urllib.parse

    if engine == "duckduckgo":
        search_url = f"https://html.duckduckgo.com/html/?q={urllib.parse.quote(query)}"
    else:
        search_url = f"https://html.duckduckgo.com/html/?q={urllib.parse.quote(query)}"

    result = _get_browser().fetch(search_url, output="text")
    return {
        "query": query,
        "content": result["content"],
        "url": search_url,
    }


async def read_many(urls: list[str]) -> list[dict]:
    """Read multiple webpages concurrently.

    Use this when: Agent needs to compare or gather info from multiple sources.

    Args:
        urls: List of webpage URLs to read.

    Returns:
        List of {"title", "content", "url", "content_length"} dicts.
        Failed fetches include an "error" key instead.
    """
    results = await _get_browser().fetch_many_async(urls, output="markdown")
    out = []
    for r in results:
        if "error" in r:
            out.append(r)
        else:
            out.append({
                "title": r.get("title"),
                "content": r["content"],
                "url": r["url"],
                "content_length": r["content_length"],
            })
    return out


def extract_text(url: str) -> str:
    """Extract plain text from a webpage. No markdown formatting.

    Use this when: Agent needs raw text for analysis/comparison.

    Args:
        url: The webpage URL.

    Returns:
        Plain text string.
    """
    result = _get_browser().fetch(url, output="text")
    return result["content"]


# ---- Tool Definitions (for LLM function calling) ----

TOOL_DEFINITIONS = [
    {
        "name": "read_webpage",
        "description": "Read a webpage and return its content as clean markdown. Use when you need to understand what a webpage says.",
        "parameters": {
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The full URL of the webpage to read",
                }
            },
            "required": ["url"],
        },
    },
    {
        "name": "search_and_read",
        "description": "Search the web for information. Returns search results as text.",
        "parameters": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query",
                }
            },
            "required": ["query"],
        },
    },
    {
        "name": "read_many",
        "description": "Read multiple webpages at once. Use when comparing information across sources.",
        "parameters": {
            "type": "object",
            "properties": {
                "urls": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of URLs to read",
                }
            },
            "required": ["urls"],
        },
    },
    {
        "name": "extract_text",
        "description": "Extract plain text from a webpage without any formatting.",
        "parameters": {
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The full URL of the webpage",
                }
            },
            "required": ["url"],
        },
    },
]


def handle_tool_call(name: str, arguments: dict) -> str | dict | list:
    """Dispatch an LLM tool call to the appropriate function.

    Usage in your Agent loop:
        tool_result = handle_tool_call(tool_name, tool_args)
    """
    import asyncio

    tools = {
        "read_webpage": read_webpage,
        "search_and_read": search_and_read,
        "read_many": lambda **kw: asyncio.run(read_many(**kw)),
        "extract_text": extract_text,
    }

    fn = tools.get(name)
    if fn is None:
        raise ValueError(f"Unknown tool: {name}")

    return fn(**arguments)
