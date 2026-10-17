"""Refresh this pass's coverage, preserved-WIP check and scoped evidence receipt."""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def load(name):
    return json.loads((HERE / name).read_text(encoding="utf-8"))


def save(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


parser = argparse.ArgumentParser()
parser.add_argument("--test-state", choices=["running", "passed", "failed", "cancelled"], required=True)
parser.add_argument("--test-exit", type=int)
args = parser.parse_args()
assert load("pending_read.json") == [], "Acknowledge actual reading first"
inventory = load("source_inventory.json")["files"]
coverage = load("read_coverage.json")
assert all(digest(ROOT / path) == info["sha256"] for path, info in inventory.items()), "Refresh changed inventory before recording"
status = json.loads(subprocess.check_output(
    ["python", str(HERE / "read_coverage.py"), "status"], cwd=ROOT, text=True))
previous = load("cxf_admission_receipt.json")["preserved_production_and_test_hashes"]
preserved = {path: digest(ROOT / path) for path in previous}
assert preserved == previous, "Earlier repair WIP changed; investigate before claiming preservation"
native = load("unix_endpoint_native_output.txt")
for path, evidence in native["sources"].items():
    assert digest(ROOT / path) == evidence["sha256"], "Native probe source basis changed"

command = "cargo test -p djinn --lib door::identity_endpoint::tests --locked --offline -j 1 --target-dir C:/t/cargo-targets/mere --config build.build-dir=C:/t/cargo-targets/mere/build"
baseline = {"command": command, "state": "failed", "exit_code": 101,
            "log": "djinn_identity_endpoint_output.txt", "passed": 10, "failed": 1,
            "cause": "Q051: exact fixture comparison failed after Windows CRLF checkout; semantic bytes matched HEAD after newline normalization."}
test_binary = Path("C:/t/cargo-targets/mere/build/debug/deps/djinn-80eb7569cbe46a36.exe")
test = {"command": str(test_binary) + " door::identity_endpoint::tests", "state": args.test_state, "exit_code": args.test_exit,
        "log": "djinn_identity_endpoint_lf_output.txt", "passed": 11, "failed": 0,
        "binary_sha256": digest(test_binary), "build_and_pre_repair_run": baseline,
        "scope": "Existing identity endpoint unit tests, in-process fixed authority. Passing does not establish disclosure safety when the production reader is composed, or fresh-process readiness and transfer durability."}
if args.test_state == "passed":
    assert args.test_exit == 0
    assert "test result: ok. 11 passed; 0 failed" in (HERE / test["log"]).read_text(encoding="utf-8", errors="replace")
fixture_path = "ports/graphshell/tests/fixtures/identity_cards.json"
assert (ROOT / fixture_path).read_bytes() == subprocess.check_output(
    ["git", "show", "HEAD:" + fixture_path], cwd=ROOT), "Fixture content must stay at its original committed bytes"

source_paths = [
    ".gitattributes",
    "ports/graphshell/tests/fixtures/identity_cards.json",
    "ports/djinn/src/bin/djinn.rs",
    "ports/djinn/src/bin/h4_identity_receipt.rs",
    "ports/djinn/src/door/app_broker.rs",
    "ports/djinn/src/door/device_broker.rs",
    "ports/djinn/src/door/identity_endpoint.rs",
    "ports/djinn/src/door/local_session.rs",
    "ports/djinn/src/personal_sync.rs",
    "ports/djinn/src/resident_status.rs",
    "ports/djinn/src/resident_blobs.rs",
    "ports/djinn/src/embedded_reservoir.rs",
    "ports/djinn/src/pairing.rs",
    "ports/djinn/src/settings.rs",
    "ports/djinn/src/startup_vault.rs",
    "ports/djinn/src/resident_distillery.rs",
    "ports/djinn/src/resident_mere.rs",
    "ports/djinn/src/resident_reservoir.rs",
    "ports/djinn/src/sync/carriage_host.rs",
    "ports/djinn/src/sync/carriage_host_commissioning.rs",
    "ports/djinn/src/sync/graph_keys.rs",
    "ports/djinn/src/sync/transfer_staging.rs",
    "ports/djinn/tests/embedded_reservoir_two_process.rs",
    "ports/djinn/tests/device_directory_two_instance.rs",
    "crates/graphshell/graphshell-host/src/endpoint_catalog.rs",
    "ports/distillery/src/installed.rs",
    "ports/distillery/src/resident.rs",
    "ports/djinn/install-linux.sh",
    "ports/djinn/install-windows.ps1",
    "ports/djinn/install-ssh-server-windows.ps1",
    "crates/graphshell/graphshell-endpoint/src/door/local_endpoint.rs",
    "crates/graphshell/graphshell-endpoint/src/lib.rs",
    "crates/graphshell/graphshell-host/src/session_loop.rs",
    "crates/system/djinn-testkit/src/resident.rs",
    "ports/djinn/src/sync/personal_sync_host.rs",
    "design_docs/dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md",
]
remaining = [path for path in sorted(inventory)
             if path.startswith("ports/djinn/")
             and coverage.get(path, {}).get("through", 0) < inventory[path]["chars"]]
next_path = remaining[0]
complete_doors = [path for path in sorted(inventory) if path.startswith("ports/djinn/src/door/")]
assert all(coverage.get(path, {}).get("through", 0) == inventory[path]["chars"] for path in complete_doors)
complete_src = [path for path in sorted(inventory) if path.startswith("ports/djinn/src/")]
assert all(coverage.get(path, {}).get("through", 0) == inventory[path]["chars"]
           and coverage[path]["sha256"] == inventory[path]["sha256"] for path in complete_src)
receipt = {
    "date_utc": datetime.now(timezone.utc).isoformat(),
    "basis_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "kind": "Source-qualified findings, native Linux primitive probe and separately scoped existing test attempt",
    "findings": [f"Q{number:03}" for number in range(43, 52)],
    "authority_entries": [f"A{number:03}" for number in range(39, 48)],
    "source_sha256": {path: digest(ROOT / path) for path in source_paths},
    "native_probe": {
        "command": "wsl.exe -d Ubuntu --exec python3 /mnt/c/Users/mark_/Code/repos/mere/support/receipts/authority-success-audit/unix_endpoint_native_probe.py",
        "exit_code": 0,
        "log": "unix_endpoint_native_output.txt",
        "observations": native["observations"],
        "scope": native["kind"],
        "generated_fixture_removed": native["fixture_removed"],
    },
    "endpoint_tests": test,
    "coverage": status,
    "all_djinn_door_files_complete": complete_doors,
    "all_djinn_src_files_complete": complete_src,
    "coverage_limits": "Session-loop, host endpoint catalog, testkit and Distillery installed/resident boundary traces outside the full reader are targeted excerpts only and do not count as full-file source reading. All Djinn src source is fully read; separate integration tests and manifests remain. Whole Mere goal remains active.",
    "preserved_production_and_test_hashes": preserved,
    "production_edits_this_pass": ["ports/djinn/src/bin/h4_identity_receipt.rs", ".gitattributes"],
    "receipt_correction": {"finding": "Q048", "change": "Non-Windows unexercised standard_endpoint_refused is null instead of true; Windows exercised result unchanged.", "verification": "Changed source fully reread; complete repository field search found no typed boolean consumer. Windows syntax-format check is recorded separately. Non-Windows transport execution was not performed."},
    "fixture_correction": {"finding": "Q051", "change": "Exact LF checkout rule for identity_cards.json. Restored current HEAD bytes; JSON contents unchanged.", "worktree_before_sha256": "a0950aab3533d3e521f82107fb116848f2a27344e7c0b9f56d9b0b620b70c934", "crlf_before": 2460, "current_sha256": digest(ROOT / "ports/graphshell/tests/fixtures/identity_cards.json")},
    "receipt_format_check": {"command": "rustfmt --edition 2024 --check ports/djinn/src/bin/h4_identity_receipt.rs", "exit_code": 0, "log": "h4_receipt_format_output.txt", "scope": "Syntax and formatting; not non-Windows wire execution."},
    "diff_check": {"command": "git diff --check", "exit_code": 0, "log": "djinn_door_diffcheck_output.txt"},
    "retained_isolation": [],
    "build_output": "Reused C:/t/cargo-targets/mere and its build subdirectory. No isolated target, Cargo home or worktree created.",
    "next_read": {"path": next_path, "start": coverage.get(next_path, {}).get("through", 0)},
}
evidence_paths = ["unix_endpoint_native_probe.py", "unix_endpoint_native_output.txt", "djinn_identity_endpoint_output.txt", "djinn_identity_endpoint_lf_output.txt", "h4_receipt_format_output.txt", "djinn_door_diffcheck_output.txt", "record_djinn_door_review.py"]
receipt["evidence_sha256"] = {path: digest(HERE / path) for path in evidence_paths}
save("djinn_door_review_receipt.json", receipt)

notes = load("review_notes.json")
notes["current_area"] = "Djinn resident composition; continuing " + next_path
notes["next_area"] = "Unread Djinn integration tests and manifests, then all remaining Mere source"
for qualification in [
    {"area": "Distillery second vault open", "note": "Targeted trace shows InstalledAuthority::bind_resident consumes self and constructs ResidentAuthority without retaining InstalledAuthority or its IdentityProvider field. The extra profile vault open is temporary composition, not evidence of a second vault retained for the entire works run. Derived lane author/transport keys do remain intentionally. This is source ownership evidence, not a physical residue proof; full Distillery source reading remains ahead."},
    {"area": "Published-site readiness", "note": "Unlike the resident's pre-poll ready event, start_gemini waits for worker listener conversion acknowledgment and gates replies until publication activation after descriptor installation. No ordinary worker-error false-Serving finding was established from the fully read accept loop; accept errors log and retry."},
]:
    if not any(item["area"] == qualification["area"] for item in notes["source_qualifications"]):
        notes["source_qualifications"].append(qualification)
notes["checkpoint"] = {
    "classification": "progress", "goal_status": "active; full-source objective incomplete",
    "coverage": status, "finding_count": len(notes["findings"]),
    "authority_entries": len(load("authority_map.json")["entries"]),
    "new_receipts": ["djinn_door_review_receipt.json"],
    "diagnostic_outcomes": "Nine defects/overclaims recorded; Q048 receipt evidence corrected and Q051 exact-fixture checkout repaired. Linux primitive policy probe passed. Endpoint suite first failed 1/11, then passed 11/11 after only LF checkout correction.",
    "next_read": receipt["next_read"], "workspace": receipt["build_output"],
}
notes["audit_continuation"] = {
    "previous_turn": "Completed Castellan and reproduced Q040-Q042",
    "classification_of_previous_turn": "progress", "production_edits_this_turn": receipt["production_edits_this_pass"],
    "completed_reading_this_pass": "Djinn installers and every source file under ports/djinn/src, including binaries, conditions, doors, embedded reservoir, native UI, keeper, lock triggers, pairing, all resident composition and MereRoutes tests, settings, startup vault, personal sync, carriage host/commissioning/tests, graph keys and transfer staging. Integration tests read through embedded_reservoir_two_process.rs, including shared fixtures, custody, CXF, device directory, Chronicle and trainer feature paths. Exact acknowledged hashes/offsets are in read_coverage.json.",
    "completed_areas_this_pass": ["ports/djinn/src/door", "ports/djinn/src"],
    "coverage_checkpoint": status,
    "next_read": receipt["next_read"],
    "user_authorization": notes["repair_policy"]["user_instruction"],
    "diagnostics": test,
    "coverage_note": "Full source review continues. Existing repair WIP hashes were checked unchanged; contained changes are Q048's non-Windows receipt evidence field and Q051's LF checkout rule.",
}
if "ports/djinn/src/door" not in notes["completed_areas"]:
    notes["completed_areas"].append("ports/djinn/src/door")
if "ports/djinn/src" not in notes["completed_areas"]:
    notes["completed_areas"].append("ports/djinn/src")
save("review_notes.json", notes)
print(json.dumps({"coverage": status, "preserved_repair_files": len(preserved), "test": test["state"], "next_read": receipt["next_read"]}))
