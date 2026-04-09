"""Tests for probe.py — pure logic (no server needed)."""

from python.agent_browser.probe import diff_snapshots, PROBE_TOOL_DEFINITIONS


class TestDiffSnapshots:
    def test_identical(self):
        nodes = [{"tag": "div", "id": "app", "class": "", "text": "hello", "children_count": 2}]
        result = diff_snapshots(nodes, nodes)
        assert result["changed"] is False
        assert result["added"] == []
        assert result["removed"] == []
        assert result["modified"] == []

    def test_added(self):
        before = [{"tag": "div", "id": "a", "class": ""}]
        after = [
            {"tag": "div", "id": "a", "class": ""},
            {"tag": "span", "id": "b", "class": "new"},
        ]
        result = diff_snapshots(before, after)
        assert result["changed"] is True
        assert len(result["added"]) == 1
        assert result["added"][0]["id"] == "b"
        assert result["removed"] == []

    def test_removed(self):
        before = [
            {"tag": "div", "id": "a", "class": ""},
            {"tag": "span", "id": "b", "class": "old"},
        ]
        after = [{"tag": "div", "id": "a", "class": ""}]
        result = diff_snapshots(before, after)
        assert result["changed"] is True
        assert len(result["removed"]) == 1
        assert result["removed"][0]["id"] == "b"

    def test_modified_text(self):
        before = [{"tag": "h1", "id": "title", "class": "", "text": "Old Title", "children_count": 0}]
        after = [{"tag": "h1", "id": "title", "class": "", "text": "New Title", "children_count": 0}]
        result = diff_snapshots(before, after)
        assert result["changed"] is True
        assert len(result["modified"]) == 1
        assert result["modified"][0]["before"]["text"] == "Old Title"
        assert result["modified"][0]["after"]["text"] == "New Title"

    def test_modified_children_count(self):
        before = [{"tag": "div", "id": "list", "class": "", "text": "", "children_count": 3}]
        after = [{"tag": "div", "id": "list", "class": "", "text": "", "children_count": 5}]
        result = diff_snapshots(before, after)
        assert result["changed"] is True
        assert len(result["modified"]) == 1

    def test_empty_snapshots(self):
        result = diff_snapshots([], [])
        assert result["changed"] is False

    def test_mixed_changes(self):
        before = [
            {"tag": "div", "id": "keep", "class": "", "text": "same"},
            {"tag": "p", "id": "remove", "class": "", "text": "bye"},
            {"tag": "h1", "id": "change", "class": "", "text": "old", "children_count": 1},
        ]
        after = [
            {"tag": "div", "id": "keep", "class": "", "text": "same"},
            {"tag": "span", "id": "new", "class": "", "text": "hi"},
            {"tag": "h1", "id": "change", "class": "", "text": "new", "children_count": 1},
        ]
        result = diff_snapshots(before, after)
        assert result["changed"] is True
        assert len(result["added"]) == 1
        assert len(result["removed"]) == 1
        assert len(result["modified"]) == 1


class TestProbeToolDefinitions:
    def test_has_required_tools(self):
        names = [t["name"] for t in PROBE_TOOL_DEFINITIONS]
        assert "smoke_test" in names
        assert "assert_page" in names

    def test_smoke_test_schema(self):
        tool = next(t for t in PROBE_TOOL_DEFINITIONS if t["name"] == "smoke_test")
        params = tool["parameters"]
        assert params["type"] == "object"
        assert "url" in params["properties"]
        assert "url" in params["required"]

    def test_assert_page_schema(self):
        tool = next(t for t in PROBE_TOOL_DEFINITIONS if t["name"] == "assert_page")
        params = tool["parameters"]
        assert "url" in params["required"]
        assert "selectors" in params["properties"]
