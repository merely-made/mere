#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Reproduce the Windows Dramatis app's synthetic custody and native captures.
No actual vault is opened. Deletes only marker-identified fixture runtime data.
"""
from pathlib import Path
import argparse, hashlib, json, os, shutil, subprocess, time
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "support/receipts/dramatis-app"
TARGET = Path("C:/t/cargo-targets/mere")
CARGO = ["cargo", "+1.98.1"]
records = []
JOBS = 1
def run(name, args, env=None, expected=0):
    with (OUT / (name + ".log")).open("w", encoding="utf-8") as log:
        result = subprocess.run(args, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    records.append({"name":name,"command":args,"exit_code":result.returncode,"expected":expected})
    print(name, result.returncode, flush=True)
    assert result.returncode == expected, name
    return (OUT / (name + ".log")).read_text(encoding="utf-8", errors="replace")
def cargo(*args):
    return CARGO + list(args) + ["--locked", "--offline", "--target-dir", str(TARGET), "-j" + str(JOBS)]
def remove_runtime(root):
    absolute = root.resolve()
    assert absolute.parent == OUT.resolve() and root.name.startswith("runtime-")
    assert (root / ".dramatis-generated").read_text() == "synthetic Dramatis custody fixture"
    shutil.rmtree(root)
def fixture(mode, capture=True):
    root = OUT / ("runtime-" + mode)
    assert not root.exists(), "retain unexpected runtime for inspection"
    endpoint = r"\\.\pipe\dramatis-app-receipt-" + mode
    env = dict(os.environ, GRAPHSHELL_APP_ENDPOINT=endpoint, DRAMATIS_RECEIPT_ENDPOINT=endpoint, DRAMATIS_RECEIPT_MODE=mode)
    flags = getattr(subprocess,"CREATE_NO_WINDOW",0)
    with (OUT / ("host-" + mode + ".log")).open("w") as host_log:
        host = subprocess.Popen([str(TARGET / "debug/dramatis-review-host.exe"), endpoint, str(root), mode],cwd=ROOT,stdout=host_log,stderr=subprocess.STDOUT,creationflags=flags)
        try:
            deadline = time.monotonic() + 60
            while not (root / "ready").exists():
                assert host.poll() is None, "fixture host exited"
                assert time.monotonic() < deadline, "fixture readiness"
                time.sleep(.1)
            run("app-wire-"+mode, cargo("test","-p","dramatis-app","--test","state")+["--","--ignored","--exact","app_wire_receipt","--nocapture"], env)
            if capture:
                for size in ["860x900", "420x900"]:
                    filename = mode + "-" + size + ".png"
                    shown = dict(env,DRAMATIS_CAPTURE=str(OUT/filename),DRAMATIS_CAPTURE_MODE=mode,DRAMATIS_CAPTURE_SIZE=size)
                    run("window-" + mode + "-" + size, [str(TARGET / "debug/dramatis-ui.exe")], shown)
                    assert (OUT/filename).is_file()
        finally:
            if host.poll() is None: host.terminate()
            host.wait(timeout=30)
        remove_runtime(root)
def main():
    global JOBS
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build",action="store_true")
    parser.add_argument("--jobs",type=int,default=1,help="Compiler jobs; keep memory available for concurrent lanes")
    args=parser.parse_args()
    assert args.jobs > 0
    JOBS = args.jobs
    OUT.mkdir(parents=True,exist_ok=True)
    if not args.skip_build:
        run("host-build",cargo("build","-p","djinn","--bin","dramatis-review-host","--features","custody-receipt-host,cxf"))
        run("app-build",cargo("build","-p","dramatis-app","--bin","dramatis-ui"))
    run("resident-check",cargo("check","-p","djinn","--bin","djinn","--features","cxf"))
    log = run("green",cargo("test","-p","djinn","--test","quarantine_review","--features","cxf"))
    assert "running 16 tests" in log
    run("app-state",cargo("test","-p","dramatis-app","--test","state"))
    run("interaction",cargo("test","-p","dramatis-app","--test","interaction"))
    run("castellan",cargo("test","-p","castellan","--features","cxf","--lib"))
    source=ROOT/"ports/djinn/src/keeper/review.rs"
    original=source.read_bytes()
    try:
        offset=original.rfind(b"if !self")
        assert offset >= 0
        broken=original[:offset]+original[offset:].replace(b"if !self",b"if false && !self",1)
        assert broken != original
        source.write_bytes(broken)
        log=run("negative-consent",cargo("test","-p","djinn","--test","quarantine_review","--features","cxf")+["--","--exact","cards_are_secret_free_scoped_and_denial_preserves_the_item"],expected=101)
        assert "cards_are_secret_free_scoped_and_denial_preserves_the_item ... FAILED" in log
        assert "error[E" not in log
    finally:
        source.write_bytes(original)
    assert source.read_bytes()==original
    run("restored",cargo("test","-p","djinn","--test","quarantine_review","--features","cxf"))
    run("store-discovery",cargo("test","-p","djinn","--lib","--features","cxf")+["--","--exact","resident::tests::startup_discovers_original_store_after_selected_persona_changes"])
    # Rebuild the receipt host after restoring the production consent gate.
    run("restored-build",cargo("build","-p","djinn","--bin","dramatis-review-host","--features","custody-receipt-host,cxf"))
    run("capture-build",cargo("build","-p","dramatis-app","--bin","dramatis-ui"))
    absent = r"\\.\pipe\dramatis-app-receipt-absent"
    env=dict(os.environ,DRAMATIS_RECEIPT_ENDPOINT=absent,DRAMATIS_RECEIPT_MODE="absent",GRAPHSHELL_APP_ENDPOINT=absent,DRAMATIS_CAPTURE=str(OUT/"absent.png"),DRAMATIS_CAPTURE_MODE="absent")
    run("app-wire-absent",cargo("test","-p","dramatis-app","--test","state")+["--","--ignored","--exact","app_wire_receipt"],env)
    run("window-absent",[str(TARGET/"debug/dramatis-ui.exe")],env)
    fixture("locked"); fixture("unlocked")
    hashes={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in ROOT.glob("ports/dramatis/**/*.rs")}
    owned = subprocess.check_output(["git", "ls-files", "ports/castellan", "ports/djinn", "ports/graphshell", "ports/dramatis"],cwd=ROOT,text=True).splitlines() + ["Cargo.toml", "Cargo.lock"]
    for p in [ROOT/name for name in owned if name.endswith((".rs", ".toml", "Cargo.lock"))]:
        hashes[p.relative_to(ROOT).as_posix()]=hashlib.sha256(p.read_bytes()).hexdigest()
    executables={name:hashlib.sha256((TARGET/"debug"/name).read_bytes()).hexdigest() for name in ["dramatis-ui.exe","dramatis-review-host.exe"]}
    artifacts={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in OUT.iterdir() if p.suffix in [".png",".log"]}
    (OUT/"receipt.json").write_text(json.dumps({"platform":"Windows native Cambium/Genet","base":subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,text=True).strip(),"commands":records,"source_sha256":hashes,"executables_sha256":executables,"artifacts_sha256":artifacts,"restored_consent_sha256":hashlib.sha256(original).hexdigest(),"scope":"synthetic fixtures; no actual vault; native captures require separate visual review; consent gate restored byte-exact"},indent=2)+"\n",encoding="utf-8")
if __name__=="__main__": main()
