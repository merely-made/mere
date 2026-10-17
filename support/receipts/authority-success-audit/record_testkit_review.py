"""Persist this audit-only continuation, with source-bound diagnostic limits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda name: json.loads((HERE / name).read_text(encoding="utf-8"))
def save(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2)+"\n", encoding="utf-8")
def merge(existing, additions):
    remaining = {item["id"]: item for item in additions}
    return [remaining.pop(item["id"], item) for item in existing]+list(remaining.values())

assert load("pending_read.json") == []
inventory = load("source_inventory.json")["files"]
coverage = load("read_coverage.json")
assert all(sha(ROOT / path) == info["sha256"] for path, info in inventory.items())
previous = load("djinn_door_review_receipt.json")
preserved = previous["preserved_production_and_test_hashes"].copy()
for path in [".gitattributes", "ports/djinn/src/bin/h4_identity_receipt.rs", "ports/graphshell/tests/fixtures/identity_cards.json"]:
    preserved[path] = previous["source_sha256"][path]
assert all(sha(ROOT / path) == digest for path, digest in preserved.items()), "Prior product repair basis changed"
classifier = load("knot_residue_classifier_receipt.json")
boundary = load("testkit_boundary_receipt.json")
assert sha(ROOT / classifier["source"]) == classifier["source_sha256"]
assert classifier["test_exit"] == boundary["test_exit"] == 0
assert classifier["generated_native_output_removed"] and boundary["generated_native_output_and_fixtures_removed"]
assert "test result: ok. 4 passed" in (HERE / "knot_residue_classifier_output.txt").read_text(encoding="utf-8")
assert "test result: ok. 5 passed; 0 failed; 1 ignored" in (HERE / "testkit_boundary_output.txt").read_text(encoding="utf-8")
for path, digest in boundary["source_sha256"].items():
    assert sha(ROOT / path) == digest
for item in boundary["saved_evidence"]:
    assert sha(HERE / item["path"]) == item["sha256"]
areas = {}
for prefix in ["ports/djinn/", "crates/system/djinn-testkit/"]:
    files = [p for p in sorted(inventory) if p.startswith(prefix)]
    assert all(coverage.get(p, {}).get("through", 0) == inventory[p]["chars"] and coverage[p]["sha256"] == inventory[p]["sha256"] for p in files)
    areas[prefix] = {"files": len(files), "chars": sum(inventory[p]["chars"] for p in files), "source_sha256": {p: inventory[p]["sha256"] for p in files}}
receipt_name = "testkit_review_receipt.json"
findings = [
    {"id": "Q052", "status": "classifier overclaim reproduced; not repaired",
     "title": "Knot's no-copy residue success excludes intermittent and same-size additional copies",
     "source": "ports/djinn/tests/knot_residue.rs:279", "related_sources": ["ports/djinn/tests/knot_residue.rs:391", "ports/djinn/castellan/tests/residue/mod.rs"],
     "evidence": "The exact own_blocks source was compiled in a standalone native Rust harness. A persistent unique allocation size is reported as its positive control. A copy seen in only one of five host runs, a persistent copy matching one baseline observation, and additional same-size copies are all discarded. The sync-host gate only fails on the resulting set, then prints Knot leaves no copy beyond its bind/join baseline.",
     "scope": "Exact classifier on synthetic already-observed allocation records, not execution of Knot, its allocator tracker or a real seed leak. Non-network author/session/capture paths check every hit and are not subject to this particular intersection/subtraction defect. The tracker is reused from Castellan; the attribution classifier and stronger success message are Knot-local.",
     "verification": receipt_name, "diagnostic": "knot_residue_classifier_receipt.json",
     "repair_triage": {"disposition": "documented; correct evidence claim and qualify attribution separately", "reason": "A deterministic size-class excess detector cannot prove absence of all copies. Retain every observed hit, qualify baseline attribution and multiplicity, and state the heap/arming window and excluded upstream cases. Strong ownership attribution needs more than allocation size and may exceed a one-hour audit diversion."}},
    {"id": "Q053", "status": "reproduced through public guard and Windows process environment; not repaired",
     "title": "Windows environment case aliases bypass the testkit's root and forbidden-variable checks",
     "source": "crates/system/djinn-testkit/src/guard.rs:116", "related_sources": ["crates/system/djinn-testkit/src/guard.rs:132", "crates/system/djinn-testkit/src/resident.rs:217"],
     "evidence": "Guard checks BTreeMap names with exact-case lookup. The canonical LOCALAPPDATA outside-root and SSH_AUTH_SOCK standard-agent controls are refused. Adding localappdata alongside the redirected uppercase key, plus lowercase ssh_auth_sock, passes Guard::check. An environment-only child composed with the same env_clear/envs operations sees the outside-root value and the standard-agent name through uppercase Windows lookup.",
     "scope": "Native Windows public Guard API plus actual Windows child environment semantics; Resident::env/env_map/guarded_env source confirms this composition path. The outside value was a generated decoy, and the child only read variables: no Djinn, SSH client or endpoint connection, and no real user state touched. Unix variable lookup is case-sensitive and was not tested as vulnerable.",
     "verification": receipt_name, "diagnostic": "testkit_boundary_receipt.json",
     "repair_triage": {"disposition": "documented; bind validation to the effective Windows environment", "reason": "Normalize or reject case-colliding Windows names before both checks and process composition, using the same effective map. Cover redirected roots, endpoint variables, forbidden names and permitted harness-role/askpass additions; the current guard is not an arbitrary-variable allowlist."}},
    {"id": "Q054", "status": "reproduced through public Run finalization; not repaired",
     "title": "Receipt finalization reports passed after dropping requested evidence or failing to write receipt.json",
     "source": "crates/system/djinn-testkit/src/run.rs:430", "related_sources": ["crates/system/djinn-testkit/src/run.rs:474", "crates/system/djinn-testkit/src/receipt.rs:verify"],
     "evidence": "Run::write filter_maps evidence hash errors away and ignores receipt/summary write errors. A public Run requesting a nonexistent evidence file finishes passed with an empty evidence list; verify returns ok with checked=0. A directory deliberately occupying receipt.json makes its write fail, yet finish_record returns passed and prints a passed receipt path while a summary is written. Existing readable-evidence control retains/checks one file.",
     "scope": "Current public API, native Windows generated fixtures, no resident. The verifier correctly errors when receipt.json cannot be read and correctly checks retained evidence; its documented job is hash verification, not deriving outcome. The missing-evidence case silently narrowed what it could check before verification. These diagnostic tests expose defects, not repairs or acceptance of the harness.",
     "verification": receipt_name, "diagnostic": "testkit_boundary_receipt.json",
     "repair_triage": {"disposition": "documented; make finalization and required-evidence failures observable", "reason": "Preserve failed requested paths as failures instead of dropping them; propagate receipt/summary publication errors through an explicit fallible finish contract and qualify unwritable/missing evidence controls across consumers. Retain failed-run diagnostic records rather than replacing them with a passed summary."}},
    {"id": "Q055", "status": "reproduced through public Resident::start; not repaired",
     "title": "Resident start writes lock settings outside its run before the spawn guard refuses the path",
     "source": "crates/system/djinn-testkit/src/resident.rs:467", "related_sources": ["crates/system/djinn-testkit/src/resident.rs:471", "crates/system/djinn-testkit/src/run.rs:resident"],
     "evidence": "Resident::new joins the supplied name directly to the run directory. An absolute generated decoy name selects a root outside the run. start creates its application directory and writes lock.toml before calling guarded. The guard then correctly refuses the outside environment, but the lock settings file already exists. No child was spawned, and the empty-wall probe run still finalizes passed.",
     "scope": "Public Run::resident/Resident::start with an absolute fixture name on Windows. The observed outside path remained inside the probe's own generated target subtree; no installed or user root was used. Does not claim ordinary fixed safe resident names escape or that the guard itself accepts this root.",
     "verification": receipt_name, "diagnostic": "testkit_boundary_receipt.json",
     "repair_triage": {"disposition": "documented; validate ownership before all pre-spawn writes", "reason": "Validate the resident name/root before construction or file creation, and perform the complete spawn/path check before writing lock settings. Qualify absolute, parent-component and ordinary names. Plan/evidence path validation is a separate source candidate."}},
]
notes = load("review_notes.json")
notes["findings"] = merge(notes["findings"], findings)
authority = load("authority_map.json")
authority["entries"] = merge(authority["entries"], [
    {"id": "A048", "domain": "Knot seed-residue tracking versus attribution and success reporting", "claim": "Knot leaves no copy of the signing seed beyond its bind/join baseline.", "implementations": ["Castellan allocator tracker reused by path", "Strict author/session/capture paths reject every reported hit", "Knot-local sync classifier intersects five runs and subtracts any baseline size class, collapsing multiplicity"], "assessment": "Real instrumentation reuse; local attribution and its no-copy conclusion are separate authority. Q052 reproduces classifier false negatives on synthetic observations. No actual Knot seed leak is established.", "state": "evidence overclaim reproduced with stated limits", "verification": receipt_name},
    {"id": "A049", "domain": "Spawn environment validation versus Windows effective variable names", "claim": "All child roots/endpoints are redirected and forbidden user-state variables are refused before spawning.", "implementations": ["isolated_env supplies conventional-case keys", "Resident extra_env and Guard use case-sensitive BTreeMap names", "Windows Command environment composition and lookup merge case aliases"], "assessment": "Q053 proves two naming authorities disagree in the actual host environment. Unknown-variable acceptance is not an independent attack proof: existing role/askpass helpers intentionally add variables. The normalized effective map must be the one validated and spawned.", "state": "public API and Windows child semantics reproduced", "verification": receipt_name},
    {"id": "A050", "domain": "Requested evidence, receipt publication, outcome and verification", "claim": "Every run writes a versioned record with evidence hashes that a verifier recomputes.", "implementations": ["Run records requested PathBufs", "Finalization silently omits failed hash reads", "Outcome only folds recorded failures while receipt/summary writes are ignored", "verify accurately recomputes only the retained entries"], "assessment": "Q054 shows successful lifecycle reporting can outlive both required evidence and the receipt file. Hash verification is not an independent authority for the completeness of the evidence request set, nor a durability acknowledgment.", "state": "missing evidence and failed publication reproduced", "verification": receipt_name},
    {"id": "A051", "domain": "Resident path ownership versus pre-spawn configuration writes", "claim": "Each resident lives under the run and the guard refuses wall crossing before a start acts.", "implementations": ["Run::resident accepts a raw name joined to dir", "start writes app_dir/lock.toml before guarded checks", "Guard rejects the eventual outside-root environment"], "assessment": "Q055 demonstrates two differently timed ownership boundaries: correct spawn refusal does not undo earlier filesystem writes. The guard is genuinely shared; the start preflight bypasses its sequencing.", "state": "outside-run generated-decoy write reproduced before refusal", "verification": receipt_name},
])
qualifications = [
    {"area": "Djinn complete source inventory", "note": "All 200 inventoried source/manifests/installers/tests under ports/djinn are fully read at current hashes, including all 100 Castellan files and 64 Djinn src files. This does not mean all feature/platform tests have been executed."},
    {"area": "Djinn shared testkit", "note": "All 14 files of djinn-testkit are fully read. Resident-status wire DTOs intentionally avoid a dependency cycle and have a producer/reader integration test; this is a versioned wire-reader seam, not a second durable resident authority. wait_ready enumerates endpoints and queries the app status for the current pid."},
    {"area": "Library daemon validation versus full binary lifecycle", "note": "embedded_reservoir_validation reuses with_divination_authority across embedded and library daemon owners and compares both projections and durable files after selected supported-schema tampering. Synthetic model/weather records are explicitly marked and do not claim model/network execution. Library daemon fixture clears its catalog; its source is not evidence that the full binary resolves Q049. These integration tests were source-reviewed, not run in this pass."},
    {"area": "Testkit platform limits", "note": "Unix pipe_names/pipes_containing are empty; non-Linux Unix process identity and parent-death handling are stubs. Therefore generic no-endpoint/no-live-child assertions need platform-scoped evidence. The accepted historical plan explicitly leaves the installed-location list empty on Unix and remote support open; do not call those empty observations all-platform protection."},
]
for item in qualifications:
    notes["source_qualifications"] = [old for old in notes["source_qualifications"] if old["area"] != item["area"]]+[item]
for item in [
    {"title": "Child job adoption follows spawn and errors do not visibly kill the unadopted child", "source": "crates/system/djinn-testkit/src/resident.rs:adopt", "note": "spawn/start/ssh_client_env launch first, then call job.adopt through ?. An adoption failure can return without recording or explicitly killing the child; dropping Child is not a kill. Qualify a controlled assignment failure and the spawn-before-adoption race without using installed processes before promoting an orphan finding."},
    {"title": "Run plan and evidence names also enter paths without a component admission check", "source": "crates/system/djinn-testkit/src/run.rs:begin/write_evidence", "note": "receipts_root is checked before joining config.plan, while plan and write_evidence name are joined as raw strings. Root/name admission and symlink handling need controlled decoy qualification. Q055 specifically reproduces Resident names and pre-guard lock writes, not every plan/evidence caller."},
    {"title": "Binary provenance is sampled at receipt finalization rather than launch", "source": "crates/system/djinn-testkit/src/run.rs:BinaryRecord", "note": "The configured binary file is hashed during write, while commit/dirty are sampled at begin and dirty excludes untracked files. A rebuild/replacement during a run could make this file hash describe a different binary from the running process. No binary-swap control was performed; do not treat the receipt as build-source identity attestation."},
    {"title": "Service-vocabulary refusal can follow an already accepted policy handshake", "source": "crates/graphshell/graphshell-host/src/admission.rs:168", "note": "Full admission.rs read shows admit_session retains notochord::admit's reply, then may replace an accepted principal with ActionNotOffered without revising that reply. Targeted carrier.rs source shows native admit_stream_with performs the I/O handshake before the service ActionCheck and shuts down on a later refusal; network_carrier maps SessionReply::Accept to an admitted stream. webrtc_door's adapter delegates to the sans-I/O helper. Trace complete carrier/join callers and exercise a policy that allows an implementation-unsupported action before promoting a wire false-accept finding. No authorization bypass is established."},
    {"title": "Captured-hello test varies authentication facts rather than two otherwise equivalent links", "source": "crates/graphshell/graphshell-host/src/admission.rs:a_captured_hello_does_not_open_a_different_connection", "note": "The hello is issued with a binding containing no carrier peer/link and then tested against authenticated facts naming a different peer. That verifies a changed-facts refusal, not replay protection between two otherwise equivalent unauthenticated sessions. Actual carrier facts, shared-link construction and nonce/replay ownership remain ahead in the full review; no replay vulnerability is claimed from this test alone."},
]:
    notes["candidates"] = [old for old in notes["candidates"] if old["title"] != item["title"]]+[item]
for area in areas:
    if area.rstrip("/") not in notes["completed_areas"]:
        notes["completed_areas"].append(area.rstrip("/"))
status = json.loads(subprocess.check_output(["python", str(HERE / "read_coverage.py"), "status"], cwd=ROOT, text=True))
next_prefix = "crates/graphshell/graphshell-host/"
remaining = [p for p in sorted(inventory) if p.startswith(next_prefix) and coverage.get(p, {}).get("through", 0) < inventory[p]["chars"]]
next_read = {"path": remaining[0], "start": coverage.get(remaining[0], {}).get("through", 0)}
receipt = {
    "date_utc": datetime.now(timezone.utc).isoformat(),
    "basis_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "governing_document_sha256": {path: sha(ROOT / path) for path in ["design_docs/mere_docs/implementation_strategy/2026-10-05_djinn_test_harness_plan.md", "design_docs/dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md"]},
    "previous_turn_classification": "progress: completed Djinn src, recorded Q043-Q051, corrected Q048/Q051 and exercised the endpoint suite",
    "kind": "Full Djinn/testkit source reading and bounded diagnostic reproductions; audit-only continuation",
    "findings": [item["id"] for item in findings], "authority_entries": ["A048", "A049", "A050", "A051"],
    "complete_areas": areas, "coverage": status, "next_read": next_read,
    "shared_host_reading": {p: {"sha256": inventory[p]["sha256"], "through": coverage[p]["through"], "total_chars": inventory[p]["chars"]} for p in sorted(inventory) if p.startswith(next_prefix) and coverage.get(p, {}).get("through", 0) > 0},
    "diagnostics": {"classifier": classifier, "public_testkit": boundary},
    "production_edits_this_pass": False, "preserved_production_and_test_hashes": preserved,
    "coverage_limits": "All inventoried Djinn and testkit source read; whole Mere goal incomplete. Existing production integration tests were read but not executed this pass. Diagnostics confirm classifier/API defects on the stated inputs, not repairs or an installed-resident/platform safety gate.",
    "retained_isolation": [],
    "generated_output": "Reused C:/t/cargo-targets/mere. Generated standalone native binaries, PDB/failed-link objects and owned decoy fixture tree were removed after terminal execution; evidence copied into this audit directory. No isolated Cargo home, target or worktree.",
    "diff_check": {"command": "git diff --check", "exit_code": 0, "log": "testkit_review_diffcheck_output.txt"},
}
evidence = ["probe_knot_residue_classifier.py", "knot_residue_classifier_probe.rs", "knot_residue_classifier_output.txt", "knot_residue_classifier_receipt.json", "testkit_boundary_probe.rs", "run_testkit_boundary_probe.py", "testkit_boundary_build_initial_output.txt", "testkit_boundary_build_link_failure_output.txt", "testkit_boundary_build_output.txt", "testkit_boundary_output.txt", "testkit_boundary_receipt.json", "djinn_testkit_build_output.jsonl", "testkit_review_diffcheck_output.txt"]
receipt["evidence_sha256"] = {path: sha(HERE / path) for path in evidence}
notes["current_area"] = "Djinn and djinn-testkit fully read; continuing shared Graphshell host ownership"
notes["next_area"] = "Shared Graphshell host catalog, admission and session lifecycle, then remaining Mere source"
notes["checkpoint"] = {"classification": "progress", "goal_status": "active; full-source objective incomplete", "coverage": status, "finding_count": len(notes["findings"]), "authority_entries": len(authority["entries"]), "new_receipts": [receipt_name], "diagnostic_outcomes": "4 exact-classifier controls and 5 public testkit diagnostic parents passed while confirming four unresolved findings; an environment-only helper was explicitly invoked in a child. No production edits.", "next_read": next_read, "workspace": receipt["generated_output"]}
notes["audit_continuation"] = {"previous_turn": receipt["previous_turn_classification"], "classification_of_previous_turn": "progress", "production_edits_this_turn": False, "completed_areas_this_pass": list(areas), "coverage_checkpoint": status, "next_read": next_read, "user_instruction": "Continue full source audit and preserve concurrent work; no production-code changes in this continuation.", "coverage_note": receipt["coverage_limits"]}
assert len({i["id"] for i in notes["findings"]}) == len(notes["findings"])
assert len({i["id"] for i in authority["entries"]}) == len(authority["entries"])
save("authority_map.json", authority)
save("review_notes.json", notes)
save(receipt_name, receipt)
print(json.dumps({"coverage": status, "findings": len(notes["findings"]), "authority_entries": len(authority["entries"]), "complete_areas": {k: v["files"] for k, v in areas.items()}, "preserved_files": len(preserved), "next_read": next_read}))
