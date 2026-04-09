"""Tests for agent_api.py — tool definitions and dispatch logic."""

import pytest
from python.agent_browser.agent_api import TOOL_DEFINITIONS, handle_tool_call


class TestToolDefinitions:
    def test_all_tools_present(self):
        names = {t["name"] for t in TOOL_DEFINITIONS}
        assert names == {"read_webpage", "search_and_read", "read_many", "extract_text"}

    def test_all_tools_have_parameters(self):
        for tool in TOOL_DEFINITIONS:
            assert "parameters" in tool, f"{tool['name']} missing parameters"
            assert tool["parameters"]["type"] == "object"
            assert "properties" in tool["parameters"]
            assert "required" in tool["parameters"]
            assert len(tool["parameters"]["required"]) > 0

    def test_all_tools_have_description(self):
        for tool in TOOL_DEFINITIONS:
            assert "description" in tool
            assert len(tool["description"]) > 10

    def test_read_webpage_requires_url(self):
        tool = next(t for t in TOOL_DEFINITIONS if t["name"] == "read_webpage")
        assert "url" in tool["parameters"]["required"]
        assert tool["parameters"]["properties"]["url"]["type"] == "string"

    def test_search_and_read_requires_query(self):
        tool = next(t for t in TOOL_DEFINITIONS if t["name"] == "search_and_read")
        assert "query" in tool["parameters"]["required"]

    def test_read_many_requires_urls_array(self):
        tool = next(t for t in TOOL_DEFINITIONS if t["name"] == "read_many")
        assert "urls" in tool["parameters"]["required"]
        assert tool["parameters"]["properties"]["urls"]["type"] == "array"


class TestHandleToolCall:
    def test_unknown_tool_raises(self):
        with pytest.raises(ValueError, match="Unknown tool"):
            handle_tool_call("nonexistent_tool", {})

    def test_dispatch_map_complete(self):
        """All defined tools should be dispatchable."""
        # We can't call them without a server, but we can verify the dispatch map
        # by checking handle_tool_call knows about each tool name
        for tool in TOOL_DEFINITIONS:
            # This should NOT raise ValueError for known tools
            # (it will raise a connection error since no server, which is fine)
            try:
                handle_tool_call(tool["name"], {"url": "http://test", "query": "test", "urls": ["http://test"]})
            except ValueError:
                pytest.fail(f"handle_tool_call doesn't know about tool: {tool['name']}")
            except Exception:
                pass  # Connection errors expected — we just want to verify dispatch works
