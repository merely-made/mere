#!/usr/bin/env python3

# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Workspace roots as graph nodes for check_family_cycles.py (D35, Python 3.11+)."""

from __future__ import annotations

import os


class Roots:
    """Each workspace root is a node (D35, Mark 2026-11-09: "Each root is a node").

    A manifest belongs to the deepest directory at or above it whose Cargo.toml has a
    `[workspace]` table, matched by path prefix. The node of a repository's top root (or of
    a manifest under no workspace) is the repository name; a nested root is
    `<repository>/<directory>`. A dependency row names a package; its node is the root that
    holds that package, or the repository name when no scanned manifest declares it."""

    def __init__(self, scanned):
        self.dirs = {}
        self.packages = {}
        for name, manifests in scanned.items():
            dirs = {os.path.dirname(rel) for rel, data, _ in manifests if "workspace" in data}
            self.dirs[name] = sorted(dirs, key=len, reverse=True)
            for rel, data, _ in manifests:
                package = (data.get("package") or {}).get("name")
                if package:
                    self.packages.setdefault((name, package), self.node(name, os.path.dirname(rel)))

    def node(self, name, directory):
        for root in self.dirs.get(name, []):
            if root and (directory == root or directory.startswith(root + "/")):
                return f"{name}/{root}"
        return name

    def of_package(self, repo, package):
        return self.packages.get((repo, package), repo)


def repo_of_node(node):
    return node.split("/", 1)[0]
