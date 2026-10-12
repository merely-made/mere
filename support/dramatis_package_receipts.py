#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Windows package qualification using warm debug builds and ephemeral keys.
Never installs, publishes, opens custody or operates another app's window.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
TARGET = Path("C:/t/cargo-targets/mere")
OUT = ROOT / "support/receipts/dramatis-packaging"
GENERATED = TARGET / "dramatis-packaging"
records = []


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(name, command, env=None):
    path = OUT / (name + ".log")
    with path.open("w", encoding="utf-8") as log:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    records.append({"name": name, "command": command, "exit_code": result.returncode})
    print(name, result.returncode, flush=True)
    assert result.returncode == 0, f"See {path}"


def main():
    assert os.name == "nt", "Other native hosts remain pending"
    OUT.mkdir(parents=True, exist_ok=True)
    spec = importlib.util.spec_from_file_location("dramatis_package", ROOT / "ports/dramatis/package.py")
    package = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(package)
    args = type("Args", (), {"target_dir": str(TARGET), "profile": "dev"})()
    package.output_dir(args)
    keys = GENERATED / "dev/signing-input"
    assert not keys.exists(), "Do not replace existing signing material"
    keys.mkdir()
    (keys / ".dramatis-generated").write_text("synthetic Dramatis package keys", encoding="utf-8")
    private = keys / "package.key"
    other = keys / "other.key"
    try:
        for key in [private, other]:
            # The signer can print key material; suppress its complete output.
            result = subprocess.run(["cargo", "packager", "signer", "generate", "--ci", "--password", "", "--path", str(key)], cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            assert result.returncode == 0 and key.is_file()
        public = OUT / "synthetic-package.pub"
        public_other = OUT / "synthetic-other.pub"
        shutil.copyfile(Path(str(private) + ".pub"), public)
        shutil.copyfile(Path(str(other) + ".pub"), public_other)
        env = dict(os.environ, CARGO_PACKAGER_SIGN_PRIVATE_KEY=str(private), CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD="")
        # Only the local gate's child gets the ephemeral signing identity.
        run("pack", [sys.executable, str(ROOT / "ports/dramatis/package.py"), "pack", "--profile", "dev", "--jobs", "1"], env)
        feed = GENERATED / "dev/nsis"
        (feed / ".synthetic-package-receipt").write_text("test-signed Dramatis package fixture", encoding="utf-8")
        manifest = json.loads((feed / "luggage.json").read_text(encoding="utf-8"))
        artifacts = list(feed.glob("*-setup.exe"))
        assert len(artifacts) == 1
        artifact = artifacts[0]
        before = {p.name: digest(p) for p in [artifact, Path(str(artifact) + ".sig"), feed / "luggage.json", feed / "luggage.json.sig"]}
        scratch = GENERATED / "dev/extracted"
        assert not scratch.exists(), "Do not replace another extraction"
        scratch.mkdir()
        (scratch / ".dramatis-generated").write_text("synthetic Dramatis installer extraction", encoding="utf-8")
        try:
            run("extract", ["7z", "x", "-y", "-o" + str(scratch), str(artifact)])
            entries = [p.relative_to(scratch).as_posix() for p in scratch.rglob("*") if p.is_file()]
            forbidden = [p for p in entries if any(s in p.lower() for s in ["djinn.exe", "dramatis-review-host", "personae-vault", ".key", "luggage.json"])]
            assert not forbidden, forbidden
            apps = list(scratch.rglob("dramatis-ui.exe"))
            assert len(apps) == 1
            info = package.validate_binary(apps[0])
            run("version", [str(apps[0]), "--version"])
            capture_env = dict(os.environ, GRAPHSHELL_APP_ENDPOINT=r"\\.\pipe\dramatis-package-receipt-absent",
                               DRAMATIS_CAPTURE=str(OUT / "packaged-absent.png"), DRAMATIS_CAPTURE_MODE="absent")
            run("native-extracted", [str(apps[0])], capture_env)
            (OUT / "contents.json").write_text(json.dumps({"package_entries": entries, "build_info": info}, indent=2) + "\n", encoding="utf-8")
        finally:
            # Only marker-owned generated extraction, after its app exits.
            assert scratch.resolve().parent == (GENERATED / "dev").resolve()
            assert (scratch / ".dramatis-generated").read_text() == "synthetic Dramatis installer extraction"
            shutil.rmtree(scratch)
        gate_env = dict(os.environ, DRAMATIS_PACKAGE_FEED=str(feed), DRAMATIS_PACKAGE_PUBLIC_KEY_FILE=str(public), DRAMATIS_PACKAGE_OTHER_PUBLIC_KEY_FILE=str(public_other))
        command = ["cargo", "+1.98.1", "test", "--locked", "--offline", "-p", "dramatis-app", "--test", "package_feed", "--target-dir", str(TARGET), "-j1", "--", "--ignored", "--exact", "real_signed_package_and_tamper_controls", "--nocapture"]
        run("signed-feed", command, gate_env)
        assert all(digest(feed / name) == sha for name, sha in before.items()), "tamper controls must restore every feed byte"
        run("app-controls", ["cargo", "+1.98.1", "test", "--locked", "--offline", "-p", "dramatis-app", "--test", "state", "--test", "interaction", "--target-dir", str(TARGET), "-j1"])
        receipt = {"platform": "windows-x86_64", "profile": "dev", "synthetic_signing_only": True,
                   "native_host_qualification": {"Windows": "passed", "Fedora": "pending; wait until reachable", "macOS Intel": "pending; wait until reachable", "macOS ARM": "pending; wait until reachable"},
                   "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                   "compiled_source_sha256": package.source_hash(), "commands": records, "manifest": manifest,
                   "package_path": str(artifact), "package_feed_sha256": before,
                   "scope": "test-signed debug installer; extracted native client; signed manifest/artifact, publisher, staging tamper gates; no installation, publication or actual vault"}
        receipt["artifacts_sha256"] = {p.name: digest(p) for p in OUT.iterdir() if p.is_file() and p.name != "receipt.json"}
        (OUT / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    finally:
        assert keys.resolve().parent == (GENERATED / "dev").resolve()
        assert (keys / ".dramatis-generated").read_text() == "synthetic Dramatis package keys"
        # Delete only these generated fixture keys, never a configured release key.
        for name in ["package.key", "package.key.pub", "other.key", "other.key.pub", ".dramatis-generated"]:
            path = keys / name
            if path.exists():
                path.unlink()
        keys.rmdir()


if __name__ == "__main__":
    main()
