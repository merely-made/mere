#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Verify shared attachment receipts using the existing link evidence checks."""
from __future__ import annotations

import argparse
import importlib.util
from pathlib import Path
import sys

BASE = Path(__file__).resolve().parent.parent / "link-model-ordinary-paths-20261017" / "verify.py"
SPEC = importlib.util.spec_from_file_location("link_evidence", BASE)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)

SEED = "p4_tree_link_attachments"
REOPEN = "p4_tree_link_attachments_reopen"
PREFIX = "ordinary_link_attachments_"
SEED_CAPTURES = {PREFIX + name for name in (
    "flat", "response", "tilted", "rounded", "circle", "suggestion", "lifted", "lifted_selected",
)}
REOPEN_CAPTURES = {PREFIX + name for name in ("reopened_default", "reopened_tilted")}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("qualification_root", type=Path)
    root = parser.parse_args().qualification_root.resolve()
    revision, records = CHECK.verify_build_manifest(root)
    for name, tokens in (("build", ("cargo", "build")), ("bindgen", ("bindgen",))):
        CHECK.verify_command_record(root, records[name], revision, True, tokens)
    canvas = CHECK.verify_cargo_qualification(root, records["canvas"], revision, "pictograph")
    graphshell = CHECK.verify_cargo_qualification(root, records["graphshell"], revision, "graphshell")
    seed = CHECK.verify_browser_scenario(root, SEED, SEED_CAPTURES)
    reopen = CHECK.verify_browser_scenario(root, REOPEN, REOPEN_CAPTURES)
    CHECK.verify_source_comparison(root, seed, reopen)
    print(f"shared attachment evidence verified; source {revision}")
    print(f"Pictograph passes: {canvas}; Graphshell passes: {graphshell}")
    print("8 seed and 2 reopen PNGs; zero scenario errors/gate failures; source observations equal")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except CHECK.VerificationError as exc:
        print(f"verification failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
