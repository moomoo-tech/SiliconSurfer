"""Download all UN AI Hub activities into a single JSON.

Schema requested by user: title, details, team, date.
We keep both the requested simplified view AND the raw record (lossless).
"""
import asyncio
import json
import sys
from pathlib import Path

import httpx

BASE = "https://unaihub.aiforgood.itu.int/unai/ajax/main.php"
HEADERS = {
    "User-Agent": "Mozilla/5.0",
    "X-Requested-With": "XMLHttpRequest",
    "Referer": "https://unaihub.aiforgood.itu.int/activities.php",
}


def parse_listish(s):
    """Fields like un_agency are JSON strings stored inside JSON. Unwrap them."""
    if not s:
        return []
    if isinstance(s, list):
        return s
    try:
        v = json.loads(s)
        return v if isinstance(v, list) else [v]
    except (json.JSONDecodeError, TypeError):
        return [s]


def simplify(rec):
    agencies = parse_listish(rec.get("un_agency"))
    other = (rec.get("un_agency_other_text") or "").strip()
    partnerships = (rec.get("partnerships") or "").strip()
    contact = (rec.get("contact_information") or "").strip()

    team_parts = []
    if agencies:
        team_parts.append("UN Agencies: " + ", ".join(agencies))
    if other:
        team_parts.append("Other: " + other)
    if partnerships:
        team_parts.append("Partnerships: " + partnerships)
    if contact:
        team_parts.append("Contact: " + contact)

    return {
        "id": rec.get("id"),
        "title": rec.get("activity_title"),
        "details": {
            "description": rec.get("description"),
            "activity_type": parse_listish(rec.get("activity_type")),
            "domain": rec.get("domain"),
            "technology_platform": rec.get("technology_platform"),
            "ai_approach": rec.get("ai_approach"),
            "output_type": rec.get("output_type"),
            "activity_maturity": rec.get("activity_maturity"),
            "thematic_focus_sdgs": parse_listish(rec.get("thematic_focus_sdgs")),
            "primary_beneficiary": parse_listish(rec.get("primary_beneficiary")),
            "regions": parse_listish(rec.get("country_region")),
            "region_names": rec.get("region_names"),
            "example_countries": parse_listish(rec.get("example_countries")),
            "links_resources": rec.get("links_resources"),
            "testimonial": rec.get("testimonial"),
        },
        "team": "\n".join(team_parts),
        "date": {
            "start_date": rec.get("start_date"),
            "created_at": rec.get("created_at"),
            "updated_at": rec.get("updated_at"),
        },
    }


async def main():
    # The backend randomizes order per request, so paginating with pageSize=16
    # and dedup across 53 pages still misses rows. A single oversize page
    # ("pageSize=2000") returns the entire set deterministically (838 rows).
    async with httpx.AsyncClient(headers=HEADERS, timeout=180) as client:
        r = await client.post(
            BASE,
            data={"case": "global_map_csv_pagination", "page": 1, "pageSize": 2000},
        )
        r.raise_for_status()
        payload = r.json()
        records = payload.get("data", [])
        print(f"fetched {len(records)} records (total_pages reported={payload.get('total_pages')})")

        # Dedupe defensively
        seen, deduped = set(), []
        for rec in records:
            rid = rec.get("id")
            if rid in seen:
                continue
            seen.add(rid)
            deduped.append(rec)

        simplified = [simplify(r) for r in deduped]

        out_dir = Path("/Users/hucao/projects/SiliconSurfer/scraped")
        out_dir.mkdir(exist_ok=True)
        (out_dir / "unaihub_projects.json").write_text(
            json.dumps(simplified, indent=2, ensure_ascii=False)
        )
        (out_dir / "unaihub_projects_raw.json").write_text(
            json.dumps(deduped, indent=2, ensure_ascii=False)
        )
        print(f"wrote {len(simplified)} projects → {out_dir}/unaihub_projects.json")
        print(f"wrote raw records → {out_dir}/unaihub_projects_raw.json")


if __name__ == "__main__":
    asyncio.run(main())
