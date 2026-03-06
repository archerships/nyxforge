#!/usr/bin/env python3
"""Seed the nyxforge-node with three demo bonds for local development.

Safe to run multiple times: existing bonds are detected and skipped.
The node must already be running on http://127.0.0.1:8888/rpc.
"""

import json
import sys
import urllib.request
import urllib.error

RPC_URL = "http://127.0.0.1:8888/rpc"

ISSUER       = [0x11] * 32
ORACLE_KEY   = [0x22] * 32
RETURN_ADDR  = [0x11] * 32
ZERO_ID      = [0] * 32

ORACLE_PARAMS = {
    "oracle_keys":     [ORACLE_KEY],
    "quorum":          1,
    "required_stake":  100_000_000,
    "slash_fraction":  "0.5",
}

VERIFICATION = {
    "attestation_threshold": 1,
    "challenge_period_secs": 86400,
    "dao_override_allowed":  False,
}

LIFEBOND_ALIVE_GOAL = {
    "title":       "Subject Is Alive",
    "description": "The bond subject must be certified alive by vital records authorities.",
    "metric": {
        "data_id":     "subject.lifebond_001.vital_status",
        "operator":    "GreaterThanOrEqual",
        "threshold":   "1",
        "aggregation": None,
    },
    "deadline":        "2125-01-01T00:00:00Z",
    "evidence_format": None,
}

LIFEBOND_HEALTH_GOAL = {
    "title":       "Subject Is in Good Health (Score >= 80)",
    "description": "Bond subject must be certified in good health by a geriatric panel (score >= 80/100).",
    "metric": {
        "data_id":     "subject.lifebond_001.health_score",
        "operator":    "GreaterThanOrEqual",
        "threshold":   "80",
        "aggregation": None,
    },
    "deadline":        "2125-01-01T00:00:00Z",
    "evidence_format": None,
}

DEMO_BONDS = [
    {
        "id":           ZERO_ID,
        "issuer":       ISSUER,
        "state":        "Draft",
        "goals": [LIFEBOND_ALIVE_GOAL, LIFEBOND_HEALTH_GOAL],
        "oracle":       ORACLE_PARAMS,
        "auction": {
            "start_price":   1_000_000,    # 1 DRK
            "reserve_price":    50_000,    # 0.05 DRK
            "duration_secs":   604_800,    # 7 days
        },
        "redemption_value":  100_000_000,  # 100 DRK
        "total_supply":       10_000,
        "bonds_remaining":    10_000,
        "return_address":    RETURN_ADDR,
        "created_at_block":  1,
        "activated_at_secs": None,
        "verification":      VERIFICATION,
    },
    {
        "id":           ZERO_ID,
        "issuer":       ISSUER,
        "state":        "Draft",
        "goals": [{
            "title":       "US Unsheltered Homelessness Below 50k by 2030",
            "description": "Annual HUD PIT unsheltered count must fall below 50,000 people.",
            "metric": {
                "data_id":   "us.hud.pit_count.unsheltered",
                "operator":  "LessThan",
                "threshold": "50000",
                "aggregation": None,
            },
            "deadline":      "2030-01-01T00:00:00Z",
            "evidence_format": None,
        }],
        "oracle":       ORACLE_PARAMS,
        "auction": {
            "start_price":   5_000_000,    # 5 DRK
            "reserve_price":   500_000,    # 0.5 DRK
            "duration_secs":   604_800,
        },
        "redemption_value":   50_000_000,  # 50 DRK
        "total_supply":       10_000,
        "bonds_remaining":    10_000,
        "return_address":    RETURN_ADDR,
        "created_at_block":  100,
        "activated_at_secs": None,
        "verification":      VERIFICATION,
    },
    {
        "id":           ZERO_ID,
        "issuer":       ISSUER,
        "state":        "Draft",
        "goals": [{
            "title":       "Atmospheric CO2 Below 350 ppm by 2045",
            "description": "Annual mean CO2 at Mauna Loa must drop below 350 ppm.",
            "metric": {
                "data_id":   "noaa.co2.monthly_mean_ppm",
                "operator":  "LessThan",
                "threshold": "350",
                "aggregation": None,
            },
            "deadline":      "2045-01-01T00:00:00Z",
            "evidence_format": None,
        }],
        "oracle":       ORACLE_PARAMS,
        "auction": {
            "start_price":   2_000_000,    # 2 DRK
            "reserve_price":   200_000,    # 0.2 DRK
            "duration_secs":   604_800,
        },
        "redemption_value":   20_000_000,  # 20 DRK
        "total_supply":        5_000,
        "bonds_remaining":     5_000,
        "return_address":    RETURN_ADDR,
        "created_at_block":  200,
        "activated_at_secs": None,
        "verification":      VERIFICATION,
    },
]


def rpc(method, params=None):
    body = json.dumps({"method": method, "params": params or {}}).encode()
    req = urllib.request.Request(
        RPC_URL,
        data=body,
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        data = json.loads(resp.read())
    if data.get("error"):
        raise RuntimeError(f"RPC error from {method}: {data['error']}")
    return data["result"]


def bytes_to_hex(byte_list):
    return "".join(f"{b:02x}" for b in byte_list)


def main():
    # Check what's already seeded
    existing = rpc("bonds.list")
    existing_ids = set()
    for b in existing.get("bonds", []):
        existing_ids.add(bytes_to_hex(b["id"]))
    print(f"Node has {len(existing_ids)} bond(s) already.")

    seeded = 0
    for bond in DEMO_BONDS:
        title = bond["goals"][0]["title"]

        # Propose → get canonical ID back
        result = rpc("bonds.propose", {"bond": bond})
        bond_id = result["bond_id"]

        if bond_id in existing_ids:
            print(f"  SKIP  {title[:50]}")
            continue

        print(f"  SEED  {title[:50]}")

        # Advance through the state machine to Active
        rpc("bonds.submit_for_approval", {"bond_id": bond_id})
        rpc("bonds.oracle_accept", {"bond_id": bond_id, "oracle_key": "2" * 64})
        rpc("bonds.issue",         {"bond_id": bond_id})

        seeded += 1
        print(f"        → active  id={bond_id[:16]}…")

    print(f"\nDone. {seeded} bond(s) seeded, {len(existing_ids)} already present.")


if __name__ == "__main__":
    try:
        main()
    except (urllib.error.URLError, ConnectionRefusedError) as e:
        print(f"ERROR: cannot reach node at {RPC_URL}: {e}", file=sys.stderr)
        sys.exit(1)
