"""Agent Browser - AI-optimized web fetcher powered by Rust."""

from .client import AgentBrowser, fetch, fetch_many
from .probe import smoke_test, assert_page, snapshot, diff_snapshots, PROBE_TOOL_DEFINITIONS
from .agent_api import (
    read_webpage,
    search_and_read,
    read_many,
    extract_text,
    handle_tool_call,
    TOOL_DEFINITIONS,
)

__all__ = [
    # Core
    "AgentBrowser",
    "fetch",
    "fetch_many",
    # Agent API
    "read_webpage",
    "search_and_read",
    "read_many",
    "extract_text",
    "handle_tool_call",
    "TOOL_DEFINITIONS",
    # Probe
    "smoke_test",
    "assert_page",
    "snapshot",
    "diff_snapshots",
    "PROBE_TOOL_DEFINITIONS",
]
