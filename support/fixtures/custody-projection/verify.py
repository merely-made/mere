# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Run isolated feature gates; retain dependency and expected-failure evidence."""
import json
from pathlib import Path
import subprocess
import sys

base = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=True)
native = subprocess.check_output(["rustc", "-vV"], text=True).split("host: ")[1].splitlines()[0]


def metadata(name, target):
    manifest = str(base / name / "Cargo.toml")
    raw = subprocess.check_output([
        "cargo", "metadata", "--locked", "--format-version", "1",
        "--manifest-path", manifest, "--filter-platform", target,
    ])
    (out / f"{name}-{target}.json").write_bytes(raw)
    data = json.loads(raw)
    nodes = {node["id"]: node for node in data["resolve"]["nodes"]}
    packages = {package["id"]: package for package in data["packages"]}
    seen = set()

    def walk(package):
        if package in seen:
            return
        seen.add(package)
        for dependency in nodes[package]["deps"]:
            if any(kind["kind"] in (None, "build") for kind in dependency["dep_kinds"]):
                walk(dependency["pkg"])

    walk(data["resolve"]["root"])
    endpoint = next(p for p in seen if packages[p]["name"] == ("djinn" if name in ("custody", "custody-types", "portable") else "graphshell-endpoint"))
    names = sorted({packages[p]["name"] for p in seen})
    direct = [
        {"name": packages[d["pkg"]]["name"], "kinds": d["dep_kinds"]}
        for d in nodes[endpoint]["deps"]
    ]
    report = {"target": target, "normal_build_package_names": names,
              "endpoint_features": nodes[endpoint]["features"],
              "endpoint_direct_dependencies": direct}
    (out / f"{name}-{target}.cone.json").write_text(json.dumps(report, indent=2) + "\n")
    if name in ("custody", "custody-types"):
        assert not set(names) & {"chirograph", "sceno", "scenotime", "hest", "pandect",
                                 "mere-kernel", "mere-alembic", "graphshell"}, names
    if name in ("projection", "provider"):
        assert not {d["name"] for d in direct} & {"eidetic", "chatelaine", "dramatis", "insigne"}, direct
    print(name, target, len(names), "normal/build package names", flush=True)


for name, forbidden in [("custody", "ProjectionCatalog"), ("custody-types", "app_client"), ("projection", "custody_client"),
                        ("portable", "app_client"), ("offer", "app_client"), ("provider", "custody_client"), ("host", None)]:
    manifest = str(base / name / "Cargo.toml")
    metadata(name, native)
    subprocess.run(["cargo", "test", "--locked", "--manifest-path", manifest], check=True)
    if forbidden:
        result = subprocess.run([
            "cargo", "check", "--locked", "--manifest-path", manifest,
            "--features", "forbidden-api",
        ], capture_output=True, text=True)
        (out / f"{name}-forbidden-api.log").write_text(result.stdout + result.stderr)
        assert result.returncode and "E0432" in result.stderr and forbidden in result.stderr, result.stderr
        print(name, "expected missing", forbidden, "compile failure", flush=True)

for name in ["portable", "offer", "host"]:
    target = "wasm32-unknown-unknown"
    metadata(name, target)
    subprocess.run(["cargo", "check", "--locked", "--manifest-path",
                    str(base / name / "Cargo.toml"), "--target", target], check=True)
