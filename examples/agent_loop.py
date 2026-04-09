"""Universal Agent Loop — give it a goal, it figures out the rest.

No hardcoded scenarios. No fixed selectors. No predetermined steps.
Agent sees the page, thinks about what to do, acts, sees again, until done.
"""

import json
import time
import tomllib
import httpx
from pathlib import Path
from playwright.sync_api import sync_playwright

_config = tomllib.load(open(Path(__file__).parent.parent / "config.toml", "rb"))
GEMINI_API_KEY = _config["gemini"]["api_key"]
GEMINI_MODEL = _config["gemini"]["model"]
GEMINI_URL = f"https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_MODEL}:generateContent?key={GEMINI_API_KEY}"
SERVER = "http://localhost:9883"

MAX_STEPS = 15


def see(url: str, mode: str) -> str:
    """Agent's eyes — our distiller."""
    r = httpx.post(f"{SERVER}/fetch", json={"url": url, "fast": True, "distill": mode}, timeout=60)
    return r.json().get("content", "")


def think(prompt: str) -> str:
    """Agent's brain — Gemini."""
    resp = httpx.post(GEMINI_URL, json={"contents": [{"parts": [{"text": prompt}]}]}, timeout=60)
    return resp.json()["candidates"][0]["content"]["parts"][0]["text"]


def agent_loop(goal: str, start_url: str, pw_page):
    """
    Universal Agent loop. No hardcoded logic.

    The Agent:
    1. SEEs the current page (auto-picks best mode)
    2. THINKs about what to do next
    3. ACTs (navigate, click, fill form, or declare done)
    4. Repeats until goal is achieved or max steps reached
    """
    print(f"\n{'='*80}")
    print(f"  GOAL: {goal}")
    print(f"  START: {start_url}")
    print(f"{'='*80}")

    current_url = start_url
    pw_page.goto(current_url, wait_until="domcontentloaded")
    history = []
    collected_data = []

    for step in range(1, MAX_STEPS + 1):
        # ---- SEE: Agent looks at the page ----
        # First, quick spider to know what links exist
        spider_json = see(current_url, "spider")
        # Then operator view for full picture (forms, buttons, content, links)
        operator_view = see(current_url, "operator")

        page_info = f"URL: {current_url}\n\nPage content (operator view):\n{operator_view[:3000]}\n\nLinks on page:\n{spider_json[:2000]}"

        print(f"\n  Step {step}: SEE {current_url} ({len(operator_view)} chars)")

        # ---- THINK: Agent decides what to do ----
        action_prompt = f"""You are an autonomous web Agent. Your goal is:
"{goal}"

You are currently at: {current_url}
Steps taken so far: {len(history)}
Data collected so far: {json.dumps(collected_data[:5]) if collected_data else 'none'}
Previous actions: {json.dumps([h['action'] for h in history[-3:]]) if history else 'none'}

Here is what you see on the current page:
{page_info}

Decide your next action. Return a JSON object with EXACTLY one of these action types:

1. Navigate to a URL:
   {{"action": "navigate", "url": "https://...", "reason": "..."}}

2. Fill a form and submit:
   {{"action": "fill_form", "fields": {{"field_name": "value", ...}}, "submit_selector": "button[type=submit]", "reason": "..."}}

3. Click an element:
   {{"action": "click", "selector": "css selector", "reason": "..."}}

4. Collect data from current page:
   {{"action": "collect", "data": {{...extracted data...}}, "reason": "..."}}

5. Declare goal completed:
   {{"action": "done", "result": "...summary of what was accomplished...", "data": [...]}}

Rules:
- Only use URLs that appear in the page content above
- Only use form field names that appear in the page content
- If you already have enough data, use "done"
- Be efficient — don't revisit pages"""

        raw_decision = think(action_prompt)
        print(f"  Step {step}: THINK")

        # Parse decision
        try:
            # Extract JSON from response
            cleaned = raw_decision.strip()
            if "```" in cleaned:
                cleaned = cleaned.split("```")[1].replace("json\n", "").strip()
            start_idx = cleaned.find("{")
            end_idx = cleaned.rfind("}") + 1
            if start_idx >= 0 and end_idx > start_idx:
                decision = json.loads(cleaned[start_idx:end_idx])
            else:
                decision = {"action": "done", "result": "Could not parse action"}
        except Exception as e:
            print(f"  Step {step}: PARSE ERROR — {e}")
            decision = {"action": "done", "result": f"Parse error: {e}"}

        action = decision.get("action", "done")
        reason = decision.get("reason", "")
        print(f"  Step {step}: ACT — {action}: {reason[:60]}")

        history.append({"step": step, "url": current_url, "action": action, "decision": decision})

        # ---- ACT: Execute the decision ----
        if action == "done":
            result = decision.get("result", "")
            final_data = decision.get("data", collected_data)
            print(f"\n  DONE: {result[:100]}")
            print(f"  Data collected: {json.dumps(final_data, indent=2)[:500] if final_data else 'none'}")
            return True, {"goal": goal, "steps": len(history), "result": result, "data": final_data}

        elif action == "navigate":
            url = decision.get("url", "")
            if url.startswith("http"):
                pw_page.goto(url, wait_until="domcontentloaded", timeout=15000)
                current_url = url
                print(f"  → Navigated to {current_url}")
            else:
                print(f"  ✗ Invalid URL: {url}")

        elif action == "fill_form":
            fields = decision.get("fields", {})
            submit = decision.get("submit_selector", "")
            for name, value in fields.items():
                try:
                    # Try by name attribute first
                    selector = f"[name='{name}']"
                    if pw_page.query_selector(selector):
                        pw_page.fill(selector, str(value))
                        print(f"    Filled {name} = {value}")
                    else:
                        # Try by id
                        selector = f"#{name}"
                        if pw_page.query_selector(selector):
                            pw_page.fill(selector, str(value))
                            print(f"    Filled #{name} = {value}")
                except Exception as e:
                    print(f"    ✗ Could not fill {name}: {e}")

            if submit:
                try:
                    pw_page.click(submit)
                    pw_page.wait_for_load_state("domcontentloaded")
                    current_url = pw_page.url
                    print(f"  → Submitted, now at {current_url}")
                except Exception as e:
                    print(f"  ✗ Submit failed: {e}")

        elif action == "click":
            selector = decision.get("selector", "")
            try:
                pw_page.click(selector, timeout=5000)
                pw_page.wait_for_load_state("domcontentloaded")
                current_url = pw_page.url
                print(f"  → Clicked, now at {current_url}")
            except Exception as e:
                print(f"  ✗ Click failed: {e}")

        elif action == "collect":
            data = decision.get("data", {})
            collected_data.append(data)
            print(f"  → Collected: {json.dumps(data)[:100]}")

        else:
            print(f"  ✗ Unknown action: {action}")

    print(f"\n  TIMEOUT: Max {MAX_STEPS} steps reached")
    return False, {"goal": goal, "steps": MAX_STEPS, "result": "timeout", "data": collected_data}


def main():
    goals = [
        ("Login to quotes site and find an Einstein quote",
         "https://quotes.toscrape.com/login"),
        ("Find the cheapest book and get its full description",
         "https://books.toscrape.com/"),
        ("Collect quotes from 3 different pages",
         "https://quotes.toscrape.com/"),
        ("Find all travel books and their prices",
         "https://books.toscrape.com/"),
        ("Login to the secure area using the credentials shown on the page",
         "https://the-internet.herokuapp.com/login"),
    ]

    results = []

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=["--disable-gpu"])
        page = browser.new_page()

        for goal, start_url in goals:
            success, data = agent_loop(goal, start_url, page)
            results.append({
                "goal": goal,
                "success": success,
                "steps": data["steps"],
            })

        browser.close()

    # Summary
    print(f"\n{'='*80}")
    print(f"  AGENT LOOP RESULTS")
    print(f"{'='*80}")
    for r in results:
        tp = "✓" if r["success"] else "✗"
        print(f"  {tp}  [{r['steps']} steps] {r['goal']}")

    passed = sum(1 for r in results if r["success"])
    print(f"\n  {passed}/{len(results)} goals achieved")

    Path("eval/samples/agent_loop_results.json").write_text(
        json.dumps(results, indent=2, default=str))


if __name__ == "__main__":
    main()
