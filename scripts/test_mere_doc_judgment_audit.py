#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Regression controls for receipt names and D2 coverage."""

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import mere_doc_judgment_audit as judgment


def record(path):
    return f"""## {path}

- disposition: current
- status line: "Status: current" - accurate: yes
- claims checked: 1 - holds: 1, stale: 0, unverifiable: 0

### Stale claims
None.
### Contradictions
None.
### Recommended action
Retain.
### Notes
Fixture.
"""


class ReceiptCoverage(unittest.TestCase):
    def audit_fixture(self, *, legacy=None, include_second=True):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "design_docs"
            ledger = root / "d2"
            ledger.mkdir()
            for name in ("a", "b"):
                folder = docs / name
                folder.mkdir(parents=True)
                (folder / "README.md").write_text("Status: current\n", encoding="utf-8")
            snapshot = ledger / "snapshot.json"
            snapshot.write_text(json.dumps({"docs": legacy or {}}), encoding="utf-8")
            corrections = ledger / "corrections.json"
            corrections.write_text("{}", encoding="utf-8")
            moved = ledger / "moved.json"
            moved.write_text('{"moved": []}', encoding="utf-8")
            batch = record("a/README.md")
            if include_second:
                batch += "\n" + record("b/README.md")
            (ledger / "batch_1.md").write_text(batch, encoding="utf-8")
            with patch.multiple(judgment, DOCS=docs, LEDGER=ledger,
                                SNAPSHOT=snapshot, CORRECTIONS=corrections,
                                MOVED_OUT=moved,
                                SNAPSHOT_SHA256=hashlib.sha256(snapshot.read_bytes()).hexdigest()):
                return judgment.audit()

    def test_full_paths_disambiguate_receipt_readmes(self):
        result = self.audit_fixture()
        self.assertEqual(result["covered_active_docs"], 2)
        self.assertEqual(result["errors"], [])

    def test_legacy_basename_remains_ambiguous(self):
        result = self.audit_fixture(legacy={"README.md": {"claims": 0}})
        self.assertTrue(any("duplicate active basename" in error for error in result["errors"]))

    def test_one_record_cannot_cover_both_receipts(self):
        result = self.audit_fixture(include_second=False)
        self.assertEqual(result["missing_active_records"], ["b/README.md"])
        self.assertTrue(any("lacks D2 record" in error for error in result["errors"]))


if __name__ == "__main__":
    unittest.main()
