# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Validate retained observations and public fixture provenance.

Known gaps must reproduce as gaps for this dated comparison. These assertions
are not the safety requirements for a future updater implementation.
"""
import hashlib
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
RECEIPTS = REPO / "support/receipts/luggage-comparison"
FIXTURES = Path(__file__).resolve().parent / "fixtures"


def read(name):
    return json.loads((RECEIPTS / name).read_text(encoding="utf-8-sig"))


def verify():
    luggage, velo, tuf = [read(n + ".json") for n in ("luggage", "velopack", "tuf")]
    assert luggage["signed_download"] and luggage["fresh_process_stage"]["accepted"]
    assert luggage["artifact_only_tamper_refused"]
    assert luggage["edited_manifest_refused"] and luggage["unsigned_manifest_refused"]
    assert luggage["partial_record_ignored"]
    assert luggage["artifact_and_digest_tamper"]["accepted"]
    assert luggage["unsigned_record_tamper_accepted"] == {
        "destination_changed": True, "format": "Wix", "verified_bytes": True, "version": "99.0.0"}
    assert luggage["highest_observed"] == "0.5.0" and luggage["older_signed_offer_after_higher_seen"] == "0.3.0"
    assert velo["relocated_portable_check"]["portable"] and velo["download"]["pending"]
    assert velo["offline_fresh_process_apply"]["build_version"] == "0.2.0"
    assert velo["outside_current_state_preserved"] and velo["tampered_stage_asset_accepted"]
    failure = velo["replacement_failure"]
    assert failure["directory_handle_injection"] and failure["backup_seen"] and failure["exit"] != 0
    assert not failure["current_binary_survived"] and not failure["backup_binary_survived"]
    assert tuf["expired_repository_refused"] and tuf["expiry_error"] == "timestamp metadata is expired"
    assert tuf["expiry_bypass_negative_control_accepted"]
    assert tuf["rotated_root_version"] == 2 and tuf["edited_timestamp_refused"] and tuf["old_root_at_next_version_refused"]
    provenance = json.loads((FIXTURES / "provenance.json").read_text(encoding="utf-8-sig"))
    for entry in provenance["files"]:
        data = (FIXTURES / entry["path"]).read_bytes()
        assert hashlib.sha256(data).hexdigest() == entry["sha256"], entry["path"]
        assert hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest() == entry["git_blob"], entry["path"]
    receipt = read("receipt.json")
    assert hashlib.sha256((REPO / receipt["cargo_lock"]).read_bytes()).hexdigest() == receipt["cargo_lock_sha256"]
    for entry in receipt["luggage_sources"]:
        assert hashlib.sha256((REPO / entry["path"]).read_bytes()).hexdigest() == entry["sha256"], entry["path"]
    for entry in receipt["evidence"]:
        assert hashlib.sha256((RECEIPTS / entry["path"]).read_bytes()).hexdigest() == entry["sha256"], entry["path"]
    print("Recorded observations, evidence hashes and pinned public fixtures verified.")


if __name__ == "__main__":
    verify()
