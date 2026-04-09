"""Python client for Agent Browser Rust worker."""

import asyncio
import atexit
import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

import httpx

# Locate the Rust binary
_PROJECT_ROOT = Path(__file__).parent.parent.parent
_BINARY_DEBUG = _PROJECT_ROOT / "target" / "debug" / "agent-browser-server"
_BINARY_RELEASE = _PROJECT_ROOT / "target" / "release" / "agent-browser-server"


def _find_binary() -> Path:
    """Find the compiled Rust server binary."""
    if _BINARY_RELEASE.exists():
        return _BINARY_RELEASE
    if _BINARY_DEBUG.exists():
        return _BINARY_DEBUG
    raise FileNotFoundError(
        f"Rust binary not found. Run 'cargo build -p agent-browser-server' first.\n"
        f"Looked in:\n  {_BINARY_RELEASE}\n  {_BINARY_DEBUG}"
    )


class AgentBrowser:
    """Manages a Rust worker process and provides fetch API.

    Usage:
        browser = AgentBrowser()
        browser.start()
        result = browser.fetch("https://example.com")
        browser.stop()

    Or as context manager:
        with AgentBrowser() as browser:
            result = browser.fetch("https://example.com")
            print(result["content"])
    """

    def __init__(self, port: int = 9877):
        self.port = port
        self.base_url = f"http://127.0.0.1:{port}"
        self._process: Optional[subprocess.Popen] = None
        self._client = httpx.Client(timeout=60)
        self._async_client = httpx.AsyncClient(timeout=60)

    def start(self, timeout: float = 5.0) -> "AgentBrowser":
        """Start the Rust worker process."""
        if self._process and self._process.poll() is None:
            return self  # Already running

        binary = _find_binary()
        env = {**os.environ, "PORT": str(self.port)}

        self._process = subprocess.Popen(
            [str(binary)],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

        # Register cleanup
        atexit.register(self.stop)

        # Wait for worker to be ready
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try:
                resp = self._client.get(f"{self.base_url}/health")
                if resp.status_code == 200:
                    return self
            except httpx.ConnectError:
                time.sleep(0.1)

        self.stop()
        raise TimeoutError(f"Worker failed to start within {timeout}s")

    def stop(self):
        """Stop the Rust worker process."""
        if self._process and self._process.poll() is None:
            self._process.send_signal(signal.SIGTERM)
            try:
                self._process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self._process.kill()
        self._process = None

    def fetch(
        self,
        url: str,
        output: str = "markdown",
        timeout_secs: int = 30,
    ) -> dict:
        """Fetch a URL and return clean content (sync).

        Args:
            url: Target URL
            output: "markdown" or "text"
            timeout_secs: Request timeout

        Returns:
            dict with keys: url, title, content, status, content_length
        """
        resp = self._client.post(
            f"{self.base_url}/fetch",
            json={"url": url, "output": output, "timeout_secs": timeout_secs},
        )
        resp.raise_for_status()
        return resp.json()

    async def fetch_async(
        self,
        url: str,
        output: str = "markdown",
        timeout_secs: int = 30,
    ) -> dict:
        """Fetch a URL and return clean content (async)."""
        resp = await self._async_client.post(
            f"{self.base_url}/fetch",
            json={"url": url, "output": output, "timeout_secs": timeout_secs},
        )
        resp.raise_for_status()
        return resp.json()

    async def fetch_many_async(
        self,
        urls: list[str],
        output: str = "markdown",
        timeout_secs: int = 30,
    ) -> list[dict]:
        """Fetch multiple URLs concurrently (async)."""
        tasks = [
            self.fetch_async(url, output=output, timeout_secs=timeout_secs)
            for url in urls
        ]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        out = []
        for url, r in zip(urls, results):
            if isinstance(r, Exception):
                out.append({"url": url, "error": str(r)})
            else:
                out.append(r)
        return out

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, *args):
        self.stop()

    async def __aenter__(self):
        self.start()
        return self

    async def __aexit__(self, *args):
        self.stop()


# --- Convenience functions (module-level singleton) ---

_default: Optional[AgentBrowser] = None


def _get_default() -> AgentBrowser:
    global _default
    if _default is None:
        _default = AgentBrowser()
        _default.start()
    return _default


def fetch(url: str, output: str = "markdown", timeout_secs: int = 30) -> dict:
    """Quick fetch - auto-starts worker if needed."""
    return _get_default().fetch(url, output=output, timeout_secs=timeout_secs)


async def fetch_many(urls: list[str], output: str = "markdown") -> list[dict]:
    """Quick concurrent fetch - auto-starts worker if needed."""
    return await _get_default().fetch_many_async(urls, output=output)
