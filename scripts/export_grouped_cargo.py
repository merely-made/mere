#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Export named Cargo metadata snapshots as a grouped S1 host dataset.

Input: --workspace repo=metadata.json, produced by cargo metadata --no-deps.
Additional standalone workspaces can use repo/workspace-name=metadata.json.
--package repo/package selects a bounded subset; otherwise all disclosed
workspace members are included. This exports manifest declarations, including
optional/build/dev dependencies, NOT a resolved or compiled dependency closure.
Only endpoints present in the input selection receive edges. No subprocess,
network access, manifest mutation, or machine-local paths appear in the output.
"""

import argparse
import hashlib
import json
from pathlib import Path
from urllib.parse import urlsplit


def export(workspaces, authority, selected):
    occurrences, relations, packages, owners = {}, [], {}, {}

    def occurrence(identity, label, kind):
        occurrences[identity] = {
            "occurrence_id": identity,
            "source": {"adapter": "cargo.manifest-components/v1", "id": identity},
            "values": {key: {"kind": "text", "value": value} for key, value in {
                "occurrence_id": identity, "label": label, "component_kind": kind,
                "coverage": "Manifest declarations among selected workspace members; optional, build, dev and target-specific declarations included; external and compiled closure excluded.",
            }.items()},
        }

    def relation(identity, parent, child, kind, explanation, evidence):
        label = {"contains": "Contains", "declares_normal_dependency": "Declares a dependency on",
            "declares_build_dependency": "Declares a build dependency on",
            "declares_dev_dependency": "Declares a development dependency on"}.get(kind, kind)
        relations.append({"id": identity, "from_occurrence": parent, "to_occurrence": child,
            "kind": kind, "label": label, "explanation": explanation,
            "provenance": {"source": source, "source_revision": "", "method": "cargo.metadata-manifest-declarations",
                "method_version": 1, "provider": "export_grouped_cargo.py",
                "evidence": [{"adapter": "cargo.manifest-components/v1", "id": evidence}]}})

    source = {"authority": authority, "domain": "cargo-manifest-components", "resource": "selected-workspaces"}
    seen_scopes = set()
    for scope, metadata in workspaces:
        if scope in seen_scopes:
            raise ValueError(f"duplicate workspace scope {scope}")
        seen_scopes.add(scope)
        repo, _, workspace = scope.partition("/")
        workspace = workspace or "root"
        repo_id, workspace_id = f"repo:{repo}", f"workspace:{repo}:{workspace}"
        members = set(metadata["workspace_members"])
        root = Path(metadata["workspace_root"]).resolve()
        included = [p for p in metadata["packages"] if p["id"] in members and
            (not selected or f"{repo}/{p['name']}" in selected)]
        if not included:
            continue
        occurrence(repo_id, repo, "repository")
        occurrence(workspace_id, f"{repo} workspace" if workspace == "root" else f"{repo} / {workspace} workspace", "workspace")
        relation(f"contains:{workspace_id}", repo_id, workspace_id, "contains",
            f"Named input scope {scope} discloses this Cargo workspace.", workspace_id)
        for package in included:
            identity = f"crate:{repo}:{package['name']}"
            if identity in packages:
                raise ValueError(f"duplicate package identity {identity}; choose one snapshot")
            manifest = Path(package["manifest_path"]).resolve()
            relative = manifest.relative_to(root).as_posix()
            occurrence(identity, package["name"], "crate")
            packages[identity] = (repo, package, relative)
            if manifest.parent in owners and owners[manifest.parent] != identity:
                raise ValueError(f"one manifest directory is attributed to multiple repositories: {identity}")
            owners[manifest.parent] = identity
            relation(f"contains:{identity}", workspace_id, identity, "contains",
                f"Cargo workspace_members includes {package['name']} {package['version']} at {relative}.", f"{scope}/{relative}")

    if selected:
        found = {f"{repo}/{package['name']}" for repo, package, _ in packages.values()}
        if missing := selected - found:
            raise ValueError(f"selected packages not disclosed: {', '.join(sorted(missing))}")
    if not packages:
        raise ValueError("no workspace packages disclosed")
    for identity, (repo, package, manifest) in sorted(packages.items()):
        for dependency in package["dependencies"]:
            endpoint = None
            if dependency.get("path"):
                endpoint = owners.get(Path(dependency["path"]).resolve())
            elif (origin := dependency.get("source", "") or "").startswith("git+"):
                parsed = urlsplit(origin[4:])
                if parsed.hostname == "github.com" and parsed.path.startswith("/merely-made/"):
                    owner = parsed.path.split("/")[2].removesuffix(".git")
                    candidate = f"crate:{owner}:{dependency['name']}"
                    if candidate in packages:
                        endpoint = candidate
            if endpoint is None or endpoint == identity:
                continue
            kind = dependency.get("kind") or "normal"
            explanation = (f"{repo}/{manifest} declares {dependency['name']} {dependency['req']} "
                f"as {dependency.get('rename') or dependency['name']}; kind={kind}, "
                f"optional={dependency['optional']}, target={dependency.get('target') or 'all'}. "
                f"Declared source: {dependency.get('source') or 'local path in the named input snapshots'}. "
                "This is a declaration, not evidence that these snapshot versions compile together.")
            selector = [kind, dependency.get("rename") or dependency["name"], dependency.get("target")]
            selector_hash = hashlib.sha256(json.dumps(selector, separators=(",", ":")).encode()).hexdigest()[:16]
            relation(f"declares:{identity}:{selector_hash}:{endpoint}", identity, endpoint,
                f"declares_{kind}_dependency", explanation, f"{repo}/{manifest}")
    relations.sort(key=lambda r: r["id"])
    fields = {name: "text" for name in ["component_kind", "coverage", "label", "occurrence_id"]}
    material = {"source": source, "fields": fields, "occurrences": [occurrences[k] for k in sorted(occurrences)], "relationships": relations}
    revision = "manifest-sha256:" + hashlib.sha256(json.dumps(material, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    for r in relations:
        r["provenance"]["source_revision"] = revision
    return {"schema": "scenomise.host-dataset/v1",
        "dataset": {"source": source, "revision": revision, "fields": fields, "occurrences": material["occurrences"]},
        "relationships": relations, "revisions": [{"sequence": 1, "revision": revision}]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", action="append", required=True, metavar="REPO=METADATA_JSON")
    parser.add_argument("--package", action="append", default=[], metavar="REPO/PACKAGE")
    parser.add_argument("--authority", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    workspaces = []
    for value in args.workspace:
        scope, separator, filename = value.partition("=")
        if not separator or not scope.strip():
            parser.error("workspace must name REPO=METADATA_JSON")
        workspaces.append((scope, json.loads(Path(filename).read_text())))
    try:
        result = export(workspaces, args.authority, set(args.package))
    except (ValueError, KeyError) as error:
        parser.error(str(error))
    text = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    if len(text.encode()) > 1024 * 1024:
        parser.error("dataset exceeds the S1 byte budget; select fewer packages")
    args.output.write_text(text)
    print(f"{len(result['dataset']['occurrences'])} occurrences, {len(result['relationships'])} relationships; {len(text.encode())} bytes")


if __name__ == "__main__":
    main()
