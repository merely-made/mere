"""Native public admission diagnostics without a network or source patch."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path("C:/t/cargo-targets/mere").resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
libraries, metadata, native, finished = {}, [], [], False
for line in (HERE / "graphshell_host_admission_build_output.jsonl").read_text(encoding="utf-8-sig", errors="replace").splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get("reason") == "build-finished":
        finished = item.get("success", False)
    if item.get("reason") == "build-script-executed":
        native.extend(p for p in item.get("linked_paths", []) if p.startswith("native="))
    if item.get("reason") == "compiler-artifact":
        name = item["target"]["name"]
        if name in ["graphshell_host", "insigne", "notochord", "personae"]:
            paths = [Path(p) for p in item["filenames"] if p.endswith(".rlib")]
            assert len(paths) == 1, name
            libraries[name] = paths[0]
        if name == "graphshell_host":
            metadata.extend(Path(p).parent for p in item["filenames"] if p.endswith(".rmeta"))
assert finished and len(libraries) == 4 and len(metadata) == 1
policy_source = ROOT / "crates/graphshell/graphshell-host/src/policy_projection.rs"
text = policy_source.read_text(encoding="utf-8")
start = text.index("fn rule(")
opening = text.index("{", start)
depth, end = 1, opening+1
while depth:
    depth += (text[end] == "{")-(text[end] == "}")
    end += 1
extracted = HERE / "n4_policy_rule_extracted.rs"
extracted.write_text(text[start:end]+"\n", encoding="utf-8")
binary = TARGET / "authority-success-host-admission-probe.exe"
pdb = binary.with_suffix(".pdb")
assert binary.parent.resolve() == TARGET
assert not binary.exists() and not pdb.exists(), "Preserve an existing native-output owner"
source = HERE / "host_admission_probe.rs"
command = ["rustc", "--edition=2024", "--test", str(source), "-L", "dependency="+str(metadata[0]), "-C", "debuginfo=0", "-o", str(binary)]
for name, path in libraries.items():
    command.extend(["--extern", name+"="+str(path)])
for path in sorted(set(native)):
    command.extend(["-L", path])
built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / "host_admission_probe_build_output.txt").write_text(built.stdout+built.stderr, encoding="utf-8")
assert built.returncode == 0, built.stdout+built.stderr
binary_sha = sha(binary)
tested = subprocess.run([str(binary), "--nocapture"], cwd=ROOT, capture_output=True, text=True, timeout=30)
(HERE / "host_admission_probe_output.txt").write_text(tested.stdout+tested.stderr, encoding="utf-8")
assert binary.read_bytes().startswith(b"MZ")
binary.unlink()
if pdb.exists():
    assert pdb.read_bytes().startswith(b"Microsoft C/C++ MSF")
    pdb.unlink()
paths = ["crates/graphshell/graphshell-host/src/admission.rs", "crates/graphshell/graphshell-host/src/carrier.rs", "crates/graphshell/graphshell-host/src/lifecycle.rs", "crates/graphshell/graphshell-host/src/policy_projection.rs", "crates/graphshell/graphshell-host/src/webrtc_door.rs", "crates/graphshell/graphshell-host/src/webrtc_join.rs"]
receipt = {"date_utc": datetime.now(timezone.utc).isoformat(),
    "scope": "Current public sans-I/O Graphshell admission under admission-only features, synthetic signed identities and supplied authenticated memory-carrier facts; exact extracted N4 rule helper. No native transport, installed resident, WebRTC join or persistence scenario executed.",
    "build_command": command, "build_exit": built.returncode, "test_exit": tested.returncode,
    "source_sha256": {p: sha(ROOT / p) for p in paths},
    "library_sha256": {name: {"path": str(path), "sha256": sha(path)} for name, path in libraries.items()},
    "harness_sha256": sha(source), "n4_extracted_helper_sha256": sha(extracted), "native_binary_sha256": binary_sha,
    "generated_native_output_removed": not binary.exists() and not pdb.exists()}
(HERE / "host_admission_probe_receipt.json").write_text(json.dumps(receipt, indent=2)+"\n", encoding="utf-8")
print(tested.stdout)
assert tested.returncode == 0, tested.stderr
print(json.dumps({"test_exit": tested.returncode, "generated_output_removed": receipt["generated_native_output_removed"], "scope": receipt["scope"]}))
