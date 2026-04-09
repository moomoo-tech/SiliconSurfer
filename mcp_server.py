"""SiliconSurfer MCP Server — 2 tools for Claude to browse and interact with the web.

Tools:
1. observe(url, mode) — See a page through 5 visual modes
2. act(action, target, value) — Execute actions on @e elements

The Agent loop: observe → think → act → observe → ...
"""

import json
import subprocess
import os
import time
import httpx
import tomllib
from pathlib import Path
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

_config_path = Path(__file__).parent / "config.toml"
_config = tomllib.load(open(_config_path, "rb")) if _config_path.exists() else {}

SERVER_URL = "http://localhost:9883"
_server_proc = None

# Global state
_current_url = ""
_locator_map = {}

app = Server("silicon-surfer")


def _ensure_server():
    global _server_proc
    try:
        httpx.get(f"{SERVER_URL}/health", timeout=2)
        return
    except Exception:
        pass

    env = {**os.environ, "PORT": "9883"}
    binary = Path(__file__).parent / "target" / "release" / "agent-browser-server"
    if not binary.exists():
        binary = Path(__file__).parent / "target" / "debug" / "agent-browser-server"
    _server_proc = subprocess.Popen(
        [str(binary)], env=env,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    time.sleep(4)


def _fetch(url: str, distill: str = "reader") -> dict:
    _ensure_server()
    r = httpx.post(f"{SERVER_URL}/fetch", json={"url": url, "fast": True, "distill": distill}, timeout=60)
    return r.json()


def _distill(html: str, url: str, distill: str = "reader") -> dict:
    _ensure_server()
    r = httpx.post(f"{SERVER_URL}/distill", json={"html": html, "url": url, "distill": distill}, timeout=60)
    return r.json()


@app.list_tools()
async def list_tools():
    return [
        Tool(
            name="observe",
            description="""See a webpage through SiliconSurfer's multi-mode vision system. Returns structured content optimized for your understanding.

Modes:
- "reader" (default): Clean markdown, best for reading articles/docs. Strips all UI noise.
- "operator": Shows ALL interactive elements with @e references. Use this when you need to interact (click/fill/submit). Each element gets @e1, @e2 etc.
- "spider": Returns JSON with all links on the page, categorized as nav/content/footer.
- "data": Returns structured JSON with tables and lists extracted from the page.
- "developer": Returns DOM skeleton with id/class/role attributes.

After using "operator" mode, you can use the "act" tool with @e references.""",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to observe"},
                    "mode": {
                        "type": "string",
                        "enum": ["reader", "operator", "spider", "data", "developer"],
                        "default": "reader",
                        "description": "Visual mode",
                    },
                },
                "required": ["url"],
            },
        ),
        Tool(
            name="act",
            description="""Execute an action on a webpage element identified by its @e reference.

IMPORTANT: You must first call observe(url, mode="operator") to see the page with @e references before using this tool.

Actions:
- "click": Click the element (links, buttons)
- "fill": Type text into an input field
- "submit": Submit a form (clicks submit button or submits form directly)
- "navigate": Go to a new URL (target should be the URL)

Examples:
  act("click", "@e3")           — Click the 3rd interactive element
  act("fill", "@e1", "admin")   — Type "admin" into the 1st input
  act("submit", "@e5")          — Click the submit button
  act("navigate", "https://example.com") — Go to a URL""",
            inputSchema={
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["click", "fill", "submit", "navigate"],
                        "description": "Action to perform",
                    },
                    "target": {
                        "type": "string",
                        "description": "@e reference (e.g. '@e3') or URL for navigate",
                    },
                    "value": {
                        "type": "string",
                        "default": "",
                        "description": "Text to fill (only for 'fill' action)",
                    },
                },
                "required": ["action", "target"],
            },
        ),
    ]


@app.call_tool()
async def call_tool(name: str, arguments: dict):
    global _current_url, _locator_map

    if name == "observe":
        url = arguments.get("url", "")
        mode = arguments.get("mode", "reader")

        if not url:
            return [TextContent(type="text", text="Error: url is required")]

        try:
            result = _fetch(url, distill=mode)
            content = result.get("content", "")
            title = result.get("title", "")
            length = result.get("content_length", 0)
            _current_url = url

            if mode in ("spider", "data"):
                try:
                    data = json.loads(content)
                    content = json.dumps(data, indent=2, ensure_ascii=False)
                except json.JSONDecodeError:
                    pass

            header = f"# {title}\n" if title else ""
            footer = f"\n\n---\n_URL: {url} | {length} chars | mode: {mode}_"

            if mode == "operator":
                footer += f"\n_Use act() with @e references shown above to interact._"

            return [TextContent(type="text", text=f"{header}{content}{footer}")]

        except Exception as e:
            return [TextContent(type="text", text=f"Error: {e}")]

    elif name == "act":
        action = arguments.get("action", "")
        target = arguments.get("target", "")
        value = arguments.get("value", "")

        if action == "navigate":
            # Navigate is just observe with operator mode
            try:
                result = _fetch(target, distill="operator")
                _current_url = target
                content = result.get("content", "")
                return [TextContent(type="text", text=f"Navigated to {target}.\n\n{content[:2000]}")]
            except Exception as e:
                return [TextContent(type="text", text=f"Navigation failed: {e}")]

        # For click/fill/submit — we need a live browser session.
        # Current limitation: T0 mode can't execute actions.
        # TODO: Wire through to AgentSession when T1 CDP is integrated into MCP.
        return [TextContent(type="text", text=
            f"Action '{action}' on '{target}' received.\n"
            f"⚠️ Direct browser actions require T1 (Chrome CDP) session.\n"
            f"Current MCP runs in T0 (HTTP fetch) mode.\n"
            f"To execute actions, use the agent_loop.py with Playwright,\n"
            f"or wait for T1 CDP integration into MCP server."
        )]

    return [TextContent(type="text", text=f"Unknown tool: {name}")]


async def main():
    async with stdio_server() as (read_stream, write_stream):
        await app.run(read_stream, write_stream, app.create_initialization_options())


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
