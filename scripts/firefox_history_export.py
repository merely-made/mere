#!/usr/bin/env python3
"""Export a Firefox `places.sqlite` copy as `import::ImportedHistoryVisitItem` JSON.

Mere cannot take a SQLite crate, so the read side of a Firefox history import
lives here: python's stdlib `sqlite3` walks a *copy* of the profile database and
writes exactly what serde would deserialize back into
`import::ImportedHistoryVisitItem`, one item per `moz_historyvisits` row.

Two files come out:

* `--out` — the visit array. `referring_url` is resolved through `from_visit`,
  `visit_count_hint` from `moz_places.visit_count`, and `view_time_ms` from
  `moz_places_metadata.total_view_time` (Firefox's own interaction timer, which
  the Rust side takes as `TraceEvent::dwell_ms`).
* `--frecency-out` — Firefox's own `moz_places.frecency`, keyed by the BLAKE3
  hex of the place URL so the Rust side can join without the file carrying a
  readable address list. `--digest sha256` swaps the hash when `blake3` is not
  installed; whichever is used must also be used on the Rust side.

Metadata rows carry no visit id, only `place_id` + `created_at`, so they are
matched one-to-one to the nearest visit of the same place inside
`--metadata-tolerance-ms`. The summary reports how many matched.

Run:

    python scripts/firefox_history_export.py PLACES_COPY.sqlite \\
        --out visits.json --frecency-out frecency.json

The database is opened read-only. This script prints counts and never prints a
URL, title, or query.
"""

from __future__ import annotations

import argparse
import bisect
import collections
import hashlib
import json
import sqlite3
import sys
from pathlib import Path

# Firefox's `nsINavHistoryService` transition constants, mapped onto
# `import::HistoryTransitionKind`. 4 (EMBED) and 8 (FRAMED_LINK) are both
# subframe loads; 5 (REDIRECT_PERMANENT) and 6 (REDIRECT_TEMPORARY) are both
# redirects; 7 (DOWNLOAD) has no import variant and takes the open one.
TRANSITIONS = {
    1: "Link",
    2: "Typed",
    3: "AutoBookmark",
    4: "AutoSubframe",
    5: "Redirect",
    6: "Redirect",
    7: {"Other": "download"},
    8: "AutoSubframe",
    9: "Reload",
}

# Firefox stamps visits in microseconds and interaction metadata in
# milliseconds.
VISIT_DATE_PER_MS = 1000


def digest_fn(name: str):
    """The URL-keying hash, as a hex function. Must match the Rust side."""
    if name == "blake3":
        import blake3  # noqa: PLC0415 — optional, and only on this branch.

        return lambda text: blake3.blake3(text.encode("utf-8")).hexdigest()
    return lambda text: hashlib.sha256(text.encode("utf-8")).hexdigest()


def visit_view_times(connection, tolerance_ms: int) -> dict[int, int]:
    """`total_view_time` per visit id, matched by place and time.

    `moz_places_metadata` records an interaction, not a visit, so the join is
    positional: each metadata row claims the nearest unclaimed visit of its own
    place within the tolerance. Rows whose place has no visit that close (an
    expired visit, a metadata row updated long after) are dropped rather than
    attached to a visit they did not describe.
    """
    by_place: dict[int, list[tuple[int, int]]] = collections.defaultdict(list)
    for visit_id, place_id, visit_date in connection.execute(
        "select id, place_id, visit_date from moz_historyvisits"
    ):
        by_place[place_id].append((visit_date // VISIT_DATE_PER_MS, visit_id))
    for visits in by_place.values():
        visits.sort()

    view_times: dict[int, int] = {}
    claimed: set[int] = set()
    for place_id, created_at, total_view_time in connection.execute(
        "select place_id, created_at, total_view_time from moz_places_metadata"
        " where total_view_time > 0 order by created_at"
    ):
        visits = by_place.get(place_id)
        if not visits:
            continue
        stamps = [stamp for stamp, _ in visits]
        pivot = bisect.bisect_left(stamps, created_at)
        best = None
        for index in range(max(0, pivot - 2), min(len(visits), pivot + 2)):
            stamp, visit_id = visits[index]
            delta = abs(created_at - stamp)
            if delta > tolerance_ms or visit_id in claimed:
                continue
            if best is None or delta < best[0]:
                best = (delta, visit_id)
        if best is None:
            continue
        claimed.add(best[1])
        # An interaction can outlive the visit row's own accounting; keep the
        # largest reading for a visit rather than the last one written.
        view_times[best[1]] = max(view_times.get(best[1], 0), total_view_time)
    return view_times


def export_visits(connection, view_times: dict[int, int]) -> list[dict]:
    rows = connection.execute(
        "select v.id, v.visit_date, v.visit_type, p.url, p.title, p.visit_count,"
        "       (select rp.url from moz_historyvisits rv"
        "          join moz_places rp on rp.id = rv.place_id"
        "         where rv.id = v.from_visit)"
        "  from moz_historyvisits v"
        "  join moz_places p on p.id = v.place_id"
        " order by v.visit_date, v.id"
    )
    items = []
    for visit_id, visit_date, visit_type, url, title, visit_count, referrer in rows:
        title = (title or "").strip() or None
        transition = TRANSITIONS.get(visit_type, {"Other": f"firefox_{visit_type}"})
        items.append(
            {
                "page": {
                    # Firefox stores what its own URL parser produced, so the
                    # verbatim string is already the import crate's normalized
                    # form; eidetic's page key canonicalizes again downstream.
                    "canonical_url": url,
                    "normalized_title": title,
                    "raw_url": None,
                    "raw_title": title,
                    "favicon_url": None,
                },
                "visit_id": str(visit_id),
                "visited_at_unix_secs": visit_date // 1_000_000,
                "visit_count_hint": visit_count if visit_count is not None else None,
                "transition": transition,
                "referring_url": referrer,
                "session_context": None,
                "view_time_ms": view_times.get(visit_id),
            }
        )
    return items


def export_frecency(connection, digest) -> tuple[dict[str, int], int]:
    table: dict[str, int] = {}
    scored = 0
    for url, frecency in connection.execute(
        "select url, frecency from moz_places where url is not null"
    ):
        if frecency is None:
            continue
        if frecency > 0:
            scored += 1
        table[digest(url)] = frecency
    return table, scored


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("places", type=Path, help="a copy of places.sqlite")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--frecency-out", type=Path, required=True)
    parser.add_argument("--digest", choices=("blake3", "sha256"), default="blake3")
    parser.add_argument("--metadata-tolerance-ms", type=int, default=120_000)
    args = parser.parse_args(argv)

    digest = digest_fn(args.digest)
    connection = sqlite3.connect(f"file:{args.places.as_posix()}?mode=ro", uri=True)
    try:
        view_times = visit_view_times(connection, args.metadata_tolerance_ms)
        items = export_visits(connection, view_times)
        frecency, scored = export_frecency(connection, digest)
        metadata_rows = connection.execute(
            "select count(*) from moz_places_metadata where total_view_time > 0"
        ).fetchone()[0]
    finally:
        connection.close()

    args.out.write_text(json.dumps(items), encoding="utf-8")
    args.frecency_out.write_text(json.dumps(frecency), encoding="utf-8")

    kinds = collections.Counter(
        json.dumps(item["transition"], sort_keys=True) for item in items
    )
    print(f"visits={len(items)} bytes={args.out.stat().st_size}")
    print(f"places={len(frecency)} frecency_positive={scored} digest={args.digest}")
    print(
        f"view_time: metadata_rows={metadata_rows} matched={len(view_times)}"
        f" items_with_view_time={sum(1 for i in items if i['view_time_ms'] is not None)}"
    )
    print(f"referring_url_present={sum(1 for i in items if i['referring_url'])}")
    print(f"titled={sum(1 for i in items if i['page']['normalized_title'])}")
    for kind, count in sorted(kinds.items()):
        print(f"transition {kind} {count}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
