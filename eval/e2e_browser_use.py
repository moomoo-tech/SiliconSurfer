"""E2E comparison: SiliconSurfer vs browser-use.

Same 5 goals, same LLM (Gemini). Different Agent framework.
"""

import asyncio
import json
import time
import tomllib
import httpx
from pathlib import Path
from browser_use import Agent
from browser_use.llm.models import ChatOpenAI

_config = tomllib.load(open(Path(__file__).parent.parent / "config.toml", "rb"))
GEMINI_API_KEY = _config["gemini"]["api_key"]

SERVER = "http://localhost:9883"

GOALS = [
    "Go to https://quotes.toscrape.com/login, find the login form fields (username and password field names), and report them.",
    "Go to https://books.toscrape.com/ and find the cheapest book. Report its title and price.",
    "Go to https://quotes.toscrape.com/ and collect all quotes from the first page. Report how many you found and the first quote.",
    "Go to https://books.toscrape.com/, navigate to the Travel category, and list all travel book titles with prices.",
    "Go to https://the-internet.herokuapp.com/login, read the page to find credentials, login, and verify you reached the secure area.",
]


async def run_browser_use(goal: str, timeout: int = 120) -> dict:
    """Run a goal with browser-use."""
    t0 = time.perf_counter()

    openai_key = _config["openai"]["api_key"]
    openai_model = _config["openai"]["model"]
    llm = ChatOpenAI(
        model=openai_model,
        api_key=openai_key,
    )

    agent = Agent(
        task=goal,
        llm=llm,
        max_actions_per_step=3,
    )

    try:
        result = await asyncio.wait_for(agent.run(max_steps=10), timeout=timeout)
        elapsed = time.perf_counter() - t0
        final_result = result.final_result() if result else "No result"
        steps = result.n_steps() if result else 0
        return {
            "success": result.is_done() if result else False,
            "result": str(final_result)[:200],
            "steps": steps,
            "elapsed_s": round(elapsed, 1),
        }
    except asyncio.TimeoutError:
        elapsed = time.perf_counter() - t0
        return {"success": False, "result": "TIMEOUT", "steps": 0, "elapsed_s": round(elapsed, 1)}
    except Exception as e:
        elapsed = time.perf_counter() - t0
        return {"success": False, "result": str(e)[:200], "steps": 0, "elapsed_s": round(elapsed, 1)}


async def run_siliconsurfer(goal: str) -> dict:
    """Run same goal with our SiliconSurfer agent loop."""
    import re
    from playwright.async_api import async_playwright

    # Import the loop logic inline (adapted for async)
    urls = re.findall(r'https?://[^\s,]+', goal)
    start_url = urls[0].rstrip('.') if urls else "https://example.com"

    t0 = time.perf_counter()
    try:
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True, args=["--disable-gpu"])
            page = await browser.new_page()
            success, data = await _agent_loop_async(goal, start_url, page)
            await browser.close()
    except Exception as e:
        success = False
        data = {"steps": 0, "result": str(e)[:200]}

    elapsed = time.perf_counter() - t0
    return {
        "success": success,
        "result": str(data.get("result", ""))[:200],
        "steps": data.get("steps", 0),
        "elapsed_s": round(elapsed, 1),
    }


async def _agent_loop_async(goal, start_url, pw_page, max_steps=10):
    """Async version of agent_loop."""
    GEMINI_URL_LOCAL = f"https://generativelanguage.googleapis.com/v1beta/models/{_config['gemini']['model']}:generateContent?key={GEMINI_API_KEY}"

    async def see(mode="operator"):
        """See current page through Playwright's eyes + our distiller."""
        html = await pw_page.content()
        url = pw_page.url
        r = httpx.post(f"{SERVER}/distill", json={"html": html, "url": url, "distill": mode}, timeout=60)
        return r.json().get("content", "")

    def think(prompt):
        resp = httpx.post(GEMINI_URL_LOCAL, json={"contents": [{"parts": [{"text": prompt}]}]}, timeout=60)
        return resp.json()["candidates"][0]["content"]["parts"][0]["text"]

    current_url = start_url
    await pw_page.goto(current_url, wait_until="domcontentloaded")
    history = []
    collected = []

    for step in range(1, max_steps + 1):
        spider = await see("spider")
        operator = await see("operator")

        history_str = "\n".join(f"  Step {h['step']}: {h['action']} at {h['url']}" for h in history[-5:])
        recent_urls = [h['url'] for h in history[-4:]]
        loop_warn = "\nWARNING: You are looping. Use 'done' now.\n" if len(recent_urls) >= 4 and len(set(recent_urls)) <= 2 else ""

        page_info = f"URL: {current_url}\n\nPage content (operator view):\n{operator[:2500]}\n\nLinks:\n{spider[:1500]}"

        prompt = f"""You are an autonomous web Agent. Your goal is:
"{goal}"

STEP 1 — ANALYZE CURRENT STATE (mandatory):
Before deciding your action, you MUST first analyze:
- What page am I on? What do I see?
- Have I already achieved my goal? (Check collected data and page content)
- Am I stuck in a loop? (Check action history)

ACTION HISTORY:
{history_str}{loop_warn}
Data collected: {json.dumps(collected[:3]) if collected else 'none'}

What you see:
{page_info}

STEP 2 — DECIDE ACTION:
Return a JSON object. You MUST include "state_analysis" first:

{{"state_analysis": "I see [describe what you observe]. My goal is [X]. I have [done/not done Y].",
  "is_goal_achieved": true/false,
  "action": "navigate|fill_form|click|collect|done",
  ...}}

Action types:
1. {{"action": "navigate", "url": "https://...", "reason": "..."}}
2. {{"action": "fill_form", "fields": {{"name": "value"}}, "submit_selector": "button[type=submit]", "reason": "..."}}
3. {{"action": "click", "selector": "css selector", "reason": "..."}}
4. {{"action": "collect", "data": {{...}}, "reason": "..."}}
5. {{"action": "done", "result": "summary", "data": [...]}}

Rules:
- If is_goal_achieved is true, you MUST use "done"
- If you are looping, use "done" with what you have
- NEVER revisit a URL"""

        raw = think(prompt)
        try:
            c = raw.strip()
            if "```" in c: c = c.split("```")[1].replace("json\n","").strip()
            si = c.find("{"); ei = c.rfind("}") + 1
            decision = json.loads(c[si:ei]) if si >= 0 else {"action": "done", "result": "parse error"}
        except Exception:
            decision = {"action": "done", "result": "parse error"}

        action = decision.get("action", "done")
        if decision.get("is_goal_achieved") and action != "done":
            action = "done"

        history.append({"step": step, "url": current_url, "action": action})

        if action == "done":
            return True, {"steps": step, "result": decision.get("result", ""), "data": collected}
        elif action == "navigate":
            url = decision.get("url", "")
            if url.startswith("http"):
                await pw_page.goto(url, wait_until="domcontentloaded", timeout=15000)
                current_url = url
        elif action == "fill_form":
            for name, val in decision.get("fields", {}).items():
                try:
                    sel = f"[name='{name}']"
                    if await pw_page.query_selector(sel):
                        await pw_page.fill(sel, str(val))
                except Exception: pass
            submit = decision.get("submit_selector", "")
            if submit:
                try:
                    await pw_page.click(submit, timeout=5000)
                    await pw_page.wait_for_load_state("domcontentloaded")
                    current_url = pw_page.url
                except Exception: pass
        elif action == "click":
            try:
                await pw_page.click(decision.get("selector", ""), timeout=5000)
                await pw_page.wait_for_load_state("domcontentloaded")
                current_url = pw_page.url
            except Exception: pass
        elif action == "collect":
            collected.append(decision.get("data", {}))

    # Max steps reached — let LLM summarize from what it already has
    last_page = await see("reader")
    summary = think(f"""You ran out of steps trying to achieve: "{goal}"

Here is everything you collected:
{json.dumps(collected, indent=2)[:2000] if collected else 'nothing collected'}

Here is the last page you were on:
{last_page[:2000]}

Your action history:
{json.dumps([{"step": h["step"], "action": h["action"], "url": h["url"]} for h in history], indent=2)}

Based on ALL the information above, give your best answer for the goal.
Return JSON: {{"result": "your answer", "data": [any structured data]}}""")

    try:
        c = summary.strip()
        if "```" in c: c = c.split("```")[1].replace("json\n","").strip()
        si = c.find("{"); ei = c.rfind("}") + 1
        final = json.loads(c[si:ei]) if si >= 0 else {"result": summary[:200]}
    except Exception:
        final = {"result": summary[:200]}

    return True, {"steps": max_steps, "result": final.get("result", ""), "data": final.get("data", collected)}


async def main():
    # Ensure server is running
    import subprocess, os
    env = {**os.environ, "PORT": "9883"}
    server_proc = subprocess.Popen(
        ["./target/release/agent-browser-server"],
        env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    await asyncio.sleep(4)

    print("=" * 90)
    print("  E2E: SiliconSurfer vs browser-use — Same Goals, Same LLM")
    print("=" * 90)

    results = {"siliconsurfer": [], "browser_use": []}

    for i, goal in enumerate(GOALS):
        print(f"\n{'─'*90}")
        print(f"  Goal {i+1}: {goal[:70]}...")
        print(f"{'─'*90}")

        # SiliconSurfer
        print(f"  Running SiliconSurfer...")
        ss_result = await run_siliconsurfer(goal)
        results["siliconsurfer"].append(ss_result)
        ss_icon = "✓" if ss_result["success"] else "✗"
        print(f"  SiliconSurfer: {ss_icon} ({ss_result['steps']} steps, {ss_result['elapsed_s']}s)")

        # browser-use
        print(f"  Running browser-use...")
        bu_result = await run_browser_use(goal)
        results["browser_use"].append(bu_result)
        bu_icon = "✓" if bu_result["success"] else "✗"
        print(f"  browser-use:   {bu_icon} ({bu_result['steps']} steps, {bu_result['elapsed_s']}s)")

    # Summary
    print(f"\n{'='*90}")
    print(f"  SUMMARY")
    print(f"{'='*90}\n")

    print(f"  {'Goal':<50s}  {'SiliconSurfer':>14s}  {'browser-use':>12s}")
    print(f"  {'─'*50}  {'─'*14}  {'─'*12}")

    ss_pass = 0
    bu_pass = 0
    ss_time = 0
    bu_time = 0

    for i, goal in enumerate(GOALS):
        ss = results["siliconsurfer"][i]
        bu = results["browser_use"][i]
        ss_icon = "✓" if ss["success"] else "✗"
        bu_icon = "✓" if bu["success"] else "✗"
        ss_pass += int(ss["success"])
        bu_pass += int(bu["success"])
        ss_time += ss["elapsed_s"]
        bu_time += bu["elapsed_s"]
        short_goal = goal[:48]
        print(f"  {short_goal:<50s}  {ss_icon} {ss['elapsed_s']:>6.1f}s {ss['steps']:>2d}st  {bu_icon} {bu['elapsed_s']:>5.1f}s {bu['steps']:>2d}st")

    print(f"  {'─'*50}  {'─'*14}  {'─'*12}")
    print(f"  {'TOTAL':<50s}  {ss_pass}/5 {ss_time:>5.1f}s     {bu_pass}/5 {bu_time:>4.1f}s")

    # Save
    Path("eval/samples/e2e_vs_browseruse.json").write_text(json.dumps(results, indent=2, default=str))
    print(f"\n  Saved to: eval/samples/e2e_vs_browseruse.json")

    # Cleanup server
    server_proc.terminate()


if __name__ == "__main__":
    asyncio.run(main())
