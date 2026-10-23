"""Helpers that turn decoded wire events (tools/pgproxy.py) into per-request round trips."""
from __future__ import annotations


def trips_of(events: list[dict]) -> list[dict]:
    """Group the frontend messages of one request into round trips: a simple query is one `Q`;
    an extended round trip ends at its `Sync`. Startup and password messages are ignored."""
    trips, cur = [], []
    front = [e for e in events if e["dir"] == "F" and e["type"] not in ("startup", "p", "X")]
    for e in front:
        cur.append(e)
        if e["type"] in ("Q", "S"):
            trips.append({"front": cur, "end_seq": e["seq"]})
            cur = []
    for t in trips:
        back = []
        for e in events:
            if e["dir"] == "B" and e["seq"] > t["end_seq"]:
                back.append(e)
                if e["type"] == "Z":
                    break
        t["back"] = back
        kinds = [m["type"] for m in t["front"]]
        t["protocol"] = "simple" if kinds == ["Q"] else "extended"
        t["kinds"] = "".join(kinds)
        t["sql"] = next((m["sql"] for m in t["front"] if m["type"] in ("Q", "P")), None)
        bind = next((m for m in t["front"] if m["type"] == "B"), None)
        parse = next((m for m in t["front"] if m["type"] == "P"), None)
        describe = next((m for m in t["front"] if m["type"] == "D"), None)
        t["params"] = bind["params"] if bind else []
        t["bind"], t["parse"], t["describe"] = bind, parse, describe
    return trips
