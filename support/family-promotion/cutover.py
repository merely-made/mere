# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Redirect actual manifest suppliers, preserving unrelated formatting and work."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

BASE = Path(r"C:\Users\mark_\Code\repos")
MERE = BASE / "mere"
HERE = MERE / "support/family-promotion"


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root, text=True).strip()


def scalar(value):
    if isinstance(value, bool):
        return str(value).lower()
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ", ".join(scalar(v) for v in value) + "]"
    return str(value)


def inline(value):
    return "{ " + ", ".join(f"{key} = {scalar(v)}" for key, v in value.items()) + " }"


def write_guarded(path, before, after, changes):
    if before == after:
        return
    if path.read_bytes() != before:
        raise RuntimeError(f"Concurrent change detected: {path}")
    changes.append({"path": str(path), "before_sha256": hashlib.sha256(before).hexdigest(), "after_sha256": hashlib.sha256(after).hexdigest()})
    if not os.environ.get("FAMILY_PROMOTION_DRY_RUN"):
        path.write_bytes(after)


def main():
    revision_file = "dry-revisions.json" if os.environ.get("FAMILY_PROMOTION_DRY_RUN") else "supplier-revisions.json"
    revisions = json.loads((HERE / revision_file).read_text())
    mapping = json.loads((HERE / "package-map.json").read_text())
    paths = {v[1]: key for key, v in mapping.items()}
    changes = []
    statuses = {}
    for root in BASE.iterdir():
        if not (root / ".git").exists() or root.name in ("eidetic", "armillary", "esp", "gemot", "smolweb"):
            continue
        statuses[root.name] = git(root, "status", "--porcelain=v1")
        files = git(root, "ls-files", "--", "Cargo.toml", "**/Cargo.toml").splitlines()
        for rel in files:
            if any(part in rel.split("/") for part in ("receipts", "archive_docs", "vendor", "probes", "fixtures")):
                continue
            if root == MERE and (rel.startswith("support/") or any(rel == src + "/Cargo.toml" for src in paths)):
                continue
            path = root / rel
            if not path.exists():
                continue
            before = path.read_bytes()
            text = before.decode("utf-8-sig")
            lines = text.splitlines(keepends=True)
            out = []
            section = ""
            for line in lines:
                stripped = line.strip()
                if stripped.startswith("["):
                    section = stripped
                if root == MERE and rel == "Cargo.toml" and stripped.strip(',"') in paths:
                    continue
                match = re.match(r'^(\s*)([\w-]+)\s*=\s*(\{.*\})(\s*(?:#.*)?)(\r?\n)?$', line)
                if match and any(kind in section for kind in ("dependencies", "[patch.")):
                    indent, alias, value, comment, ending = match.groups()
                    try:
                        spec = tomllib.loads("x = " + value)["x"]
                    except tomllib.TOMLDecodeError:
                        out.append(line)
                        continue
                    package = spec.get("package", alias)
                    owner = mapping.get(package)
                    dep_path = spec.get("path")
                    dep_git = spec.get("git", "").lower().replace("www.github.com", "github.com").rstrip("/")
                    actual = False
                    if owner and dep_path:
                        target = (path.parent / dep_path).resolve()
                        actual = target == (MERE / owner[1]).resolve()
                    if owner and (actual or dep_git in ("https://github.com/merely-made/mere.git", "https://github.com/merely-made/smolweb.git")):
                        family = owner[0]
                        spec = {k: v for k, v in spec.items() if k not in ("path", "git", "branch", "rev", "tag")}
                        spec.update(git=f"https://github.com/merely-made/{family}.git", rev=revisions[family])
                        line = f"{indent}{alias} = {inline(spec)}{comment}{ending or ''}"
                    elif dep_git == "https://github.com/merely-made/smolweb.git":
                        # Continuing repository identity; individual protocol crates retain names.
                        spec["git"] = "https://github.com/merely-made/errand.git"
                        line = f"{indent}{alias} = {inline(spec)}{comment}{ending or ''}"
                out.append(line)
            after = "".join(out).encode("utf-8")
            # All source-root consumers share one identity for promoted crates,
            # including registry suppliers still naming pre-promotion releases.
            selected = {owner[0] for owner in mapping.values() if f'https://github.com/merely-made/{owner[0]}.git' in after.decode("utf-8")}
            if "https://github.com/merely-made/murm.git" in text:
                selected.add("eidetic")
            if "gemot" in selected:
                selected.update(("eidetic", "errand"))
            if "esp" in selected:
                selected.update(("eidetic", "armillary"))
            if root == MERE and rel == "Cargo.toml":
                selected = {owner[0] for owner in mapping.values()}
            if (rel == "Cargo.toml" or "workspace" in tomllib.loads(after.decode("utf-8"))) and selected:
                promoted = {name: {"git": f"https://github.com/merely-made/{owner[0]}.git", "rev": revisions[owner[0]]} for name, owner in mapping.items() if owner[0] in selected}
                parsed = tomllib.loads(after.decode("utf-8"))
                old_patches = parsed.get("patch", {}).get("crates-io", {})
                additions = {name: spec for name, spec in promoted.items() if name not in old_patches}
                if additions:
                    lines = after.decode("utf-8").splitlines(keepends=True)
                    start = next((i for i, line in enumerate(lines) if line.strip() == "[patch.crates-io]"), None)
                    block = "".join(f"{name} = {inline(spec)}\n" for name, spec in sorted(additions.items()))
                    if start is None:
                        lines.append("\n[patch.crates-io]\n" + block)
                    else:
                        lines.insert(start + 1, "# Canonical promoted families; keep registry and Git consumers type-identical.\n" + block)
                    after = "".join(lines).encode("utf-8")
            write_guarded(path, before, after, changes)
    suffix = "consumer-cutover-dry.json" if os.environ.get("FAMILY_PROMOTION_DRY_RUN") else "consumer-cutover.json"
    ledger = HERE / suffix
    if ledger.exists() and not os.environ.get("FAMILY_PROMOTION_DRY_RUN"):
        previous = json.loads(ledger.read_text())
        statuses = previous["baseline_status"]
        changes = previous["changes"] + changes
    ledger.write_text(json.dumps({"baseline_status": statuses, "changes": changes}, indent=2) + "\n")
    print(f"Updated {len(changes)} manifests; preserved all unrelated lines")


if __name__ == "__main__":
    main()
