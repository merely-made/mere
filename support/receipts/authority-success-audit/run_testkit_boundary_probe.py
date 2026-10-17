"""Link the freshly built public testkit and exercise only generated decoys."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import os
import shutil
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path("C:/t/cargo-targets/mere").resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
artifacts = []
dependency_dirs = []
native_searches = []
build_finished = False
for line in (HERE / "djinn_testkit_build_output.jsonl").read_text(encoding="utf-8-sig", errors="replace").splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get("reason") == "build-finished":
        build_finished = item.get("success", False)
    if item.get("reason") == "build-script-executed":
        native_searches.extend(p for p in item.get("linked_paths", []) if p.startswith("native="))
    if item.get("reason") == "compiler-artifact" and item["target"]["name"] == "djinn_testkit":
        artifacts.extend(Path(p) for p in item["filenames"] if p.endswith(".rlib"))
        dependency_dirs.extend(Path(p).parent for p in item["filenames"] if p.endswith(".rmeta"))
assert len(artifacts) == 1, "First complete the fresh testkit library build"
assert build_finished, "Fresh library build must have completed successfully"
assert len(dependency_dirs) == 1
library = artifacts[0]
source = HERE / "testkit_boundary_probe.rs"
binary = TARGET / "authority-success-testkit-probe.exe"
pdb = binary.with_suffix(".pdb")
fixtures = TARGET / "authority-success-testkit-fixtures"
assert not any(p.exists() for p in [binary, pdb, fixtures]), "An earlier generated-output owner exists"
assert fixtures.parent.resolve() == TARGET and binary.parent.resolve() == TARGET
command = ["rustc", "--edition=2024", "--test", str(source), "--extern", "djinn_testkit="+str(library), "-L", "dependency="+str(dependency_dirs[0]), "-C", "debuginfo=0", "-o", str(binary)]
for search in sorted(set(native_searches)):
    command.extend(["-L", search])
built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / "testkit_boundary_build_output.txt").write_text(built.stdout+built.stderr, encoding="utf-8")
assert built.returncode == 0, built.stdout+built.stderr
binary_sha = sha(binary)
fixtures.mkdir()
marker = fixtures / ".authority-audit-generated"
marker.write_text("testkit boundary probe; no Djinn or user state\n", encoding="utf-8")
environment = os.environ.copy()
environment["MERE_AUTHORITY_AUDIT_FIXTURES"] = str(fixtures)
tested = subprocess.run([str(binary), "--test-threads=1", "--nocapture"], cwd=ROOT, env=environment, capture_output=True, text=True, timeout=60)
(HERE / "testkit_boundary_output.txt").write_text(tested.stdout+tested.stderr, encoding="utf-8")
saved = []
for path in sorted((fixtures / "receipts").glob("*/*/*")):
    if path.is_file() and path.name in ["receipt.json", "summary.md"]:
        destination = HERE / ("testkit_"+path.parent.parent.name+"_"+("receipt.json" if path.suffix == ".json" else "summary.txt"))
        shutil.copyfile(path, destination)
        saved.append({"path": destination.name, "sha256": sha(destination)})
# The native process and its read-only environment child have both exited.
# Verify the absolute generated target and marker before removing only our fixtures.
assert fixtures.resolve().parent == TARGET and marker.read_text(encoding="utf-8").startswith("testkit boundary probe;")
shutil.rmtree(fixtures)
assert binary.read_bytes().startswith(b"MZ")
binary.unlink()
if pdb.exists():
    assert pdb.read_bytes().startswith(b"Microsoft C/C++ MSF")
    pdb.unlink()
source_files = [p for p in (ROOT / "crates/system/djinn-testkit").rglob("*") if p.is_file() and p.suffix in [".rs", ".toml"]]
receipt = {
    "date_utc": datetime.now(timezone.utc).isoformat(),
    "kind": "Public current djinn-testkit API on Windows, generated decoy roots and environment-only child; no Djinn/SSH process or real endpoint connection",
    "build_command": command, "build_exit": built.returncode,
    "library_path": str(library), "library_sha256": sha(library),
    "test_exit": tested.returncode, "native_binary_sha256": binary_sha,
    "harness_sha256": sha(source),
    "source_sha256": {str(p.relative_to(ROOT)).replace("\\", "/"): sha(p) for p in source_files},
    "saved_evidence": saved,
    "generated_native_output_and_fixtures_removed": not any(p.exists() for p in [binary, pdb, fixtures]),
}
(HERE / "testkit_boundary_receipt.json").write_text(json.dumps(receipt, indent=2)+"\n", encoding="utf-8")
print(tested.stdout)
assert tested.returncode == 0, tested.stderr
print(json.dumps({"native_exit": tested.returncode, "generated_output_removed": receipt["generated_native_output_and_fixtures_removed"], "scope": receipt["kind"]}))
