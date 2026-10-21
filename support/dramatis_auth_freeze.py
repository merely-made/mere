#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Freeze native authentication fixtures and record their repository inputs.

Run on the acceptance machine after building the app and the ignored test host.
The executables remain private; build-inputs.json is public receipt material.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import shlex
import shutil
import subprocess


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["repo", "artifacts", "build-dir", "host-build", "output"]:
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--toolchain", default="1.99.1")
    args = parser.parse_args()
    repo = args.repo.resolve()
    output = args.output.resolve()
    output.mkdir(mode=0o700)
    assert platform.system() in {"Linux", "Darwin"}, "unqualified native format"

    def run(*command, cwd=repo):
        return subprocess.check_output(command, cwd=cwd, text=True).strip()

    cargo = str(Path.home() / ".cargo/bin/cargo")
    compiler = str(Path.home() / ".cargo/bin/rustc")
    if platform.system() == "Darwin":
        bundle = output / "Dramatis.app"
        app = bundle / "Contents/MacOS/dramatis-ui"
        app.parent.mkdir(parents=True)
        with (bundle / "Contents/Info.plist").open("wb") as stream:
            plistlib.dump({"CFBundleIdentifier": "made.merely.dramatis.authentication.acceptance",
                          "CFBundleName": "Dramatis authentication test",
                          "CFBundleExecutable": "dramatis-ui", "CFBundlePackageType": "APPL",
                          "NSHighResolutionCapable": True}, stream)
    else:
        app = output / "dramatis-ui"
    shutil.copy2(args.artifacts / "debug/dramatis-ui", app)
    artifacts = [json.loads(line) for line in args.host_build.read_text().splitlines()
                 if line.startswith("{")]
    artifacts = [a for a in artifacts if a.get("reason") == "compiler-artifact"]
    host_source = next(a["executable"] for a in artifacts
                       if a["target"]["name"] == "passphrase_change" and a.get("executable"))
    host = output / "authentication-host"
    shutil.copy2(host_source, host)
    if platform.system() == "Darwin":
        for path in [bundle, host]:
            run("/usr/bin/codesign", "--force", "--sign", "-", "--timestamp=none", str(path))
            run("/usr/bin/codesign", "--verify", "--strict", str(path))

    info = json.loads(run(str(app), "--build-info"))
    names = {"Cargo.lock", "Cargo.toml", "ports/dramatis/Cargo.toml", "ports/dramatis/build.rs"}
    names.update(str(p.relative_to(repo)) for p in (repo / "ports/dramatis/src").glob("*.rs"))
    digest = hashlib.sha256()
    for name in sorted(names):
        digest.update(name.encode() + b"\0" + (repo / name).read_text().replace("\r\n", "\n").encode() + b"\0")
    assert digest.hexdigest() == info["source_sha256"], "app source digest mismatch"

    files = set(names)
    files.update(["support/dramatis_auth_receipts.py", "support/dramatis_auth_freeze.py",
                  "ports/djinn/tests/passphrase_change.rs"])
    depfiles = {args.artifacts / "debug/dramatis-ui.d"}
    for artifact in artifacts:
        manifest = Path(artifact["manifest_path"]).resolve()
        if manifest.is_relative_to(repo):
            files.add(str(manifest.relative_to(repo)))
        for filename in artifact["filenames"]:
            stem = Path(filename).stem.removeprefix("lib")
            depfile = args.build_dir / "debug/deps" / (stem + ".d")
            if depfile.is_file():
                depfiles.add(depfile)
    for depfile in depfiles:
        rule = depfile.read_text().split(": ", 1)[1].split("\n\n", 1)[0].replace("\\\n", "")
        for name in shlex.split(rule):
            path = Path(name)
            path = (path if path.is_absolute() else repo / path).resolve()
            if path.is_relative_to(repo) and path.is_file():
                files.add(str(path.relative_to(repo)))
    tracked = set(run("git", "ls-files").splitlines())
    assert files <= tracked, ("untracked repository build inputs", sorted(files - tracked))

    metadata = json.loads(run(cargo, "+" + args.toolchain, "metadata", "--locked", "--offline", "--format-version", "1"))
    suppliers = {}
    actual_packages = {a["package_id"] for a in artifacts}
    external_paths = []
    for package in metadata["packages"]:
        source = package["source"]
        manifest = Path(package["manifest_path"])
        if source and source.startswith("git+"):
            root = run("git", "rev-parse", "--show-toplevel", cwd=manifest.parent)
            if root not in suppliers:
                head = run("git", "rev-parse", "HEAD", cwd=root)
                assert head == source.split("#")[-1], (root, head, source)
                assert not run("git", "diff", "--name-only", "HEAD", "--", "*.rs", "*.toml",
                               "Cargo.lock", "*.wgsl", "*.c", "*.h", "*.cpp", "*.hpp", cwd=root), root
                suppliers[root] = {"source": source, "head": head, "packages": [], "tracked_code_changes": []}
            suppliers[root]["packages"].append(package["name"])
        elif source is None and not manifest.is_relative_to(repo):
            assert package["id"] not in actual_packages, ("unrecorded external source", package["name"])
            external_paths.append({"name": package["name"], "manifest": str(manifest)})
    for package in ["dramatis-app", "djinn"]:
        tree = run(cargo, "+" + args.toolchain, "tree", "--locked", "--offline", "-p", package, "--edges", "normal,build")
        (output / (package + "-tree.txt")).write_text(tree + "\n")
        if package == "dramatis-app":
            assert "castellan v" not in tree, "secret custody entered the app dependency cone"
    binaries = {}
    for path in [app, host]:
        binaries[str(path.relative_to(output))] = {
            "sha256": sha(path), "bytes": path.stat().st_size,
            "format": run("file", "-b", str(path)),
            "signing": "ad-hoc; timestamp none" if platform.system() == "Darwin" else "native ELF; no signing step",
        }
    value = {"source_commit": run("git", "rev-parse", "HEAD"), "build_info": info,
             "files": {name: sha(repo / name) for name in sorted(files)}, "binaries": binaries,
             "git_suppliers": suppliers, "uncompiled_external_path_packages": external_paths,
             "rustc": run(compiler, "+" + args.toolchain, "-Vv"), "depfiles_read": len(depfiles),
             "supplier_scope": "Resolved Git checkouts; tracked Rust/TOML/lock/WGSL/C/C++ only. Untracked and asset-only supplier changes excluded.",
             "environment": {key: os.environ.get(key) for key in ["CARGO_TARGET_DIR", "CARGO_BUILD_BUILD_DIR", "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG"]}}
    (output / "build-inputs.json").write_text(json.dumps(value, indent=2) + "\n")
    print(json.dumps({"source": value["source_commit"], "files": len(files), "git_suppliers": len(suppliers), "binaries": binaries}))


if __name__ == "__main__":
    main()
