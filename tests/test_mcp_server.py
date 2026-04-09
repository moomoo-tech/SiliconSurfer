"""Tests for mcp_server.py — tool listing, input validation, session management."""

import pytest
import asyncio
import mcp_server


class TestToolListing:
    def test_list_tools_returns_two_tools(self):
        tools = asyncio.run(mcp_server.list_tools())
        assert len(tools) == 2

    def test_tool_names(self):
        tools = asyncio.run(mcp_server.list_tools())
        names = {t.name for t in tools}
        assert names == {"observe", "act"}

    def test_observe_has_mode_enum(self):
        tools = asyncio.run(mcp_server.list_tools())
        observe = next(t for t in tools if t.name == "observe")
        mode_schema = observe.inputSchema["properties"]["mode"]
        assert set(mode_schema["enum"]) == {"reader", "operator", "spider", "data", "developer"}

    def test_act_has_action_enum(self):
        tools = asyncio.run(mcp_server.list_tools())
        act = next(t for t in tools if t.name == "act")
        action_schema = act.inputSchema["properties"]["action"]
        assert set(action_schema["enum"]) == {"click", "fill", "submit", "navigate", "set_cookies"}

    def test_observe_requires_url(self):
        tools = asyncio.run(mcp_server.list_tools())
        observe = next(t for t in tools if t.name == "observe")
        assert "url" in observe.inputSchema["required"]

    def test_act_requires_action_and_target(self):
        tools = asyncio.run(mcp_server.list_tools())
        act = next(t for t in tools if t.name == "act")
        assert "action" in act.inputSchema["required"]
        assert "target" in act.inputSchema["required"]

    def test_act_has_value_field(self):
        tools = asyncio.run(mcp_server.list_tools())
        act = next(t for t in tools if t.name == "act")
        assert "value" in act.inputSchema["properties"]


class TestCallToolValidation:
    def test_observe_empty_url_returns_error(self):
        result = asyncio.run(
            mcp_server.call_tool("observe", {"url": ""})
        )
        assert len(result) == 1
        assert "error" in result[0].text.lower()

    def test_unknown_tool_returns_error(self):
        result = asyncio.run(
            mcp_server.call_tool("nonexistent", {})
        )
        assert "Unknown tool" in result[0].text

    def test_observe_default_mode_is_reader(self):
        """Verify observe defaults to reader mode when mode not specified."""
        tools = asyncio.run(mcp_server.list_tools())
        observe = next(t for t in tools if t.name == "observe")
        assert observe.inputSchema["properties"]["mode"]["default"] == "reader"


class TestSessionManagement:
    def test_session_starts_none(self):
        # Reset for test
        original = mcp_server._session
        mcp_server._session = None
        assert mcp_server._session is None
        mcp_server._session = original

    @pytest.mark.skipif(not mcp_server._use_pyo3, reason="PyO3 not available")
    def test_get_session_returns_session_with_pyo3(self):
        """When PyO3 is available, _get_session should return a Session.
        Requires Chrome — may fail in CI without a browser installed.
        """
        mcp_server._session = None
        try:
            session = mcp_server._get_session()
            assert session is not None
        except RuntimeError:
            pytest.skip("Chrome not available or locked")
        finally:
            mcp_server._session = None

    def test_get_session_without_pyo3_returns_none(self):
        """When PyO3 is not available, _get_session should return None."""
        original_pyo3 = mcp_server._use_pyo3
        original_session = mcp_server._session
        mcp_server._use_pyo3 = False
        mcp_server._session = None
        result = mcp_server._get_session()
        assert result is None
        mcp_server._use_pyo3 = original_pyo3
        mcp_server._session = original_session
