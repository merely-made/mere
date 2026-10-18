# Move the full-source audit record out of the scenograph editor plan into its own plan,
# byte for byte, and seal the move in a successor receipt beside audit_completion_receipt.json.
# The scenograph plan carries CRLF endings; the appended audit record carries LF. The move
# keeps each as it was.
import hashlib, json, datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OLD = ROOT / "design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md"
NEW = ROOT / "design_docs/mere_docs/implementation_strategy/2026-10-16_full_source_audit_plan.md"
PRIOR = HERE / "audit_completion_receipt.json"
FIRST, LAST = 2394, 5481  # 1-indexed inclusive lines of the record in OLD at commit 00db7399
MARKER = b"## Full-source audit and contained repairs, 2026-10-16"
sha = lambda b: hashlib.sha256(b).hexdigest()

old_bytes = OLD.read_bytes()
assert old_bytes.count(MARKER) == 1
idx = old_bytes.find(MARKER)
assert old_bytes[:idx].count(b"\n") == FIRST - 1, old_bytes[:idx].count(b"\n")
assert old_bytes.count(b"\n") == LAST, old_bytes.count(b"\n")
record = old_bytes[idx:]
prefix = old_bytes[:idx].rstrip()
prefix_eol = b"\r\n" if b"\r\n" in prefix[:4000] else b"\n"
record_eol = b"\r\n" if b"\r\n" in record[:4000] else b"\n"

def block(lines, eol):
    return eol.join(l.encode("utf-8") for l in lines) + eol

header = block([
    "# Full-Source Audit Plan",
    "",
    "**Date:** 2026-10-16 (record extracted from the scenograph editor plan on 2026-10-17)",
    "**Status (2026-10-17):** landed as an audit. The full-source review of Mere is complete at",
    "working HEAD `7c718105` plus the preserved shared working changes: 3,738 files, 1,043,171",
    "lines, 39,556,704 characters, every file read contiguously against its SHA-256. The registers",
    "hold findings Q001 to Q465 (24 repaired, the rest documented with evidence and limits),",
    "authority entries A001 to A1536, 212 unconfirmed candidates and 23 ruled-out concerns.",
    "At closeout 23 of the 24 repairs were uncommitted in the shared checkout; ruling R1 below",
    "governs their landing. Repairs of the open findings and wider runtime qualification are",
    "follow-up work, not phases of this plan.",
    "**Scope:** the whole-Mere read-only source audit for duplicated authority and misleading",
    "success, with the contained repairs made under the one-hour diversion limit. Evidence lives",
    "in [`support/receipts/authority-success-audit/`](../../../support/receipts/authority-success-audit/):",
    "the [completion receipt](../../../support/receipts/authority-success-audit/audit_completion_receipt.json),",
    "the [findings register](../../../support/receipts/authority-success-audit/review_notes.json) and",
    "the [authority map](../../../support/receipts/authority-success-audit/authority_map.json).",
    "",
    "## 1. Rulings",
    "",
    "Put to Mark on 2026-10-17 after a fresh read of the closeout, the registers and three",
    "findings checked against source (Q453, Q455, Q465, all confirmed).",
    "",
    "### R1. The uncommitted repair batch (2026-10-17)",
    "",
    "Question as put: the 23 recorded repairs sit uncommitted in the shared mere checkout",
    "(33 files, +1,574/-168, idle since 2026-10-16 18:13, gated on `b567660d` before a",
    "16-commit fast-forward). What should happen to them?",
    "",
    "Options: (a) re-gate and sweep in: run check and tests for the touched crates",
    "sequentially on the shared target, then commit the whole tree with the doc addenda as",
    "one sweep on main and push under standing permission, any red gate coming back first;",
    "(b) re-gate, commit per lane (custody; chartulary and graph-kernel; scenograph and",
    "scenomise); (c) leave it dirty for the owning sessions.",
    "",
    'Mark: **"Re-gate and sweep in (Recommended)"**.',
    "",
    "What follows: the gates are the audit's own recorded commands (chartulary plus mere-kernel",
    "with `store`, pandect, castellan with `keeper`, scenograph default and `scripts`, scenomise",
    "with `authoring`, script-rhai, graphshell-client all-features, the djinn `h4_identity_receipt`",
    "bin check) rerun on the fast-forwarded base. One commit on main carries the repairs, their",
    "three new test files and the five doc addenda. The graphshell-client all-features gate has",
    "one pre-existing exact-byte failure (Q073) that is not this batch's. *Reading, not ruled:*",
    "the receipts' `-j 1` is not required when the machine is otherwise idle, and `--offline`",
    "cannot be kept because the fast-forward repinned genet to a revision absent from the local",
    "git cache.",
    "",
    "### R2. Where the audit record lives (2026-10-17)",
    "",
    "Question as put: the 3,090-line audit record lives inside the scenograph editor plan, and",
    "the completion receipt binds that path and SHA-256. Where should it live?",
    "",
    "Options: (a) leave in place and add a pointer plan; (b) extract to its own plan and record",
    "the move in a successor receipt, accepting that every existing receipt binding to the old",
    "path and hash becomes historical; (c) leave as is.",
    "",
    'Mark: **"Extract to its own plan"**.',
    "",
    "What follows: this file. The record below is moved byte for byte from",
    "`2026-10-07_scenograph_editor_plan.md` lines 2394 to 5481 as committed in `00db7399`;",
    "its relative receipt links are unchanged because both plans sit in the same directory.",
    "The [relocation receipt](../../../support/receipts/authority-success-audit/audit_record_relocation_receipt.json)",
    "records both files' hashes before and after and the hash of the moved region. The",
    "receipts that name the old path describe the record's location at the time they were",
    "sealed.",
    "",
    "## 2. Phases",
    "",
    "- **A1, full-source read.** Done-condition: every live source file read contiguously",
    "  against its current hash, with findings, authority entries and evidence registers",
    "  reconciled. Met 2026-10-17; the completion receipt lists the six requirements and",
    "  their results (five proved, the main refresh proved with scope).",
    "- **A2, land the contained repairs (R1).** Done-conditions: the recorded gates pass on",
    "  the current base, apart from the pre-existing Q073 failure; one sweep commit on main",
    "  carries the 33 dirty files and three untracked tests; pushed; the Progress log below",
    "  records the gate counts.",
    "",
    "## 3. Findings and progress",
    "",
    "The record that follows is the audit's own dated log, unchanged. It is the Findings and",
    "Progress sections of this plan. New entries go after its closeout.",
    "",
    "---",
    "",
], record_eol)

pointer = block([
    "",
    "",
    "## Full-source audit record moved, 2026-10-17",
    "",
    "The whole-Mere audit record this plan carried from 2026-10-16 (lines 2394 to 5481 at",
    "commit `00db7399`) now lives in",
    "[2026-10-16_full_source_audit_plan.md](2026-10-16_full_source_audit_plan.md), per Mark's",
    "ruling R2 there. Audit receipts that bind this file's earlier path and hash describe that",
    "history; the",
    "[relocation receipt](../../../support/receipts/authority-success-audit/audit_record_relocation_receipt.json)",
    "records both files before and after the move.",
], prefix_eol)

assert not NEW.exists(), NEW
new_bytes = header + record
old_after = prefix + prefix_eol + pointer
NEW.write_bytes(new_bytes)
OLD.write_bytes(old_after)
assert NEW.read_bytes() == new_bytes and OLD.read_bytes() == old_after
assert new_bytes.endswith(record) and record in old_bytes and old_after.startswith(prefix)

prior = json.loads(PRIOR.read_text(encoding="utf-8"))
assert prior["document"]["path"] == OLD.relative_to(ROOT).as_posix()
naming_old = sum(
    1 for p in HERE.glob("*")
    if p.is_file() and p.suffix in {".json", ".py", ".txt", ".md"}
    and OLD.name in p.read_text(encoding="utf-8", errors="replace")
)
receipt = {
    "date_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "purpose": "Record the move of the full-source audit record out of the scenograph editor plan into its own plan (ruling R2, 2026-10-17). Content moved byte for byte; no finding, authority entry or receipt changed.",
    "prior_receipt": {"path": PRIOR.name, "sha256": sha(PRIOR.read_bytes()), "document_sha256_at_completion": prior["document"]["sha256"]},
    "old_document": {
        "path": OLD.relative_to(ROOT).as_posix(), "sha256_before": sha(old_bytes), "sha256_after": sha(old_after),
        "removed_lines": [FIRST, LAST], "lines_before": LAST, "lines_after": old_after.count(b"\n"),
        "line_ending": "CRLF" if prefix_eol == b"\r\n" else "LF",
        "appended": "pointer section 'Full-source audit record moved, 2026-10-17'",
    },
    "new_document": {
        "path": NEW.relative_to(ROOT).as_posix(), "sha256": sha(new_bytes),
        "header_bytes": len(header), "record_offset": len(header), "lines": new_bytes.count(b"\n"),
    },
    "moved_record": {
        "sha256": sha(record), "bytes": len(record), "lines": record.count(b"\n"),
        "first_heading": MARKER.decode(), "line_ending": "CRLF" if record_eol == b"\r\n" else "LF",
        "byte_identical_in_both": True,
    },
    "receipts_naming_old_path": naming_old,
    "relative_links": "unchanged; both plans are in design_docs/mere_docs/implementation_strategy/",
    "recorder": {"path": Path(__file__).name},
}
out = HERE / "audit_record_relocation_receipt.json"
with out.open("x", encoding="utf-8") as f:
    json.dump(receipt, f, indent=2)
    f.write("\n")
print(json.dumps({k: receipt[k] for k in ("old_document", "new_document", "moved_record", "receipts_naming_old_path")}, indent=1))
