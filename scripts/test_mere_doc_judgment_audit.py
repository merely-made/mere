#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Regression controls for receipt names and D2 coverage."""

import hashlib
import json
import contextlib
import io
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
    def audit_fixture(self, *, legacy=None, include_second=True, relocated=False,
                      register=True, corrupt=False, destination=None, unavailable=False,
                      source="mere_docs/testing/receipt.md"):
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
            relocations = root / "receipt-relocations.json"
            if relocated:
                batch += "\n" + record(source)
                target = root / "testing/mere/receipt.md"
                target.parent.mkdir(parents=True)
                if not unavailable:
                    target.write_bytes(b"changed" if corrupt else b"original")
                if register:
                    relocations.write_text(json.dumps({"schema_version": 1, "receipts": [{
                        "source": source,
                        "destination": destination or "Code/testing/mere/receipt.md",
                        "sha256": hashlib.sha256(b"original").hexdigest(),
                        "git_blob": "1" * 40, "source_commit": "2" * 40,
                    }]}), encoding="utf-8")
            (ledger / "batch_1.md").write_text(batch, encoding="utf-8")
            with patch.multiple(judgment, DOCS=docs, LEDGER=ledger,
                                SNAPSHOT=snapshot, CORRECTIONS=corrections,
                                MOVED_OUT=moved,
                                RECEIPT_RELOCATIONS=relocations, WORKSPACE=root,
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

    def test_registered_receipt_remains_historical_without_editing_ledger(self):
        result = self.audit_fixture(relocated=True)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["inactive_supplemental_records"], ["mere_docs/testing/receipt.md"])

    def test_unregistered_receipt_still_reports_missing_record(self):
        result = self.audit_fixture(relocated=True, register=False)
        self.assertTrue(any("supplemental record is not active" in error for error in result["errors"]))

    def test_changed_external_copy_is_not_accepted(self):
        result = self.audit_fixture(relocated=True, corrupt=True)
        self.assertTrue(any("digest mismatch" in error for error in result["errors"]))
        self.assertEqual(result["registered_receipt_relocations"], [])

    def test_relocation_cannot_escape_evidence_root(self):
        result = self.audit_fixture(relocated=True, destination="Code/testing/mere/../../../receipt.md")
        self.assertTrue(any("invalid or duplicate" in error for error in result["errors"]))

    def test_receipt_namespace_accepts_other_owners_and_nested_packets(self):
        result = self.audit_fixture(relocated=True,
                                    source="cambium_docs/testing/receipts/device/receipt.md")
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["inactive_supplemental_records"],
                         ["cambium_docs/testing/receipts/device/receipt.md"])

    def test_relocation_cannot_hide_active_plans_or_governance(self):
        for source in ("mere_docs/implementation_strategy/receipt.md", "DOC_POLICY.md",
                       "mere_docs/testing/../implementation_strategy/receipt.md"):
            with self.subTest(source=source):
                result = self.audit_fixture(relocated=True, source=source)
                self.assertTrue(any("invalid or duplicate" in error for error in result["errors"]))

    def test_missing_local_evidence_is_reported_without_rewriting_history(self):
        result = self.audit_fixture(relocated=True, unavailable=True)
        self.assertEqual(result["errors"], [])
        self.assertEqual(result["unavailable_relocated_receipts"], ["mere_docs/testing/receipt.md"])

    def test_text_output_exposes_missing_evidence(self):
        result = self.audit_fixture(relocated=True, unavailable=True)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            judgment.print_text_report(result)
        self.assertIn("Relocated receipts unavailable locally: 1", output.getvalue())
        self.assertIn("unavailable receipt: mere_docs/testing/receipt.md", output.getvalue())
        self.assertIn("Coverage is not artifact verification", output.getvalue())


if __name__ == "__main__":
    unittest.main()
