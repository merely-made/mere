#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Local two-process composition experiment with current executable provenance."""

import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys


def invoke(args, root, timeout=180):
    return subprocess.run(args, cwd=root, text=True, capture_output=True,
                          check=True, timeout=timeout)


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: python3 run.py RETINUE_PEER NEW_OUTPUT_DIRECTORY")
    here = Path(__file__).resolve().parent
    root = here.parents[4]
    peer = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    peer_before = hashlib.sha256(peer.read_bytes()).hexdigest()
    build = invoke(["cargo", "build", "--locked", "--offline", "-p", "mere-mesh",
                    "--features", "admitted-ingress", "--example", "inventory_ingress",
                    "--message-format=json"], root, timeout=600)
    (output / "build.log").write_text(build.stderr)
    artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")]
    executables = [item["executable"] for item in artifacts
                   if item.get("reason") == "compiler-artifact"
                   and item.get("target", {}).get("name") == "inventory_ingress"
                   and item.get("executable")]
    if len(executables) != 1:
        raise RuntimeError("Cargo did not report one current composition executable")
    executable = Path(executables[0])
    reports = []
    for name in ["composition", "replay"]:
        path = output / (name + ".json")
        result = invoke([str(executable), str(peer), str(path)], root)
        (output / (name + ".log")).write_text(result.stdout + result.stderr)
        reports.append(json.loads(path.read_text()))
    semantic_fields = ["source", "deliveries", "preservation", "candidate_count",
                       "receiver_inventory_before_hex", "receiver_inventory_after_hex",
                       "owner", "permission_refusals", "invalid_records", "framing_refusals", "scope"]
    for field in semantic_fields:
        if reports[0][field] != reports[1][field]:
            raise RuntimeError("independent-process semantic difference: " + field)
    if peer_before != hashlib.sha256(peer.read_bytes()).hexdigest():
        raise RuntimeError("peer executable changed during composition")
    files = [*sorted(here.glob("*.rs")), here / "run.py",
             *sorted((root / "crates/mesh/mesh/src/ingress").glob("*.rs")),
             *sorted((root / "crates/stickleback/src/inventory").glob("*.rs")),
             root / "crates/murm/transport/src/p2panda_transport/bounded_accept.rs",
             root / "crates/murm/transport/src/p2panda_transport.rs",
             root / "crates/murm/transport/src/lib.rs",
             root / "crates/mesh/mesh/src/lib.rs",
             root / "ports/distillery/src/resident.rs",
             root / "ports/distillery/tests/record_ingress.rs",
             root / "Cargo.lock", root / "Cargo.toml",
             root / "crates/mesh/mesh/Cargo.toml", root / "ports/distillery/Cargo.toml"]
    metadata = {
        "schema": "mere.exact-inventory-admitted-provenance/v1",
        "base_revision": invoke(["git", "rev-parse", "HEAD"], root).stdout.strip(),
        "source_dirty": bool(invoke(["git", "status", "--porcelain"], root).stdout),
        "toolchain": invoke(["rustc", "-Vv"], root).stdout,
        "platform": platform.platform(),
        "source_sha256": {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest()
                          for p in files},
        "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
        "peer_executable_sha256": peer_before,
        "peer_executable_blake3": reports[0]["peer_executable_blake3"],
        "compiled_mere_source_blake3": reports[0]["mere_source_blake3"],
        "compiled_lock_blake3": reports[0]["compiled_lock_blake3"],
        "report_sha256": {name: hashlib.sha256((output / (name + ".json")).read_bytes()).hexdigest()
                          for name in ["composition", "replay"]},
        "independent_process_semantics_equal": True,
        "semantic_fields": semantic_fields,
        "timing_and_actor_observations_compared": False,
    }
    (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print("Two independent composition processes passed; evidence:", output)


if __name__ == "__main__":
    main()
