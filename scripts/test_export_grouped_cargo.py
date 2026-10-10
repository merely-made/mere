# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

import copy
import json
import unittest

from export_grouped_cargo import export


def fixture():
    def package(name):
        return {"id": name, "name": name, "version": "1.0.0", "manifest_path": f"/machine/root/{name}/Cargo.toml", "dependencies": []}
    a, b = package("a"), package("b")
    a["dependencies"] = [
        {"name": "b", "rename": "engine", "req": "^1", "path": "/machine/root/b", "source": None, "kind": None, "optional": True, "target": "cfg(target_arch = \"wasm32\")"},
        {"name": "outside", "req": "*", "source": "registry+https://example.test", "kind": "dev", "optional": False, "target": None},
    ]
    return {"workspace_root": "/machine/root", "workspace_members": ["a", "b"], "packages": [a, b]}


class GroupedCargoExportTests(unittest.TestCase):
    def test_alias_optional_and_target_are_disclosed_without_external_or_local_path_leaks(self):
        result = export([("repo", fixture())], "example.test", set())
        dependencies = [r for r in result["relationships"] if r["kind"] != "contains"]
        self.assertEqual(len(dependencies), 1)
        relation = dependencies[0]
        self.assertEqual(relation["to_occurrence"], "crate:repo:b")
        self.assertIn("as engine", relation["explanation"])
        self.assertIn("optional=True", relation["explanation"])
        self.assertIn("wasm32", relation["explanation"])
        self.assertNotIn("/machine/", json.dumps(result))

    def test_order_independent_revision_and_relationship_identity(self):
        metadata = fixture()
        first = export([("repo", metadata)], "example.test", set())
        metadata["packages"].reverse()
        metadata["packages"][1]["dependencies"].reverse()
        self.assertEqual(export([("repo", metadata)], "example.test", set()), first)

    def test_selection_reports_absent_input_and_does_not_invent_missing_dependencies(self):
        result = export([("repo", fixture())], "example.test", {"repo/a"})
        self.assertEqual(len(result["relationships"]), 2)
        with self.assertRaisesRegex(ValueError, "not disclosed"):
            export([("repo", fixture())], "example.test", {"repo/missing"})
        with self.assertRaisesRegex(ValueError, "duplicate workspace"):
            export([("repo", fixture()), ("repo", fixture())], "example.test", set())

    def test_git_component_links_preserve_declaration_scope_and_build_kind(self):
        left, right = fixture(), copy.deepcopy(fixture())
        right['workspace_root'] = '/machine/other'
        for package in right['packages']:
            package['manifest_path'] = package['manifest_path'].replace('/machine/root', '/machine/other')
            for dependency in package['dependencies']:
                if dependency.get('path'):
                    dependency['path'] = dependency['path'].replace('/machine/root', '/machine/other')
        left["packages"][0]["dependencies"] = [{"name": "b", "req": "=1.2.3", "source": "git+https://github.com/merely-made/other.git?rev=old", "kind": "build", "optional": False, "target": None}]
        result = export([("repo", left), ("other", right)], "example.test", set())
        relation = next(r for r in result["relationships"] if r["kind"] == "declares_build_dependency")
        self.assertEqual(relation["to_occurrence"], "crate:other:b")
        self.assertIn("not evidence", relation["explanation"])


if __name__ == "__main__":
    unittest.main()
