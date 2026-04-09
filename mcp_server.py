"""SiliconSurfer MCP Server — PyO3 direct, no HTTP server.

Two tools: observe + act
Transport: stdio (Claude Desktop / Claude Code)
Backend: Rust core via PyO3 FFI (in-process, zero network overhead)
"""

import json
import os
from pathlib import Path
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

app = Server("silicon-surfer")

# Try PyO3 direct import first, fallback to HTTP
_use_pyo3 = False
_use_http = False

try:
    import agent_browser
    _use_pyo3 = True
except ImportError:
    # PyO3 not built — fallback to HTTP server
    import httpx
    _use_http = True

SERVER_URL = "http://localhost:9883"
_server_proc = None

# Persistent browser session for observe/act continuity.
# Created lazily on first use; keeps the page alive between calls.
_session = None

def _get_session():
    """Get or create the persistent BrowserSession."""
    global _session
    if _session is None and _use_pyo3:
        _session = agent_browser.Session()
    return _session


def _ensure_http_server():
    """Start HTTP server as fallback when PyO3 not available."""
    global _server_proc
    if not _use_http:
        return
    try:
        httpx.get(f"{SERVER_URL}/health", timeout=2)
        return
    except Exception:
        pass

    import subprocess, time, signal, atexit, tempfile

    # Cleanup zombies
    pid_file = Path(__file__).parent / ".server.pid"
    if pid_file.exists():
        try:
            os.kill(int(pid_file.read_text().strip()), 9)
        except (ValueError, ProcessLookupError, PermissionError):
            pass
        pid_file.unlink(missing_ok=True)

    lock = Path(tempfile.gettempdir()) / "chromiumoxide-runner" / "SingletonLock"
    lock.unlink(missing_ok=True)

    env = {**os.environ, "PORT": "9883"}
    binary = Path(__file__).parent / "target" / "release" / "agent-browser-server"
    if not binary.exists():
        binary = Path(__file__).parent / "target" / "debug" / "agent-browser-server"

    _server_proc = subprocess.Popen(
        [str(binary)], env=env,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    pid_file.write_text(str(_server_proc.pid))
    time.sleep(4)

    def _shutdown():
        if _server_proc and _server_proc.poll() is None:
            _server_proc.terminate()
            try:
                _server_proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                _server_proc.kill()
        pid_file.unlink(missing_ok=True)

    atexit.register(_shutdown)
    signal.signal(signal.SIGTERM, lambda *_: _shutdown())


def _fetch(url: str, distill: str = "reader") -> dict:
    """Fetch and distill a URL."""
    if _use_pyo3:
        fast = True
        result = agent_browser.fetch(url, mode="t0", fast=fast)
        # TODO: pass distill mode through PyO3 when available
        return result
    else:
        _ensure_http_server()
        r = httpx.post(f"{SERVER_URL}/fetch",
                       json={"url": url, "fast": True, "distill": distill}, timeout=60)
        return r.json()


def _distill_html(html: str, url: str, distill: str = "reader") -> dict:
    """Distill raw HTML (for when we have page content from browser)."""
    if _use_http:
        _ensure_http_server()
        r = httpx.post(f"{SERVER_URL}/distill",
                       json={"html": html, "url": url, "distill": distill}, timeout=60)
        return r.json()
    else:
        # PyO3 direct distill — TODO: add distill_html to PyO3 bindings
        from agent_browser_core.distiller_fast import FastDistiller
        content = FastDistiller.distill(html, distill, url)
        return {"content": content, "content_length": len(content)}


@app.list_tools()
async def list_tools():
    mode_desc = "PyO3 direct (in-process)" if _use_pyo3 else "HTTP server (localhost:9883)"
    return [
        Tool(
            name="observe",
            description=f"""See a webpage through SiliconSurfer's multi-mode vision. Backend: {mode_desc}

Modes:
- "reader" (default): Clean markdown for reading. Strips all UI noise.
- "operator": Shows interactive elements with @e1, @e2 references. Use before act().
- "spider": JSON with all links (nav/content/footer).
- "data": JSON with tables and lists.
- "developer": DOM skeleton with attributes.

After "operator" mode, use act() with @e references to interact.""",
            inputSchema={
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to observe"},
                    "mode": {
                        "type": "string",
                        "enum": ["reader", "operator", "spider", "data", "developer"],
                        "default": "reader",
                    },
                },
                "required": ["url"],
            },
        ),
        Tool(
            name="act",
            description="""Execute an action on a @e element. Must call observe(mode="operator") first.
After every act(), @e refs are invalidated — call observe() again before next act().

Actions: click, fill, submit, navigate, set_cookies
Examples: act("click", "@e3"), act("fill", "@e1", "admin"), act("navigate", "https://...")
Cookies: act("set_cookies", "", '[{"name":"session","value":"abc","domain":".example.com"}]')""",
            inputSchema={
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["click", "fill", "submit", "navigate", "set_cookies"],
                    },
                    "target": {"type": "string", "description": "@eN reference, URL, or cookie domain"},
                    "value": {"type": "string", "default": "", "description": "Text for fill, or JSON array for set_cookies"},
                },
                "required": ["action", "target"],
            },
        ),
    ]


@app.call_tool()
async def call_tool(name: str, arguments: dict):
    if name == "observe":
        url = arguments.get("url", "")
        mode = arguments.get("mode", "reader")
        if not url:
            return [TextContent(type="text", text="Error: url is required")]

        try:
            session = _get_session()
            if session and mode in ("operator", "developer"):
                # Stateful path: use BrowserSession so page stays alive for act()
                session.navigate(url)
                content = session.see(mode)
                title = ""  # session.see() returns distilled content directly
                length = len(content)
            else:
                # Stateless path: T0 fetch for read-only modes (reader/spider/data)
                result = _fetch(url, distill=mode)
                content = result.get("content", "")
                title = result.get("title", "")
                length = result.get("content_length", 0)

            if mode in ("spider", "data"):
                try:
                    content = json.dumps(json.loads(content), indent=2, ensure_ascii=False)
                except json.JSONDecodeError:
                    pass

            header = f"# {title}\n" if title else ""
            footer = f"\n\n---\n_URL: {url} | {length} chars | mode: {mode}_"
            if mode == "operator":
                footer += "\n_Use act() with @e references to interact._"

            return [TextContent(type="text", text=f"{header}{content}{footer}")]
        except Exception as e:
            return [TextContent(type="text", text=f"Error: {e}")]

    elif name == "act":
        action = arguments.get("action", "")
        target = arguments.get("target", "")
        value = arguments.get("value", "")

        session = _get_session()
        if not session:
            return [TextContent(type="text", text="Error: PyO3 not available — browser actions require native bindings.")]

        try:
            if action == "navigate":
                result = session.navigate(target)
                # Return operator view of the new page
                content = session.see("operator")
                return [TextContent(type="text", text=f"Navigated to {target}.\n\n{content[:2000]}")]

            elif action == "click":
                if target.startswith("@"):
                    result = session.click_agent_ref(target)
                else:
                    result = session.click(target)
                detail = result.get("detail", "")
                url = result.get("url", "")
                return [TextContent(type="text", text=f"Clicked {target}. {detail}\nURL: {url}")]

            elif action == "fill":
                if target.startswith("@"):
                    result = session.fill_agent_ref(target, value)
                else:
                    result = session.fill(target, value)
                detail = result.get("detail", "")
                return [TextContent(type="text", text=f"Filled {target} = {value!r}. {detail}")]

            elif action == "submit":
                result = session.submit(target if target else "form")
                detail = result.get("detail", "")
                url = result.get("url", "")
                return [TextContent(type="text", text=f"Submitted. {detail}\nURL: {url}")]

            elif action == "set_cookies":
                # value is JSON array: [{"name":"x","value":"y","domain":".example.com"}]
                count = session.set_cookies(value)
                return [TextContent(type="text", text=f"Set {count} cookie(s). Navigate to apply.")]

            else:
                return [TextContent(type="text", text=f"Unknown action: {action}")]

        except Exception as e:
            return [TextContent(type="text", text=f"Action failed: {e}")]

    return [TextContent(type="text", text=f"Unknown tool: {name}")]


async def main():
    async with stdio_server() as (read_stream, write_stream):
        await app.run(read_stream, write_stream, app.create_initialization_options())


if __name__ == "__main__":
    import asyncio
    asyncio.run(main())
