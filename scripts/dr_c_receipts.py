#!/usr/bin/env python3
"""Run DR-C app receipts against an absent broker and a real Locked keeper.

Each test runs in its own process. The endpoint is never the installed resident.
Raw logs, source hashes, commands and exit codes are retained under the stable
Mere target. Generated synthetic vaults are removed after the host is stopped.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import uuid


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--woodshed", type=Path)
    parser.add_argument("--knot", type=Path)
    parser.add_argument("--turnstone", type=Path)
    parser.add_argument("--target-root", type=Path, default=Path("C:/t/cargo-targets"))
    parser.add_argument("--fallback-control", type=Path, required=True,
                        help="clean isolated Turnstone worktree for the restored-fallback control")
    args = parser.parse_args()
    mere = Path(__file__).resolve().parents[1]
    repos = mere.parent
    woodshed = (args.woodshed or repos / "woodshed").resolve()
    knot = (args.knot or repos / "knot-editor").resolve()
    turnstone = (args.turnstone or repos / "turnstone").resolve()
    targets = args.target_root.resolve()
    output = targets / "mere" / "dr-c-receipts"
    output.mkdir(parents=True, exist_ok=True)
    runtime = output / "runtime"
    if runtime.exists():
        raise RuntimeError("retained runtime exists; check its host owner before cleanup")
    runtime.mkdir()
    (runtime / ".dr-c-generated").write_text("synthetic receipt data\n")
    records = []
    sources = {
        "mere": (mere, ["scripts/dr_c_receipts.py", "ports/castellan/Cargo.toml", "ports/graphshell/Cargo.toml", "ports/djinn/Cargo.toml", "ports/graphshell/src/native/custody.rs", "ports/graphshell/src/native/custody_client.rs", "ports/graphshell/src/native/custody_identity.rs", "ports/graphshell/src/bin/personae_vault.rs", "ports/djinn/src/bin/dr_c_receipt_host.rs", "ports/djinn/src/custody.rs", "ports/djinn/tests/custody_route.rs", "ports/castellan/src/authority.rs", "ports/castellan/src/custody/vault_commands/mod.rs", "ports/castellan/src/custody/vault_commands/certs.rs", "ports/castellan/src/custody/vault_commands/tests.rs", "ports/graphshell/src/profile.rs", "ports/graphshell/tests/dr_c_pending.rs", "ports/signalman/src/authority.rs", "ports/signalman/tests/dr_c_pending.rs", "ports/djinn/src/bin/distillery_installed.rs"]),
        "knot": (knot, ["crates/knot-editor/src/startup.rs", "apps/desktop/src/main.rs", "crates/knot-editor/src/bin/knot_endpoint.rs", "crates/knot-editor/src/bin/knot_sync_host.rs"]),
        "woodshed": (woodshed, ["crates/woodshed-genet/src/storage.rs", "ports/hocket/crates/hocket-genet/src/identity.rs"]),
        "turnstone": (turnstone, ["src/identity.rs", "src/app/tests.rs", "src/bin/g3_receipt.rs", "src/remote_projection.rs"]),
    }
    for repo, files in sources.values():
        files.extend(["Cargo.toml", "Cargo.lock"])
    sources["woodshed"][1].extend(["ports/hocket/Cargo.toml", "ports/hocket/Cargo.lock"])
    source_record = {name: {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(), "files": {file: digest(repo / file) for file in files}} for name, (repo, files) in sources.items()}
    host = None
    finished = False
    control_record = None
    env = os.environ.copy()
    try:
        def run(label, command, cwd=mere, expected=0, checks=None, child_env=None):
            log = output / f"{label}.log"
            started = time.monotonic()
            print(label, flush=True)
            with log.open("w", encoding="utf-8") as stream:
                result = subprocess.run(command, cwd=cwd, env=child_env or env, stdout=stream, stderr=subprocess.STDOUT)
            record = {"label": label, "cwd": str(cwd), "command": [str(value) for value in command], "exit": result.returncode, "expected": expected, "elapsed_seconds": round(time.monotonic() - started, 2), "log": log.name, "sha256": digest(log)}
            executable = Path(command[0])
            if executable.is_file():
                record["executable_sha256"] = digest(executable)
            records.append(record)
            if result.returncode != expected:
                raise RuntimeError(f"{label} exited {result.returncode}; see {log}")
            if checks:
                checks(log.read_text(encoding="utf-8", errors="replace"))
            return log

        def cargo(repo, target, *options):
            return ["cargo", "test", "--locked", "--target-dir", str(targets / target), "-j", "4", *options]

        jobs = [
            ("graphshell", mere, "mere", ["-p", "graphshell", "--test", "dr_c_pending"]),
            ("signalman", mere, "mere", ["-p", "mere-signalman", "--test", "dr_c_pending"]),
            ("knot", knot, "knot-editor", ["-p", "knot-editor", "--lib"]),
            ("knot-desktop", knot, "knot-editor", ["-p", "knot-desktop", "--bin", "knot"]),
            ("woodshed", woodshed, "woodshed", ["-p", "woodshed-genet", "--bin", "woodshed-genet"]),
            ("hocket", woodshed / "ports/hocket", "woodshed", ["-p", "hocket-genet", "--bin", "hocket-genet"]),
            ("turnstone", turnstone, "turnstone", ["-p", "turnstone", "--lib"]),
        ]
        suffix = ".exe" if os.name == "nt" else ""
        binaries = {}
        run("build-host", ["cargo", "build", "--locked", "--target-dir", str(targets / "mere"), "-j", "4", "-p", "djinn", "--features", "custody-receipt-host", "--bin", "dr-c-receipt-host", "--bin", "distillery-installed"])
        run("build-vault-cli", ["cargo", "build", "--locked", "--target-dir", str(targets / "mere"), "-j", "4", "-p", "graphshell", "--bin", "personae-vault"])
        run("custody-route", cargo(mere, "mere", "-p", "djinn", "--test", "custody_route"), cwd=mere)
        run("vault-commands", cargo(mere, "mere", "-p", "castellan", "--features", "keeper", "--lib", "custody::vault_commands"), cwd=mere)
        def test_executable(label, repo, target, options):
            # --message-format=json gives the exact executable, never a glob
            # that could select a stale binary from a different feature set.
            build = run(f"build-{label}", cargo(repo, target, *options, "--no-run", "--message-format=json"), cwd=repo)
            artifacts = []
            for line in build.read_text(encoding="utf-8", errors="replace").splitlines():
                try:
                    item = json.loads(line)
                except ValueError:
                    continue
                if item.get("reason") == "compiler-artifact" and item.get("profile", {}).get("test") and item.get("executable"):
                    artifacts.append(item["executable"])
            if len(artifacts) != 1:
                raise RuntimeError(f"expected one {label} test executable, got {artifacts}")
            return artifacts[0]

        for label, repo, target, options in jobs:
            binaries[label] = test_executable(label, repo, target, options)
        run("build-knot-cli", ["cargo", "build", "--locked", "--target-dir", str(targets / "knot-editor"), "-j", "4", "-p", "knot-editor", "--bin", "knot_endpoint", "--bin", "knot_sync_host"], cwd=knot)
        run("build-signalman-cli", ["cargo", "build", "--locked", "--target-dir", str(targets / "mere"), "-j", "4", "-p", "mere-signalman", "--bin", "mere-signalman-provision"], cwd=mere)
        run("build-turnstone-cli", ["cargo", "build", "--locked", "--target-dir", str(targets / "turnstone"), "-j", "4", "-p", "turnstone", "--bin", "g3_receipt"], cwd=turnstone)

        debug = targets / "mere" / "debug"
        host_bin = debug / ("dr-c-receipt-host" + suffix)
        cli = debug / ("personae-vault" + suffix)
        distillery = debug / ("distillery-installed" + suffix)
        for mode in ["absent", "locked"]:
            token = str(uuid.uuid4())
            endpoint = rf"\\.\pipe\dr-c-receipt-{token}" if os.name == "nt" else str(runtime / (token + ".sock"))
            env["GRAPHSHELL_APP_ENDPOINT"] = endpoint
            env["DR_C_RECEIPT_MODE"] = mode
            product_root = runtime / mode
            product_root.mkdir()
            env["WOODSHED_STATE"] = str(product_root / "practice.json")
            env["WOODSHED_SETTINGS"] = str(product_root / "settings.json")
            if mode == "locked":
                host_root = runtime / "locked-host"
                host_log = (output / "locked-host.log").open("w")
                host = subprocess.Popen([str(host_bin), endpoint, str(host_root), "locked"], stdout=host_log, stderr=subprocess.STDOUT)
                deadline = time.monotonic() + 60
                while not (host_root / "ready").exists():
                    if host.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("receipt host did not become ready")
                    time.sleep(0.1)
                locked_before = {str(path.relative_to(host_root)): digest(path)
                                 for path in host_root.rglob("*") if path.is_file()}
            for label, _, _, _ in jobs:
                def tested(text):
                    if "1 passed; 0 failed" not in text:
                        raise RuntimeError("receipt did not execute exactly one passing test")
                run(f"{mode}-{label}", [binaries[label], "dr_c_", "--ignored", "--nocapture", "--test-threads=1"], checks=tested)
            knot_root = product_root / "knot-cli"
            persona = str(uuid.uuid4())
            for label, binary, arguments, refused in [
                ("endpoint", "knot_endpoint", ["persona-vault", str(knot_root), persona, "4096"], 101),
                ("sync", "knot_sync_host", [str(knot_root), persona, "--pairing-facts"], 1),
            ]:
                log = run(f"{mode}-knot-{label}", [str(targets / "knot-editor/debug" / (binary + suffix)), *arguments], cwd=knot, expected=refused)
                if "pending" not in log.read_text() or knot_root.exists():
                    raise RuntimeError(f"Knot {label} failed to stay pending without creating state")
            before = sorted(str(path.relative_to(product_root)) for path in product_root.rglob("*"))
            station = product_root / "station"
            log = run(f"{mode}-signalman-provision", [str(debug / ("mere-signalman-provision" + suffix)), "--app-endpoint", endpoint, "--station-root", str(station), "--record", "head.json", "--label", "DR-C fixture", "--expires-hours", "1"], expected=1)
            if "pending" not in log.read_text() or station.exists():
                raise RuntimeError("Signalman must provision no credential or record while pending")
            rendered = product_root / "g3.html"
            log = run(f"{mode}-turnstone-g3", [str(targets / "turnstone/debug" / ("g3_receipt" + suffix)), str(rendered)], cwd=turnstone, expected=1)
            if "pending" not in log.read_text() or rendered.exists():
                raise RuntimeError("Turnstone G3 must produce no identity-bound receipt while pending")
            log = run(f"{mode}-vault-cli", [str(cli), "--app-endpoint", endpoint, "new-profile", "must-not-exist"], expected=2)
            if "identity pending" not in log.read_text():
                raise RuntimeError("vault CLI did not report pending")
            if sorted(str(path.relative_to(product_root)) for path in product_root.rglob("*")) != before:
                raise RuntimeError("vault CLI wrote product state while pending")
            if mode == "locked":
                def public_roster(text):
                    if "default" not in text:
                        raise RuntimeError("missing public roster")
                run("locked-vault-public-roster", [str(cli), "--app-endpoint", endpoint, "profiles"],
                    checks=public_roster)
                locked_after = {str(path.relative_to(host_root)): digest(path)
                                for path in host_root.rglob("*") if path.is_file()}
                if locked_before != locked_after:
                    raise RuntimeError("pending CLI or app wrote the resident vault")
            dist_root = product_root / "distillery"
            run(f"{mode}-distillery-configure", [str(distillery), "configure", "--data-root", str(dist_root), "--profile", "default"])
            saved = {str(path.relative_to(dist_root)): digest(path) for path in dist_root.rglob("*") if path.is_file()}
            log = run(f"{mode}-distillery-inspect", [str(distillery), "inspect", "--data-root", str(dist_root), "--app-endpoint", endpoint])
            if "Personae identity: pending" not in log.read_text():
                raise RuntimeError("Distillery did not report pending")
            if saved != {str(path.relative_to(dist_root)): digest(path) for path in dist_root.rglob("*") if path.is_file()}:
                raise RuntimeError("Distillery changed configuration while pending")
            if host:
                host.terminate()
                host.wait(timeout=20)
                host = None
                host_log.close()
        # Keep the CLI's successful administrative path exercised too. Its
        # public output crosses the broker; private slots stay in the keeper.
        token = str(uuid.uuid4())
        endpoint = rf"\\.\pipe\dr-c-cli-{token}" if os.name == "nt" else str(runtime / (token + ".sock"))
        host_root = runtime / "unlocked-host"
        host_log = (output / "unlocked-host.log").open("w")
        host = subprocess.Popen([str(host_bin), endpoint, str(host_root), "unlocked"], stdout=host_log, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 60
        while not (host_root / "ready").exists():
            if host.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError("unlocked CLI keeper did not bind")
            time.sleep(0.1)
        for label, arguments, required in [
            ("create", ["new-profile", "cli-persona"], "created profile"),
            ("roster", ["profiles"], "cli-persona"),
            ("import", ["add-ssh", "ports/castellan/tests/fixtures/ssh/ed25519"], "imported ssh:"),
            ("list", ["list"], "1 slot"),
            ("pub", ["pub", "ssh"], "ssh-ed25519"),
            ("ca", ["ca"], "TrustedUserCAKeys"),
        ]:
            log = run(f"unlocked-vault-{label}", [str(cli), "--app-endpoint", endpoint, *arguments])
            if required not in log.read_text():
                raise RuntimeError(f"CLI {label} did not return the expected public output")
        host.terminate()
        host.wait(timeout=20)
        host = None
        host_log.close()
        # A real production binding is deliberately changed in a separate clean
        # checkout. Both endpoints must now fail the SAME app receipt. Never
        # install a fallback switch in the shipping source or mutate main.
        if args.fallback_control:
            control = args.fallback_control.resolve()
            if control == turnstone or control.parent.name != "worktrees":
                raise RuntimeError("fallback control requires an isolated worktree")
            if subprocess.check_output(["git", "status", "--porcelain"], cwd=control, text=True).strip():
                raise RuntimeError("fallback control worktree must be clean")
            source = control / "src/identity.rs"
            for file in ["src/identity.rs", "src/app/tests.rs"]:
                if digest(control / file) != digest(turnstone / file):
                    raise RuntimeError(f"control does not match tested Turnstone source: {file}")
            original = source.read_bytes()
            before = digest(source)
            pending = '''            tracing::warn!(%error, "profile identity pending: djinn is absent or locked");
            None'''
            fallback = '''            tracing::warn!(%error, "DELIBERATELY RESTORED FALLBACK FOR DR-C CONTROL");
            Some(Arc::new(RootIdentity::Local(identity::InMemoryProvider::from_seed([0x6d; 32]))))'''
            text = original.decode().replace("\r\n", "\n")
            if text.count(pending) != 1:
                raise RuntimeError("production pending branch changed; review the control")
            try:
                source.write_text(text.replace(pending, fallback), encoding="utf-8")
                control_record = {"source": "src/identity.rs", "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=control, text=True).strip(), "clean_sha256": before,
                                  "fallback_sha256": digest(source), "patch": fallback}
                mutant = test_executable("restored-fallback", control, "turnstone", ["-p", "turnstone", "--lib"])
                for mode in ["absent", "locked"]:
                    token = str(uuid.uuid4())
                    endpoint = rf"\\.\pipe\dr-c-control-{token}" if os.name == "nt" else str(runtime / (token + ".sock"))
                    env["GRAPHSHELL_APP_ENDPOINT"] = endpoint
                    env["DR_C_RECEIPT_MODE"] = mode
                    if mode == "locked":
                        host_root = runtime / "control-locked-host"
                        host_log = (output / "control-locked-host.log").open("w")
                        host = subprocess.Popen([str(host_bin), endpoint, str(host_root), "locked"], stdout=host_log, stderr=subprocess.STDOUT)
                        deadline = time.monotonic() + 60
                        while not (host_root / "ready").exists():
                            if host.poll() is not None or time.monotonic() >= deadline:
                                raise RuntimeError("control keeper did not bind")
                            time.sleep(0.1)
                    log = run(f"{mode}-restored-fallback", [mutant, "dr_c_", "--ignored", "--nocapture", "--test-threads=1"], expected=101)
                    failure = log.read_text()
                    if "Turnstone must not bind a fallback root" not in failure or "0 passed; 1 failed" not in failure:
                        raise RuntimeError("the control failed for another reason")
                    if host:
                        host.terminate()
                        host.wait(timeout=20)
                        host = None
                        host_log.close()
            finally:
                source.write_bytes(original)
                if digest(source) != before:
                    raise RuntimeError("fallback control source was not restored")
            control_record["restored"] = True
        for label, repo in [("mere", mere), ("knot", knot), ("woodshed", woodshed), ("hocket", woodshed / "ports/hocket"), ("turnstone", turnstone)]:
            run(f"bans-{label}", ["cargo", "deny", "--locked", "check", "bans"], cwd=repo)
        for name, (repo, files) in sources.items():
            if {file: digest(repo / file) for file in files} != source_record[name]["files"]:
                raise RuntimeError(f"{name} source changed during receipts; rerun against that source")
        finished = bool(control_record and control_record.get("restored"))
    finally:
        if host:
            host.terminate()
            host.wait(timeout=20)
        result = {"schema": "mere.dr-c.receipts/v1", "sources": source_record,
                  "restored_fallback": control_record, "runs": records,
                  "complete": finished and all(item["exit"] == item["expected"] for item in records)}
        (output / "receipt.json").write_text(json.dumps(result, indent=2) + "\n")
        # Only marker-identified synthetic data, after its process owner ended.
        if (runtime / ".dr-c-generated").is_file():
            shutil.rmtree(runtime)


if __name__ == "__main__":
    main()
