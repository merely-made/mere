# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Windows-only disposable Velopack qualification. Never installs an app.

Run with Python 3.11+, Rust and .NET 8 runtime. The vpk 1.2.161 NuGet tools
must be extracted beneath WORK/vpk. WORK is a named subdirectory of Mere's
approved Cargo target; durable receipts are copied separately by the reviewer.
"""
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import zipfile

REPO = Path(__file__).resolve().parents[2]
TARGET = Path("C:/t/cargo-targets/mere")
WORK = TARGET / "luggage-comparison"
EXE = "merely-release-comparison.exe"
VPK = WORK / "vpk/tools/net8.0/any/vpk.dll"
RESULT = {}


def run(args, name, *, env=None, timeout=180):
    completed = subprocess.run(list(map(str, args)), cwd=REPO, env=env,
                               capture_output=True, text=True, timeout=timeout,
                               creationflags=subprocess.CREATE_NO_WINDOW)
    (WORK / (name + ".log")).write_text(completed.stdout + completed.stderr, encoding="utf-8")
    if completed.returncode:
        raise RuntimeError(f"{name}: exit {completed.returncode}; see its log")
    return completed.stdout


def build(version):
    env = dict(os.environ, LUGGAGE_COMPARISON_VERSION=version)
    run(["cargo", "build", "--locked", "--manifest-path", "support/luggage-comparison/Cargo.toml",
         "--bin", "merely-release-comparison", "--target-dir", TARGET, "--jobs", "1"],
        "build-" + version, env=env)
    inputs = WORK / "package-input"
    inputs.mkdir(exist_ok=True)
    shutil.copy2(TARGET / "debug" / EXE, inputs / EXE)
    (inputs / "fixture-note.txt").write_text("original note " + version, encoding="utf-8")
    # Identical non-executable payload makes extraction long enough to reliably
    # hold a directory handle during the controlled replacement-failure probe.
    payload = inputs / "payload.bin"
    if not payload.exists():
        payload.write_bytes(os.urandom(16 * 1024 * 1024))
    run(["dotnet", VPK, "pack", "--packId", "Merely.ReleaseComparison", "--packVersion", version,
         "--packDir", inputs, "--mainExe", EXE, "--outputDir", WORK / "feed"], "pack-" + version)
    portable = next((WORK / "feed").glob("*-Portable.zip"))
    shutil.copy2(portable, WORK / ("portable-" + version + ".zip"))
    print("packed " + version, flush=True)


def extract(version, name):
    root = WORK / name
    root.mkdir()
    with zipfile.ZipFile(WORK / ("portable-" + version + ".zip")) as archive:
        archive.extractall(root)
    assert (root / "current" / EXE).exists(), "unexpected portable layout"
    return root


def apply(root, name):
    run([root / "current" / EXE, "apply", WORK / "offline-feed"], name)
    deadline = time.monotonic() + 45
    while time.monotonic() < deadline:
        try:
            info = json.loads(run([root / "current" / EXE, "info"], name + "-version", timeout=5))
            if info["build_version"] == "0.2.0":
                return info
        except (OSError, RuntimeError, json.JSONDecodeError):
            pass
        time.sleep(0.1)
    raise RuntimeError("apply did not produce v0.2")


def portable_and_stage():
    root = extract("0.1.0", "portable-original")
    moved = WORK / "portable moved"
    root.rename(moved)
    root = moved
    (root / "user-setting.txt").write_text("preserve me", encoding="utf-8")
    RESULT["relocated_portable_check"] = json.loads(run([root / "current" / EXE, "check", WORK / "feed"], "portable-check"))
    RESULT["download"] = json.loads(run([root / "current" / EXE, "download", WORK / "feed"], "portable-download"))
    (WORK / "feed").rename(WORK / "feed-hidden")
    try:
        RESULT["offline_fresh_process_apply"] = apply(root, "portable-apply")
    finally:
        (WORK / "feed-hidden").rename(WORK / "feed")
    RESULT["outside_current_state_preserved"] = (root / "user-setting.txt").read_text() == "preserve me"
    RESULT["artifacts"] = {p.name: p.stat().st_size for p in (WORK / "feed").glob("*.nupkg")}


def staged_tamper():
    root = extract("0.1.0", "portable-stage-tamper")
    run([root / "current" / EXE, "download", WORK / "feed"], "tamper-download")
    pending = next((root / "packages").glob("*0.2.0-full.nupkg"))
    rewritten = pending.with_suffix(".rewrite")
    changed = False
    with zipfile.ZipFile(pending) as old, zipfile.ZipFile(rewritten, "w", zipfile.ZIP_DEFLATED) as new:
        for member in old.infolist():
            data = old.read(member.filename)
            if member.filename.endswith("fixture-note.txt"):
                data = b"changed after download"
                changed = True
            new.writestr(member, data)
    assert changed
    rewritten.replace(pending)
    RESULT["tampered_stage_apply_version"] = apply(root, "tampered-stage-apply")
    RESULT["tampered_stage_asset_accepted"] = (root / "current/fixture-note.txt").read_text() == "changed after download"


def replacement_failure():
    root = extract("0.1.0", "portable-replacement-failure")
    package = next((WORK / "feed").glob("*0.2.0-full.nupkg"))
    output = (WORK / "replacement-failure.log").open("w", encoding="utf-8")
    # The helper is run directly to isolate apply from SDK policy and startup.
    process = subprocess.Popen([str(root / "Update.exe"), "--silent", "apply", "--rootDir", str(root),
                                "--package", str(package), "--norestart"], stdout=output, stderr=output,
                               creationflags=subprocess.CREATE_NO_WINDOW)
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, wintypes.LPVOID,
                                  wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    kernel.CreateFileW.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    handle = None
    locked = None
    backup_seen = False
    deadline = time.monotonic() + 65
    try:
        while process.poll() is None and time.monotonic() < deadline:
            dirs = list((root / "packages/VelopackTemp").glob("tmp_*"))
            if handle is None:
                for directory in dirs:
                    # A new extraction directory starts empty. Holding it open
                    # without FILE_SHARE_DELETE allows extraction but blocks
                    # its final rename; unrelated paths never enter this call.
                    candidate = kernel.CreateFileW(str(directory), 0x80000000, 3, None, 3, 0x02000000, None)
                    if candidate != ctypes.c_void_p(-1).value:
                        handle, locked = candidate, directory
                        break
            backup_seen |= any((d / EXE).exists() for d in dirs if d != locked)
            time.sleep(0.002)
        if process.poll() is None:
            raise RuntimeError("replacement helper exceeded bounded probe timeout")
        RESULT["replacement_failure"] = {
            "directory_handle_injection": handle is not None,
            "backup_seen": backup_seen,
            "exit": process.returncode,
            "current_binary_survived": (root / "current" / EXE).exists(),
            "backup_binary_survived": any((d / EXE).exists() for d in (root / "packages/VelopackTemp").glob("tmp_*")),
        }
    finally:
        if handle is not None:
            kernel.CloseHandle(handle)
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=5)
        output.close()


if __name__ == "__main__":
    assert WORK.resolve().parent == TARGET.resolve()
    assert (WORK / ".generated-output").is_file()
    try:
        build("0.1.0")
        build("0.2.0")
        portable_and_stage()
        print("portable/offline apply complete", flush=True)
        staged_tamper()
        print("staged asset tamper complete", flush=True)
        replacement_failure()
    finally:
        (WORK / "velopack.json").write_text(json.dumps(RESULT, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(RESULT, indent=2), flush=True)
