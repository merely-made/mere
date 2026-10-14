# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Verify the actual browser download without reading browser private state."""
import hashlib
import json
from pathlib import Path
import shutil
import sys

from blake3 import blake3

actual_path = Path(sys.argv[1]).resolve(strict=True)
receipt_dir = Path(__file__).resolve().parent
record_id = "b40c579d5234ed1f14d7512474a18c958c5e07f1a16f27afc42d337eeabd07ba"
expected_filename = f"selected-clip-{record_id}.mereclip"
assert actual_path.name == expected_filename
wire = actual_path.read_bytes()
package = json.loads(wire)
fixture = json.loads((receipt_dir.parent / "text.mereclip").read_bytes())
assert package == fixture, "The download must preserve the exact selected package"
assert set(package) == {"version", "clip", "payloads"}
assert package["version"] == 1
clip = package["clip"]
assert clip["source"] == {"owner": "source-app", "entity": "selected", "revision": None}
assert clip["source_evidence"] == {
    "kind": "passage", "pack_sources": [], "source_tags": ["coffee"], "tag_asserter": None,
}
assert clip["tags"] == []
assert len(clip["parts"]) == len(package["payloads"]) == len(clip["anchors"]) == 1
part = clip["parts"][0]
payload = package["payloads"][0]
body = bytes(payload["bytes"])
assert body == "café".encode()
body_id = "blake3:" + blake3(body).hexdigest()
assert payload["id"] == part["manifest"] == body_id
assert part["name"] == "body" and part["role"] == "selected-body"
assert part["dependencies"] == []
anchor = clip["anchors"][0]
original = "private prefix; café; private suffix".encode()
original_id = blake3(original).hexdigest()
selector = anchor["selector"]
assert anchor["source"] == selector["document_address"] == "mere://knot/document/coffee"
assert anchor["representation"] == selector["source_hash"] == original_id
assert anchor["units"] == "utf8-bytes"
assert selector["byte_span"] == {"start": 16, "end": 21}
assert original[16:21] == body == selector["exact_quote"].encode()
assert b"private prefix" not in wire and b"private suffix" not in wire
assert all(p["role"] == "selected-body" for p in clip["parts"])
assert all(k not in package and k not in clip for k in ["private_backing", "provenance", "custody", "seal"])
retained_record = json.dumps({"clip": clip, "private_backing": []}, ensure_ascii=False, separators=(",", ":")).encode()
assert blake3(retained_record).hexdigest() == record_id
shutil.copyfile(actual_path, receipt_dir / "downloaded-coffee.mereclip")
proof = {
    "actual_download_path": str(actual_path),
    "actual_filename": actual_path.name,
    "expected_media_type": "application/vnd.mere.clip-package+json",
    "wire_bytes": len(wire),
    "wire_blake3": blake3(wire).hexdigest(),
    "wire_sha256": hashlib.sha256(wire).hexdigest(),
    "body_blake3": body_id,
    "retained_record_blake3": record_id,
    "anchor_source_blake3": original_id,
    "package_equals_fleece_fixture": True,
    "exact_selected_body_source_tag_anchor_verified": True,
    "private_backing_and_custody_excluded": True,
}
(receipt_dir / "download-verification.json").write_text(json.dumps(proof, indent=2) + "\n", encoding="utf-8")
print(json.dumps(proof, indent=2))
