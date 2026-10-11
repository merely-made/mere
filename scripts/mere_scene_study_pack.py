#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Embed the study's local fixture, derived faces and media in its HTML fragment."""
import base64
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
study = root / "support/design-studies"
path = study / "scene-rules.html"
source = path.read_text()
payloads = {
    "mere-fixture": json.loads((study / "scene-rules.json").read_text()),
    "mere-faces": json.loads((study / "pictograph-faces.json").read_text()),
    "mere-video": "data:video/mp4;base64," + base64.b64encode((study / "motion.mp4").read_bytes()).decode(),
}
for name, payload in payloads.items():
    value = json.dumps(payload, ensure_ascii=True, separators=(",", ":")).replace("<", "\\u003c")
    pattern = rf'(<script type="application/json" id="{name}">).*?(</script>)'
    source, count = re.subn(pattern, lambda match: match[1] + value + match[2], source, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"Expected exactly one {name} payload, found {count}")
if len(source.encode()) >= 1_000_000:
    raise SystemExit("Study exceeds the inline size budget")
path.write_text(source)
print(f"Packed {path.name}: {len(source.encode())} bytes")
