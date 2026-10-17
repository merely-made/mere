"""Exercise the exact source classifier on synthetic observed allocation records.

This does not execute Knot, its allocator tracker, transport, or a real leak.
Generated Rust source is retained as evidence; generated native output is removed.
"""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
SOURCE = ROOT / "ports/djinn/tests/knot_residue.rs"
source_bytes = SOURCE.read_bytes()
source = source_bytes.decode("utf-8")
start = source.index("fn own_blocks(")
opening = source.index("{", start)
depth = 1
end = opening + 1
while depth:
    depth += (source[end] == "{") - (source[end] == "}")
    end += 1
classifier = source[start:end]
harness = '''use std::collections::BTreeSet;
struct Run { freed: Vec<usize>, live: Vec<usize>, frame: usize }
const OUTER_FRAME: usize = usize::MAX;
'''+classifier+'''
fn run(live: Vec<usize>) -> Run {
    Run { freed: vec![], live, frame: 4096 }
}
#[test]
fn persistent_unique_size_is_detected() {
    let host: Vec<_> = (0..5).map(|_| run(vec![64])).collect();
    assert_eq!(own_blocks(&host, &[]), BTreeSet::from([(true, 64)]));
}
#[test]
fn intermittent_unique_copy_is_discarded() {
    let mut host: Vec<_> = (0..5).map(|_| run(vec![])).collect();
    host[4].live.push(64);
    assert!(own_blocks(&host, &[]).is_empty());
}
#[test]
fn persistent_copy_matching_any_baseline_size_is_discarded() {
    let host: Vec<_> = (0..5).map(|_| run(vec![64])).collect();
    let mut baseline: Vec<_> = (0..5).map(|_| run(vec![])).collect();
    baseline[0].live.push(64);
    assert!(own_blocks(&host, &[&baseline]).is_empty());
}
#[test]
fn extra_copy_multiplicity_is_discarded() {
    let host: Vec<_> = (0..5).map(|_| run(vec![64, 64])).collect();
    let baseline: Vec<_> = (0..5).map(|_| run(vec![64])).collect();
    assert!(own_blocks(&host, &[&baseline]).is_empty());
}
'''
rust_path = HERE / "knot_residue_classifier_probe.rs"
rust_path.write_text(harness, encoding="utf-8")
output = Path("C:/t/cargo-targets/mere/authority-success-residue-classifier.exe")
pdb = output.with_suffix(".pdb")
assert output.parent.resolve() == Path("C:/t/cargo-targets/mere").resolve()
assert not output.exists() and not pdb.exists(), "Do not overwrite another native-output owner"
command = ["rustc", "--edition=2024", "--test", str(rust_path), "-C", "debuginfo=0", "-o", str(output)]
build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
build_log = build.stdout + build.stderr
(HERE / "knot_residue_classifier_build.txt").write_text(build_log, encoding="utf-8")
assert build.returncode == 0, build_log
tested = subprocess.run([str(output), "--nocapture"], cwd=ROOT, capture_output=True, text=True)
(HERE / "knot_residue_classifier_output.txt").write_text(tested.stdout + tested.stderr, encoding="utf-8")
binary_sha = hashlib.sha256(output.read_bytes()).hexdigest()
assert output.read_bytes().startswith(b"MZ"), "Expected generated PE output"
output.unlink()
if pdb.exists():
    assert pdb.read_bytes().startswith(b"Microsoft C/C++ MSF"), "Expected generated PDB"
    pdb.unlink()
receipt = {
    "date_utc": datetime.now(timezone.utc).isoformat(),
    "kind": "Exact source classifier executed natively against synthetic allocation observations; not Knot/transport execution or actual seed exposure",
    "source": str(SOURCE.relative_to(ROOT)).replace("\\", "/"),
    "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
    "classifier_sha256": hashlib.sha256(classifier.encode()).hexdigest(),
    "harness_sha256": hashlib.sha256(rust_path.read_bytes()).hexdigest(),
    "native_binary_sha256": binary_sha,
    "build_command": command,
    "build_exit": build.returncode,
    "test_exit": tested.returncode,
    "observations": ["positive persistent unique size detected", "intermittent unique copy discarded", "persistent copy matching one baseline size discarded", "extra same-size copy multiplicity discarded"],
    "generated_native_output_removed": not output.exists() and not pdb.exists(),
}
(HERE / "knot_residue_classifier_receipt.json").write_text(json.dumps(receipt, indent=2)+"\n", encoding="utf-8")
print(tested.stdout)
assert tested.returncode == 0, tested.stderr
print(json.dumps({"native_exit": tested.returncode, "generated_output_removed": receipt["generated_native_output_removed"], "scope": receipt["kind"]}))
