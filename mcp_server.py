"""SiliconSurfer MCP Server — 5 tools for Claude/LLM to browse the web.

Tools:
1. browse    — Read a page (Reader mode, clean markdown)
2. interact  — See a page with UI elements (@e refs, Operator mode)
3. links     — Get link topology (Spider mode, JSON)
4. extract   — Get structured data (Data mode, JSON)
5. skeleton  — Get DOM skeleton (Developer mode)

Transport: stdio (for Claude Desktop / Claude Code)
"""

import json
import httpx
import tomllib
from pathlib import Path
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

# Load config
_config_path = Path(__file__).parent / "config.toml"
if _config_path.exists():
    _config = tomllib.load(open(_config_path, "rb"))

SERVER_URL = "http://localhost:9883"
_server_proc = None


def _ensure_server():
    """Auto-start the Rust server if not running."""
    global _server_proc
    import subprocess, os, time

    # Check if already running
    try:
        httpx.get(f"{SERVER_URL}/health", timeout=2)
        return
    except Exception:
        pass

    # Start it
    env = {**os.environ, "PORT": "9883"}
    binary = Path(__file__).parent / "target" / "release" / "agent-browser-server"
    if not binary.exists():
        binary = Path(__file__).parent / "target" / "debug" / "agent-browser-server"

    _server_proc = subprocess.Popen(
        [str(binary)], env=env,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    time.sleep(4)  # Wait for startup


app = Server("silicon-surfer")


def _fetch(url: str, distill: str = "reader", fast: bool = True) -> dict:
    """Call our Rust server."""
    r = httpx.post(
        f"{SERVER_URL}/fetch",
        json={"url": url, "fast": fast, "distill": distill},
        timeout=60,
    )
    return r.json()


def _distill(html: str, url: str, distill: str = "reader") -> dict:
    """Distill raw HTML."""
    r = httpx.post(
        f"{SERVER_URL}/distill",
        json={"html": html, "url": url, "distill": distill},
        timeout=60,
    )
    return r.json()


@app.list_tools()
async def list_tools():
    return [
        Tool(
            name="browse",
            description="Read a webpage and return clean markdown content. Best for understanding articles, documentation, blog posts. Strips all UI noise, keeps headings/links/code/tables.",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to browse"},
                },
                "required": ["url"],
            },
        ),
        Tool(
            name="interact",
            description="View a webpage with all interactive elements labeled with @e references. Shows forms, buttons, inputs, links with @e1, @e2 etc. Use this when you need to understand what you can click/fill on a page. Each @e reference maps to a unique element.",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to view"},
                },
                "required": ["url"],
            },
        ),
        Tool(
            name="links",
            description="Extract all links from a page as structured JSON. Categorized into nav_links, content_links, footer_links. Use this to discover what pages are available, find specific links, or map a website's structure.",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to extract links from"},
                },
                "required": ["url"],
            },
        ),
        Tool(
            name="extract",
            description="Extract structured data (tables and lists) from a page as JSON. Tables come with headers and rows. Use this for price comparisons, data tables, product listings.",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to extract data from"},
                },
                "required": ["url"],
            },
        ),
        Tool(
            name="skeleton",
            description="Get the DOM skeleton of a page — HTML structure with id/class/role/data-* attributes but no content. Use this to understand page layout, find CSS selectors, or write automation scripts.",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to analyze"},
                },
                "required": ["url"],
            },
        ),
    ]


@app.call_tool()
async def call_tool(name: str, arguments: dict):
    url = arguments.get("url", "")
    if not url:
        return [TextContent(type="text", text="Error: url is required")]

    mode_map = {
        "browse": "reader",
        "interact": "operator",
        "links": "spider",
        "extract": "data",
        "skeleton": "developer",
    }

    mode = mode_map.get(name, "reader")

    try:
        _ensure_server()
        result = _fetch(url, distill=mode)
        content = result.get("content", "")
        title = result.get("title", "")
        length = result.get("content_length", 0)

        if name in ("links", "extract"):
            # JSON output — format nicely
            try:
                data = json.loads(content)
                content = json.dumps(data, indent=2, ensure_ascii=False)
            except json.JSONDecodeError:
                pass

        header = f"# {title}\n" if title else ""
        footer = f"\n\n---\n_URL: {url} | {length} chars | mode: {mode}_"

        return [TextContent(type="text", text=f"{header}{content}{footer}")]

    except Exception as e:
        return [TextContent(type="text", text=f"Error fetching {url}: {e}")]


async def main():
    async with stdio_server() as (read_stream, write_stream):
        await app.run(read_stream, write_stream, app.create_initialization_options())


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
