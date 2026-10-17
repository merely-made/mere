#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Run Cargo in an app checkout against one local portable-scene cohort.

Usage: python3 with_local_scene_source.py [--document-blocks] MERE_CHECKOUT test -p knot-readings
The temporary Cargo patch is deliberately outside the app's tracked manifest.
Cargo may update that checkout's lockfile, just as with an ordinary repin.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

arguments = sys.argv[1:]
document_blocks = arguments[:1] == ["--document-blocks"]
if document_blocks:
    arguments = arguments[1:]
if len(arguments) < 2:
    raise SystemExit(__doc__)
mere = Path(arguments[0]).resolve()
members = {name: mere / "crates/cambium/scenes" / name
           for name in ["sceno", "scenomise", "scenograph", "scenotime"]}
if document_blocks:
    members.update({"inker": mere / "crates/inker/inker",
                    "nematic": mere / "crates/nematic/nematic"})
for member in members.values():
    if not (member / "Cargo.toml").is_file():
        raise SystemExit(f"missing scene member: {member}")
with tempfile.TemporaryDirectory(prefix="portable-scenes-") as temporary:
    patch = Path(temporary) / "source.toml"
    patch.write_text('[patch."https://github.com/merely-made/mere.git"]\n' + "\n".join(
        f'{name} = {{ path = {json.dumps(str(path))} }}' for name, path in members.items()) + "\n")
    raise SystemExit(subprocess.run(["cargo", "--config", str(patch), *arguments[1:]]).returncode)
