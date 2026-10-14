# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Record the final toolkit comparisons without replacing earlier receipts."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib

from PIL import Image, ImageChops


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
NATIVE = HERE / "native"
PREVIOUS = HERE.parent / "combined" / "native"
GATES = HERE.parent.parent / "query_views_20261013"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pixel_difference(a, b):
    with Image.open(a) as left, Image.open(b) as right:
        left, right = left.convert("RGBA"), right.convert("RGBA")
        assert left.size == right.size, (a.name, left.size, right.size)
        delta = ImageChops.difference(left, right)
        extrema = delta.getextrema()
        return {
            "size": list(left.size),
            "different_pixels": sum(any(pixel) for pixel in delta.get_flattened_data()),
            "max_channel_delta": [maximum for _, maximum in extrema],
        }


images = sorted(NATIVE.glob("*.png"))
assert len(images) == 32, len(images)
assert {p.name for p in images} == {p.name for p in PREVIOUS.glob("*.png")}
comparisons = []
for image in images:
    old = PREVIOUS / image.name
    comparisons.append({
        "path": f"native/{image.name}",
        "sha256": digest(image),
        "previous_sha256": digest(old),
        "equal_png_sha256": digest(image) == digest(old),
        **pixel_difference(image, old),
    })
pairs = []
for egui in (p for p in images if p.name.startswith("egui-")):
    iced = NATIVE / egui.name.replace("egui-", "iced-", 1)
    pair = {
        "egui": egui.name,
        "iced": iced.name,
        "equal_png_sha256": digest(egui) == digest(iced),
        **pixel_difference(egui, iced),
    }
    assert pair["different_pixels"] == 0, pair
    pairs.append(pair)
assert len(pairs) == 16

gate_records = json.loads((GATES / "final-native-gates.json").read_text(encoding="utf-8-sig"))
assert len(gate_records) == 10, len(gate_records)
expected_counts = {
    "final-retained-mount": 11,
    "final-frisket-workspace": 7,
    "final-native-mailbox": 1,
    "final-document-polling": 1,
    "final-shared-query": 10,
    "final-owner-query": 1,
    "final-clip-wheel": 13,
    "final-embedding": 6,
}
for gate in gate_records:
    assert gate["exit_code"] == 0, gate
    log = GATES / (gate["name"] + ".log")
    # Raw PowerShell redirection has used both UTF-8 and UTF-16 in old receipts.
    raw = log.read_bytes()
    text = raw.decode("utf-16" if raw.startswith((b"\xff\xfe", b"\xfe\xff")) else "utf-8-sig")
    counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed;", text)
    gate["test_results"] = [{"passed": int(p), "failed": int(f)} for p, f in counts]
    if gate["name"] in expected_counts:
        assert sum(int(p) for p, _ in counts) == expected_counts[gate["name"]], gate
        assert all(int(f) == 0 for _, f in counts), gate
    gate["log_sha256"] = digest(log)

source_paths = [REPO / "Cargo.toml", REPO / "Cargo.lock", GATES / "final_native_gates.ps1", Path(__file__).resolve()]
source_paths += list((REPO / "crates/cambium/cambium-rootstock/src/embedding").glob("*.rs"))
source_paths += [REPO / path for path in [
    "crates/cambium/cambium-rootstock/src/embedding.rs",
    "ports/graphshell/examples/caller_owned_toolkits.rs",
    "ports/graphshell/web/GraphshellSans.ttf",
    "ports/graphshell/docs/receipts/co_op_member.png",
    "ports/graphshell/src/retained_content.rs",
    "ports/graphshell/src/retained_content/producer.rs",
    "ports/graphshell/src/retained_content/view.rs",
    "crates/mere/src/query_view.rs",
    "crates/mere/src/query_view/stroke.rs",
]]
metadata_paths = sorted(GATES.glob("netrender-bridge-*-metadata.json"))
sources = [{"path": p.relative_to(REPO).as_posix(), "sha256": digest(p)} for p in sorted(set(source_paths + metadata_paths))]
freeze_path = HERE / "source-before.json"
freeze = json.loads(freeze_path.read_text(encoding="utf-8-sig"))
for source in freeze["files"]:
    assert digest(REPO / source["path"]) == source["sha256"], source
lock = tomllib.loads((REPO / "Cargo.lock").read_text(encoding="utf-8"))
supplier_sources = sorted({p["source"] for p in lock["package"] if "source" in p and any(name in p["source"] for name in ("/genet", "/netrender", "/vello", "/iced"))})
actual = json.loads((NATIVE / "qualification.json").read_text(encoding="utf-8"))
assert actual["caller_submission"]
assert actual["caller_gpu"]["same_handles_for_both_toolkits"]
assert not actual["caller_gpu"]["physical_device_replaced"]
assert actual["caller_gpu"]["device_count"] == actual["caller_gpu"]["queue_count"] == 1
result = {
    "kind": "final-source-and-native-toolkit-qualification",
    "git_head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
    "supplier_sources": supplier_sources,
    "source_and_metadata": sources,
    "source_freeze_sha256": digest(freeze_path),
    "source_unchanged_during_gates": True,
    "runtime_qualification_sha256": digest(NATIVE / "qualification.json"),
    "adapter": actual["adapter"],
    "caller_gpu": actual["caller_gpu"],
    "gates": gate_records,
    "toolkit_pairs": pairs,
    "previous_artifact_comparisons": comparisons,
    "all_32_previous_pngs_equal": all(p["equal_png_sha256"] for p in comparisons),
    "all_32_previous_rgba_equal": all(p["different_pixels"] == 0 for p in comparisons),
    "open": actual["open"],
}
(HERE / "source-qualification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"toolkit_pairs": len(pairs), "previous_pngs": len(comparisons), "all_32_previous_pngs_equal": result["all_32_previous_pngs_equal"], "all_32_previous_rgba_equal": result["all_32_previous_rgba_equal"], "adapter": actual["adapter"], "source_qualification_sha256": digest(HERE / "source-qualification.json")}, indent=2))
