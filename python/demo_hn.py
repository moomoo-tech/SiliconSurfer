"""Demo: Fetch Hacker News → Summarize with Gemini."""

import httpx
import tomllib
from pathlib import Path
from agent_browser import read_webpage

# Load config
_config_path = Path(__file__).parent.parent / "config.toml"
with open(_config_path, "rb") as f:
    _config = tomllib.load(f)

GEMINI_API_KEY = _config["gemini"]["api_key"]
GEMINI_MODEL = _config["gemini"]["model"]
GEMINI_URL = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={GEMINI_API_KEY}"


def ask_gemini(prompt: str) -> str:
    resp = httpx.post(
        GEMINI_URL,
        json={
            "contents": [{"parts": [{"text": prompt}]}],
        },
        timeout=60,
    )
    resp.raise_for_status()
    data = resp.json()
    return data["candidates"][0]["content"]["parts"][0]["text"]


def main():
    # Step 1: Fetch HN with our Rust browser
    print("Fetching Hacker News...")
    result = read_webpage("https://news.ycombinator.com/")
    hn_content = result["content"]
    print(f"Got {result['content_length']} chars\n")

    # Step 2: Send to Gemini
    prompt = f"""以下是今天 Hacker News 首页的内容。请用中文总结：
1. 列出今天最热门的 10 个话题（标题 + 一句话说明为什么值得关注）
2. 归纳今天 HN 社区关注的主要技术趋势

内容：
{hn_content}"""

    print("Asking Gemini...\n")
    summary = ask_gemini(prompt)
    print(summary)


if __name__ == "__main__":
    main()
