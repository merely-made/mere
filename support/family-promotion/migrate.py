# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Reproducible, guarded source extraction for the 2026-10-19 family move.

Run from any directory. Preparation never removes donor source. Cutover is a
separate explicit operation after supplier validation and publication.
"""
from __future__ import annotations

import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib
import tomli_w

BASE = Path(r"C:\Users\mark_\Code\repos")
MERE = BASE / "mere"
HERE = MERE / "support/family-promotion"
FAMILIES = {
    "eidetic": {
        **{f"crates/eidetic/{n}": f"crates/{n}" for n in ("muniment", "chartulary", "eidetic-core", "hagiograph")},
        "crates/stickleback": "crates/stickleback",
        "crates/system/luggage": "crates/luggage",
        "crates/system/proofs": "crates/proofs",
        "crates/servitor": "crates/servitor",
    },
    "armillary": {"crates/armillary": "crates/armillary"},
    "esp": {"crates/intel/esp": "crates/esp"},
    "gemot": {f"crates/moot/{n}": f"crates/{n}" for n in ("gemot", "mooting", "mien", "moothold", "commons")},
    "errand": {"crates/system/errand": "crates/errand"},
}
WORKSPACE_SOURCE = HERE / "source-workspace.toml"
ROOT = tomllib.loads((WORKSPACE_SOURCE if WORKSPACE_SOURCE.exists() else MERE / "Cargo.toml").read_text(encoding="utf-8-sig"))
PACKAGES = {}
for fam, paths in FAMILIES.items():
    for src, dest in paths.items():
        manifest = MERE / src / "Cargo.toml"
        if manifest.exists():
            doc = tomllib.loads(manifest.read_text(encoding="utf-8-sig"))
            PACKAGES[doc["package"]["name"]] = (fam, src, dest, doc["package"]["version"])
        else:
            PACKAGES.update(json.loads((HERE / "package-map.json").read_text(encoding="utf-8-sig")))
for member in tomllib.loads((BASE / "smolweb/Cargo.toml").read_text(encoding="utf-8-sig"))["workspace"]["members"]:
    path = BASE / "smolweb" / member / "Cargo.toml"
    if path.exists() and member != "crates/errand":
        doc = tomllib.loads(path.read_text(encoding="utf-8-sig"))["package"]
        PACKAGES[doc["name"]] = ("errand", "@smolweb/" + member, member, doc["version"])


def run(*args, cwd=MERE):
    result = subprocess.run(list(args), cwd=cwd, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    return result.stdout.strip()


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8", newline="\n")


def dep_sections(doc):
    for kind in ("dependencies", "dev-dependencies", "build-dependencies"):
        if kind in doc:
            yield doc[kind]
    for target in doc.get("target", {}).values():
        yield from dep_sections(target)


def canonical_dep(package, local_family, revisions=None):
    fam, src, dest, version = PACKAGES[package]
    if fam == local_family:
        return {"path": dest}
    if revisions and fam in revisions:
        return {"git": f"https://github.com/merely-made/{fam}.git", "rev": revisions[fam]}
    root = BASE / ("smolweb" if fam == "errand" else fam)
    return {"path": (root / dest).as_posix()}


def source_manifest(family, src):
    path = MERE / src / "Cargo.toml"
    if path.exists():
        text = path.read_text(encoding="utf-8-sig")
    else:
        revision = json.loads((HERE / "source-inventory.json").read_text(encoding="utf-8-sig"))["source_revision"]
        text = run("git", "show", f"{revision}:{src}/Cargo.toml")
    doc = copy.deepcopy(tomllib.loads(text))
    if family == "esp":
        doc.get("dev-dependencies", {}).pop("eidetic-search", None)
        doc.setdefault("test", []).extend([
            {"name": "library_session", "required-features": ["decoder", "model-session"]},
            {"name": "smollm2_library", "required-features": ["decoder", "model-session"]},
        ])
    if family == "gemot" and src.endswith("/gemot"):
        doc.get("dev-dependencies", {}).pop("app-host", None)
        doc.get("dev-dependencies", {}).pop("wasmtime", None)
    doc["package"]["repository"] = f"https://github.com/merely-made/{family}"
    return doc


def configure(family, revisions=None):
    root = BASE / ("smolweb" if family == "errand" else family)
    deps = {}
    members = []
    for src, dest in FAMILIES[family].items():
        members.append(dest)
        doc = source_manifest(family, src)
        for section in dep_sections(doc):
            for alias, spec in list(section.items()):
                original = spec if isinstance(spec, dict) else {}
                inherited = ROOT["workspace"]["dependencies"].get(alias, {}) if original.get("workspace") else {}
                inherited = inherited if isinstance(inherited, dict) else {"version": inherited}
                package = original.get("package", inherited.get("package", alias))
                if package in PACKAGES:
                    base = {k: v for k, v in {**inherited, **original}.items() if k not in ("workspace", "path", "git", "branch", "rev", "tag", "optional", "features")}
                    base.update(canonical_dep(package, family, revisions))
                    if inherited.get("features"):
                        base["features"] = copy.deepcopy(inherited["features"])
                    if original.get("default-features") is False:
                        base["default-features"] = False
                    deps[alias] = base
                    keep = {k: v for k, v in original.items() if k in ("features", "optional", "default-features")}
                    section[alias] = {"workspace": True, **keep}
                elif original.get("workspace"):
                    deps[alias] = copy.deepcopy(ROOT["workspace"]["dependencies"][alias])
                    if isinstance(deps[alias], dict) and "path" in deps[alias]:
                        raise RuntimeError(f"{family}: dependency reaches Mere: {alias} = {deps[alias]}")
        write(root / dest / "Cargo.toml", tomli_w.dumps(doc))
    if family == "errand":
        rootdoc = tomllib.loads(run("git", "show", "HEAD:Cargo.toml", cwd=root))
        rootdoc["workspace"]["members"] = sorted(set(rootdoc["workspace"]["members"] + members))
    else:
        rootdoc = {"workspace": {"resolver": "2", "members": members}}
    rootdoc["workspace"]["package"] = {**ROOT["workspace"]["package"], "repository": f"https://github.com/merely-made/{family}"}
    rootdoc["workspace"]["dependencies"] = deps
    if family == "errand":
        for member in rootdoc["workspace"]["members"]:
            manifest = root / member / "Cargo.toml"
            if manifest.exists() and member != "crates/errand":
                package = tomllib.loads(manifest.read_text(encoding="utf-8-sig"))["package"]["name"]
                if package in deps:
                    deps[package] = {**deps[package], "path": member}
    rootdoc["workspace"]["lints"] = ROOT.get("workspace", {}).get("lints", {})
    own = {name: {"path": values[2]} for name, values in PACKAGES.items() if values[0] == family}
    rootdoc.setdefault("patch", {})[f"https://github.com/merely-made/{family}.git"] = own
    patches = {}
    # Existing immutable dependencies may still name foundation packages via Mere.
    for package in PACKAGES if family in ("eidetic", "gemot") else ():
        if PACKAGES[package][0] != "eidetic":
            continue
        value = canonical_dep(package, family, revisions)
        if family == PACKAGES[package][0]:
            value = {"path": PACKAGES[package][2]}
        patches[package] = value
    if patches:
        rootdoc.setdefault("patch", {})["https://github.com/merely-made/mere.git"] = patches
    registry = {}
    for package, value in ROOT.get("patch", {}).get("crates-io", {}).items():
        if family in ("eidetic", "gemot") and (package.startswith("p2panda-") or package == "mere-p2panda-net"):
            registry[package] = copy.deepcopy(value)
    if family in ("eidetic", "esp", "gemot"):
        registry["muniment"] = canonical_dep("muniment", family, revisions)
    if family == "errand":
        registry.update(own)
    elif family == "gemot":
        registry.update({name: canonical_dep(name, family, revisions) for name, values in PACKAGES.items() if values[0] == "errand"})
    if registry:
        rootdoc.setdefault("patch", {})["crates-io"] = registry
    write(root / "Cargo.toml", tomli_w.dumps(rootdoc))
    write(root / ".gitignore", "/target/\n/.cargo/config.local.toml\n")
    write(root / ".gitattributes", "* text=auto\n*.rs text eol=lf\n*.toml text eol=lf\n")
    shutil.copyfile(MERE / "rust-toolchain.toml", root / "rust-toolchain.toml")
    write(root / ".cargo/config.toml", '[build]\nbuild-dir = "{workspace-root}/target/build"\n')


def scaffold():
    summaries = {
        "eidetic": "Durable content, graph history, replication, artifact distribution and participant gates.",
        "armillary": "A host-neutral actor kernel, typed command/update channels and reusable worker pool.",
        "esp": "Portable model execution, inference and embeddings with optional actor and persistence adapters.",
        "gemot": "Community governance, membership, standing and federation over shared identity and replication.",
        "errand": "Small-web protocols and scheme-routed client/server integration, continuing the Smolweb repository.",
    }
    for family in FAMILIES:
        root = BASE / ("smolweb" if family == "errand" else family)
        intro = f"# {family.capitalize()}\n\n{summaries[family]}\n\n"
        intro += "This repository preserves the implementation history promoted from Mere on 2026-10-19. Crates retain their names, APIs, licenses and selectable features.\n\n"
        intro += "Canonical migration: `mere/design_docs/mere_docs/implementation_strategy/2026-10-19_family_promotion_plan.md`.\n\n"
        intro += "| Crate | Source |\n|---|---|\n"
        if family == "errand":
            members = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8-sig"))["workspace"]["members"]
        else:
            members = FAMILIES[family].values()
        for member in members:
            package = tomllib.loads((root / member / "Cargo.toml").read_text(encoding="utf-8-sig"))["package"]
            intro += f"| `{package['name']}` | [{member}]({member}) |\n"
        intro += "\nRun `cargo test --workspace` for default library coverage. Optional network, GPU, external model and host acceptance have separate gates. No registry release is implied by this source promotion.\n"
        if family == "errand":
            intro += "\nThe protocol crates retain their MIT licenses and stewardship offers to protocol authors. Errand retains MPL-2.0; see `crates/errand/LICENSE`. Protocol wire behavior and grammars remain independently consumable.\n"
            shutil.copyfile(MERE / "LICENSE", root / "crates/errand/LICENSE")
        write(root / "README.md", intro)
        write(root / "design_docs/DOC_POLICY.md", (MERE / "design_docs/DOC_POLICY.md").read_text(encoding="utf-8-sig").split("## Local addendum")[0])
        write(root / "design_docs/DOC_README.md", f"# {family.capitalize()} documentation\n\nPreserve host-owned authority, existing formats and independent crate feature boundaries. Read [documentation policy](DOC_POLICY.md) before editing.\n\nCanonical migration and current receipts: `mere/design_docs/mere_docs/implementation_strategy/2026-10-19_family_promotion_plan.md`.\n")
        ci = "name: Library gates\non: [push, pull_request]\njobs:\n  libraries:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: dtolnay/rust-toolchain@1.99.1\n      - run: cargo test --workspace --locked\n      - run: cargo check --workspace --no-default-features --locked\n"
        # Preserve Smolweb's stronger existing protocol conformance workflow.
        if family != "errand":
            write(root / ".github/workflows/libraries.yml", ci)
    # Integration tests are owned by the existing Mere integration packages.
    p = MERE / "crates/intel/eidetic-search/Cargo.toml"
    text = p.read_text(encoding="utf-8-sig")
    if "bert = [" not in text:
        text += '\n[features]\nbert = ["esp/bert"]\n'
    if "sha2.workspace" not in text:
        text = text.replace("[dev-dependencies]", "[dev-dependencies]\nsha2.workspace = true")
    write(p, text)
    p = MERE / "ports/moot/Cargo.toml"
    text = p.read_text(encoding="utf-8-sig")
    additions = '\nmien.workspace = true\ntransport = { workspace = true, features = ["test-identity"] }\napp-host = { path = "../../crates/script/app-host" }\nwasmtime = "45"\nblake3 = "1.5"\ntempfile = "3"\n'
    text += additions
    write(p, text)
    guest = MERE / "ports/moot/examples/capsule-library/guest/src/lib.rs"
    write(guest, guest.read_text(encoding="utf-8-sig").replace("../../../../../script/wit", "../../../../../crates/script/wit"))
    example = MERE / "ports/moot/examples/capsule-library.rs"
    write(example, example.read_text(encoding="utf-8-sig").replace("cargo run -p gemot", "cargo run -p mere-moot").replace("identity::", "personae::"))
    p = MERE / ".gitignore"
    write(p, p.read_text(encoding="utf-8-sig").replace("!/crates/moot/gemot/examples/capsule-library/guest/Cargo.lock", "!/ports/moot/examples/capsule-library/guest/Cargo.lock"))
    index = MERE / "design_docs/DOC_README.md"
    link = "[Foundation family promotion](archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md)"
    text = index.read_text(encoding="utf-8-sig")
    if link not in text:
        text += f"\n\nFamily promotion (2026-10-19): {link} owns extraction, independent library validation and source cutover for Eidetic, Armillary, ESP, Gemot and the continuing Errand/Smolweb family.\n"
    write(index, text)


def clean_extraction_refs():
    """Drop only donor refs copied into newly created extraction repositories."""
    inventory = json.loads((HERE / "source-inventory.json").read_text(encoding="utf-8-sig"))
    for family in ("eidetic", "armillary", "esp", "gemot"):
        root = BASE / family
        if run("git", "rev-parse", "HEAD", cwd=root) != inventory["extractions"][family]:
            raise RuntimeError(f"{family} has advanced; do not change extraction refs")
        for ref in run("git", "for-each-ref", "--format=%(refname)", cwd=root).splitlines():
            if ref != "refs/heads/main":
                run("git", "update-ref", "-d", ref, cwd=root)
        if "origin" in run("git", "remote", cwd=root).splitlines():
            run("git", "remote", "remove", "origin", cwd=root)
        run("git", "reflog", "expire", "--expire=now", "--all", cwd=root)
        run("git", "gc", "--prune=now", cwd=root)
        run("git", "fsck", "--no-dangling", cwd=root)
        print(f"{family}: retained filtered main history only", flush=True)


def prepare():
    if (HERE / "source-inventory.json").exists():
        raise RuntimeError("Preparation already recorded; inspect before repeating")
    sha = run("git", "rev-parse", "HEAD")
    inventory = {"source_revision": sha, "files": {}, "extractions": {}}
    for family, paths in FAMILIES.items():
        for src, dest in paths.items():
            for rel in run("git", "ls-files", "--", src).splitlines():
                inventory["files"][rel] = hashlib.sha256((MERE / rel).read_bytes()).hexdigest()
        if family == "errand":
            continue
        root = BASE / family
        if root.exists():
            raise RuntimeError(f"Destination already exists: {root}")
        print(f"Extracting {family}", flush=True)
        run("git", "clone", "--shared", "--no-checkout", "--single-branch", "--no-tags", "--branch", "main", str(MERE), str(root), cwd=BASE)
        args = [sys.executable, "-m", "git_filter_repo", "--force"]
        for src, dest in paths.items():
            args += ["--path", src + "/", "--path-rename", src + "/:" + dest + "/"]
        args += ["--path", "LICENSE", "--path", "rust-toolchain.toml"]
        run(*args, cwd=root)
        run("git", "reset", "--hard", "HEAD", cwd=root)
        if "origin" in run("git", "remote", cwd=root).splitlines():
            run("git", "remote", "remove", "origin", cwd=root)
        run("git", "repack", "-a", "-d", cwd=root)
        alternates = root / ".git/objects/info/alternates"
        if alternates.exists():
            alternates.unlink()
        run("git", "fsck", "--no-dangling", cwd=root)
        inventory["extractions"][family] = run("git", "rev-parse", "HEAD", cwd=root)
    print("Splitting Errand history", flush=True)
    split = run("git", "subtree", "split", "--prefix=crates/system/errand", sha).splitlines()[-1]
    run("git", "subtree", "add", "--prefix=crates/errand", str(MERE), split, "-m", "Import Errand history into the continuing protocol family", cwd=BASE / "smolweb")
    inventory["extractions"]["errand"] = split
    # Keep cross-library/application compositions with their actual integration owner.
    test = MERE / "crates/intel/esp/tests/lexical_ngram_recall.rs"
    dest = MERE / "crates/intel/eidetic-search/tests/lexical_ngram_recall.rs"
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(test, dest)
    (BASE / "esp/crates/esp/tests/lexical_ngram_recall.rs").unlink()
    example = MERE / "crates/moot/gemot/examples/capsule-library.rs"
    shutil.copyfile(example, MERE / "ports/moot/examples/capsule-library.rs")
    shutil.copytree(example.with_suffix(""), MERE / "ports/moot/examples/capsule-library")
    extracted = BASE / "gemot/crates/gemot/examples/capsule-library"
    shutil.rmtree(extracted)
    extracted.with_suffix(".rs").unlink()
    for family in FAMILIES:
        configure(family)
    write(HERE / "source-inventory.json", json.dumps(inventory, indent=2) + "\n")
    print("Prepared all families; donor implementations remain intact", flush=True)


def finish_prepare():
    """Finish after a direct filtered Errand import, retaining source evidence."""
    sha = run("git", "rev-parse", "HEAD")
    inventory = {"source_revision": sha, "files": {}, "extractions": {}}
    for family, paths in FAMILIES.items():
        for src in paths:
            for rel in run("git", "ls-files", "--", src).splitlines():
                inventory["files"][rel] = hashlib.sha256((MERE / rel).read_bytes()).hexdigest()
        root = BASE / ("smolweb" if family == "errand" else family)
        inventory["extractions"][family] = run("git", "rev-parse", "HEAD", cwd=root)
    source = MERE / "crates/intel/esp/tests/lexical_ngram_recall.rs"
    target = MERE / "crates/intel/eidetic-search/tests/lexical_ngram_recall.rs"
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    (BASE / "esp/crates/esp/tests/lexical_ngram_recall.rs").unlink(missing_ok=True)
    source = MERE / "crates/moot/gemot/examples/capsule-library.rs"
    shutil.copyfile(source, MERE / "ports/moot/examples/capsule-library.rs")
    shutil.copytree(source.with_suffix(""), MERE / "ports/moot/examples/capsule-library", dirs_exist_ok=True)
    target = BASE / "gemot/crates/gemot/examples/capsule-library"
    assert target.resolve().is_relative_to((BASE / "gemot").resolve())
    if target.exists():
        shutil.rmtree(target)
    target.with_suffix(".rs").unlink(missing_ok=True)
    write(HERE / "source-inventory.json", json.dumps(inventory, indent=2) + "\n")


if __name__ == "__main__":
    if (HERE / "supplier-revisions.json").exists():
        raise SystemExit("Completed historical migration: supplier revisions already recorded; do not rerun mutations")
    if sys.argv[1:] == ["prepare"]:
        prepare()
    elif sys.argv[1:] == ["scaffold"]:
        for family in FAMILIES:
            configure(family)
        scaffold()
    elif sys.argv[1:] == ["clean-extraction-refs"]:
        clean_extraction_refs()
    elif sys.argv[1:] == ["finish-prepare"]:
        finish_prepare()
    else:
        raise SystemExit("Expected: migrate.py prepare")
