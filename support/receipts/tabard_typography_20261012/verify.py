# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Read-only verification of the frozen typography evidence; launches no GPU."""
from pathlib import Path
import hashlib
import json
import struct

root = Path(__file__).resolve().parent
proof = json.loads((root / "qualification.json").read_text())
for name, record in proof["files"].items():
    data = (root / name).read_bytes()
    assert hashlib.sha256(data).hexdigest() == record["sha256"], name
    assert len(data) == record["bytes"], name
    if name.endswith(".png"):
        assert data[:8] == b"\x89PNG\r\n\x1a\n", name
        assert list(struct.unpack(">II", data[16:24])) == record["size"], name
for run in proof["native_runs"]:
    done = (root / run["label"] / "scenario.done").read_text()
    assert done.startswith("RESULT ok\n") and " BLANK" not in done
    assert run["process"]["success"] and not run["process"]["timeout"]
    assert run["process"]["binary_sha256"] == proof["native_binary_sha256"]
assert proof["totals"] == {"processes": 3, "presented": 162,
                           "scenario_captures": 18, "browser_captures": 3, "blank": 0}
assert all((root / name).exists() for name in proof["manual_review"])
assert proof["durability"]["library_and_choice_bytes_unchanged_after_reopen_and_discard"]
assert proof["durability"]["fallback_copy_preserved_requested_family_and_choice"]
print("Typography receipts verified: 3 processes, 162 presentations, 21 retained captures; no runtime launched.")
