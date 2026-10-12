#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Build fresh Dramatis binaries, pack/sign, and assemble a Luggage feed.

Run from any directory with Python 3.11+. No install, publication or custody.
The default is a release build; --profile dev is a local packaging receipt.
Signing uses cargo-packager's environment variables, never a key in config.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shlex
import subprocess
import sys
import tomllib

PACKAGE = Path(__file__).resolve().parent
ROOT = PACKAGE.parent.parent
MARKER = ".dramatis-packaging.json"


def source_hash():
    names = ["Cargo.lock", "Cargo.toml", "ports/dramatis/Cargo.toml", "ports/dramatis/build.rs"]
    names += [p.relative_to(ROOT).as_posix() for p in (PACKAGE / "src").glob("*.rs")]
    digest = hashlib.sha256()
    for name in sorted(names):
        digest.update(name.encode() + b"\0")
        digest.update((ROOT / name).read_bytes().replace(b"\r\n", b"\n") + b"\0")
    return digest.hexdigest()


def run(command, **kwargs):
    # Credentials stay in cargo-packager's environment, outside argv/logs.
    print(" ".join(str(p) for p in command), flush=True)
    return subprocess.run(command, cwd=ROOT, check=True, **kwargs)


def output(command):
    return subprocess.check_output(command, cwd=ROOT, text=True, encoding="utf-8").strip()


def target_dir(args):
    value = args.target_dir or ("C:/t/cargo-targets/mere" if os.name == "nt" else os.environ.get("CARGO_TARGET_DIR"))
    if not value:
        if os.name == "nt":
            value = "C:/t/cargo-targets/mere"
        else:
            raise ValueError("Set CARGO_TARGET_DIR to this host's existing approved Mere target.")
    result = Path(value).expanduser().resolve()
    if os.name == "nt" and result != Path("C:/t/cargo-targets/mere").resolve():
        raise ValueError("Windows packaging must reuse C:/t/cargo-targets/mere.")
    return result


def cargo(args, *command):
    return ["cargo", "+" + args.toolchain, *command, "--locked", "--target-dir", str(target_dir(args))]


def profile_dir(args):
    return target_dir(args) / ("debug" if args.profile == "dev" else "release")


def output_dir(args):
    parent = target_dir(args) / "dramatis-packaging"
    marker = parent / MARKER
    if parent.exists() and not marker.exists():
        raise ValueError(f"Unowned package directory: {parent}")
    parent.mkdir(parents=True, exist_ok=True)
    if marker.exists():
        assert json.loads(marker.read_text(encoding="utf-8"))["owner"] == "dramatis-packaging"
    else:
        marker.write_text(json.dumps({"owner": "dramatis-packaging", "kind": "generated package output"}) + "\n", encoding="utf-8")
    result = parent / args.profile
    result.mkdir(exist_ok=True)
    return result


def binary(args):
    return profile_dir(args) / ("dramatis-ui.exe" if os.name == "nt" else "dramatis-ui")


def metadata():
    return tomllib.loads((PACKAGE / "Cargo.toml").read_text(encoding="utf-8"))["package"]


def validate_binary(path):
    info = json.loads(output([str(path), "--build-info"]))
    assert info["product"] == "Dramatis"
    assert info["version"] == metadata()["version"], "binary version differs from Cargo.toml"
    assert info["source_sha256"] == source_hash(), "binary does not match current source/lock"
    assert output([str(path), "--version"]) == "Dramatis " + info["version"]
    return info


def prepare(args):
    if os.environ.get("CARGO_BUILD_TARGET"):
        raise ValueError("Package natively on each host; unset CARGO_BUILD_TARGET.")
    initial = source_hash()
    run(cargo(args, "build", "-p", "dramatis-app", "--bin", "dramatis-ui", "--profile", args.profile, "-j", str(args.jobs)))
    assert source_hash() == initial, "source/lock changed during package build; rerun after its owner finishes"
    info = validate_binary(binary(args))
    info.update(source_revision=output(["git", "rev-parse", "HEAD"]), profile=args.profile,
                runtime_dependency="existing user-scoped Djinn resident")
    (output_dir(args) / "release-info.json").write_text(json.dumps(info, indent=2) + "\n", encoding="utf-8")
    return info


def camel(name):
    head, *tail = name.split("-")
    return head + "".join(s.title() for s in tail)


def config_keys(value):
    if isinstance(value, dict):
        return {camel(k): config_keys(v) for k, v in value.items()}
    if isinstance(value, list):
        return [config_keys(v) for v in value]
    return value


def config(args):
    out = output_dir(args)
    package = metadata()
    cfg = config_keys(package["metadata"]["packager"])
    cfg.update(name=package["name"], version=package["version"], description=package["description"],
               binariesDir=str(profile_dir(args)), outDir=str(out / args.format),
               licenseFile=str(ROOT / "LICENSE"))
    cfg["icons"] = [str(PACKAGE / name) for name in package["metadata"]["packager"]["icons"]]
    cfg["resources"] = [str(PACKAGE / "package-requirements.txt"), str(ROOT / "LICENSE"), str(ROOT / "LICENSES.md"), str(out / "release-info.json")]
    command = [sys.executable, str(Path(__file__).resolve()), "prepare", "--profile", args.profile,
               "--jobs", str(args.jobs), "--toolchain", args.toolchain, "--target-dir", str(target_dir(args))]
    cfg["beforePackagingCommand"] = {"script": subprocess.list2cmdline(command) if os.name == "nt" else shlex.join(command), "dir": str(ROOT)}
    path = out / "packager.json"
    path.write_text(json.dumps(cfg, indent=2) + "\n", encoding="utf-8")
    return path


def pack(args):
    if not os.environ.get("CARGO_PACKAGER_SIGN_PRIVATE_KEY"):
        raise ValueError("Set CARGO_PACKAGER_SIGN_PRIVATE_KEY outside the repository before packing.")
    key = Path(os.environ["CARGO_PACKAGER_SIGN_PRIVATE_KEY"]).expanduser().resolve()
    if not key.is_file() or key.is_relative_to(ROOT):
        raise ValueError("Use an existing signing-key file outside the source repository.")
    # Publishing needs committed code. Concurrent docs/media never enter the app.
    if args.profile == "release":
        dirty = output(["git", "diff", "--name-only", "HEAD"]).splitlines()
        dirty += output(["git", "ls-files", "--others", "--exclude-standard"]).splitlines()
        assert not any(p.endswith((".rs", "Cargo.toml", "Cargo.lock")) for p in dirty), "commit compiled source before a release build"
    run(["cargo", "packager", "--config", str(config(args)), "--formats", args.format])
    info = validate_binary(binary(args))
    out = output_dir(args) / args.format
    suffix = {"nsis": "-setup.exe", "app": ".app.tar.gz", "appimage": ".AppImage"}[args.format]
    artifacts = [p for p in out.iterdir() if p.is_file() and p.name.endswith(suffix) and info["version"] in p.name]
    assert len(artifacts) == 1, f"Expected one {suffix} artifact, found {artifacts}"
    artifact = artifacts[0]
    assert Path(str(artifact) + ".sig").is_file(), "packager did not sign the artifact"
    manifest = out / "luggage.json"
    command = cargo(args, "run", "-p", "luggage", "--bin", "luggage-manifest", "-j", str(args.jobs))
    command += ["--", "--artifact", str(artifact), "--version", info["version"], "--format", args.format, "--out", str(manifest)]
    if args.artifact_url:
        if not args.artifact_url.startswith("https://"):
            raise ValueError("Published artifact URLs must use HTTPS.")
        command += ["--url", args.artifact_url]
    run(command)
    run(["cargo", "packager", "signer", "sign", str(manifest)])
    print(f"Signed {args.profile} package and feed: {out}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["prepare", "pack"])
    parser.add_argument("--profile", choices=["dev", "release"], default="release")
    parser.add_argument("--format", choices=["nsis", "app", "appimage"], default={"Windows": "nsis", "Darwin": "app", "Linux": "appimage"}[platform.system()])
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--target-dir")
    parser.add_argument("--toolchain", default="1.98.1")
    parser.add_argument("--artifact-url")
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    if args.artifact_url and not args.artifact_url.startswith("https://"):
        parser.error("Published artifact URLs must use HTTPS.")
    expected = {"Windows": "nsis", "Darwin": "app", "Linux": "appimage"}[platform.system()]
    if args.format != expected:
        parser.error(f"Qualify natively on the {args.format} host; this host uses {expected}.")
    (prepare if args.action == "prepare" else pack)(args)


if __name__ == "__main__":
    main()
