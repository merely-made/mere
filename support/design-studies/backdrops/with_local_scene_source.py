#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Run Cargo in an app checkout against one local portable-scene cohort.

Usage: python3 with_local_scene_source.py MERE_CHECKOUT test -p knot-readings
The temporary Cargo patch is deliberately outside the app's tracked manifest.
Cargo may update that checkout's lockfile, just as with an ordinary repin.
"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile

mere = Path(sys.argv[1]).resolve()
members = {name: mere / "crates/cambium/scenes" / name
           for name in ["sceno", "scenomise", "scenograph", "scenotime"]}
for member in members.values():
    if not (member / "Cargo.toml").is_file():
        raise SystemExit(f"missing scene member: {member}")
with tempfile.TemporaryDirectory(prefix="portable-scenes-") as temporary:
    patch = Path(temporary) / "source.toml"
    patch.write_text('[patch."https://github.com/merely-made/mere.git"]\n' + "\n".join(
        f'{name} = {{ path = {json.dumps(str(path))} }}' for name, path in members.items()) + "\n")
    raise SystemExit(subprocess.run(["cargo", "--config", str(patch), *sys.argv[2:]]).returncode)
