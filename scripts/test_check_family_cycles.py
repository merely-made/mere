# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Fixture-graph tests for check_family_cycles.py (synthetic repositories, no network)."""
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "check_family_cycles", Path(__file__).with_name("check_family_cycles.py"))
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


def url(repo, spelling="canonical"):
    return {
        "canonical": f"https://github.com/merely-made/{repo}.git",
        "www": f"https://www.github.com/merely-made/{repo}",
        "ssh": f"git@github.com:merely-made/{repo}.git",
    }[spelling]


def row(repo, spelling="canonical", extra=""):
    return f'{{ git = "{url(repo, spelling)}", branch = "main"{extra} }}'


class FamilyCycleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="family-cycles-")
        self.addCleanup(self.temp.cleanup)
        self.family = Path(self.temp.name) / "family"
        self.family.mkdir()
        self.allowlist = Path(self.temp.name) / "allowed.toml"

    def manifest(self, repo, body, where=""):
        directory = self.family / repo / where
        directory.mkdir(parents=True, exist_ok=True)
        (directory / "Cargo.toml").write_text(f'[package]\nname = "{repo}-crate"\nversion = "0.1.0"\n{body}')

    def allow(self, text):
        self.allowlist.write_text(text)

    def run_check(self, *args):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = checker.main(["--family-dir", str(self.family), "--allowlist", str(self.allowlist), *args])
        return code, out.getvalue()

    def pair(self, a_section="dependencies", b_section="dependencies", **kw):
        self.manifest("a", f"[{a_section}]\nb-crate = {row('b', **kw)}\n")
        self.manifest("b", f"[{b_section}]\na-crate = {row('a', **kw)}\n")

    def test_clean_pair_passes(self):
        self.manifest("a", f"[dependencies]\nb-crate = {row('b')}\n")
        self.manifest("b", "")
        code, out = self.run_check()
        self.assertEqual(code, 0, out)
        self.assertIn("ok:", out)

    def test_two_repository_cycle_fails(self):
        self.pair()
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("a -> b -> a", out)

    def test_cycle_through_a_longer_loop_fails(self):
        self.manifest("a", f"[dependencies]\nb = {row('b')}\n")
        self.manifest("b", f"[dependencies]\nc = {row('c')}\n")
        self.manifest("c", f"[dependencies]\na = {row('a')}\n")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("a -> b -> c -> a", out)

    def test_optional_edge_counts(self):
        self.manifest("a", f"[dependencies]\nb-crate = {row('b', extra=', optional = true')}\n")
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("optional", out)

    def test_build_edge_counts(self):
        self.pair(b_section="build-dependencies")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)

    def test_dev_only_cycle_fails_by_d6_and_warns_on_request(self):
        self.pair(b_section="dev-dependencies")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("D6", out)
        code, out = self.run_check("--dev-warn")
        self.assertEqual(code, 0, out)
        self.assertIn("warning dev-only", out)

    def test_target_table_and_workspace_inheritance(self):
        (self.family / "a").mkdir()
        (self.family / "a/Cargo.toml").write_text(
            f'[workspace]\nmembers = ["m"]\n[workspace.dependencies]\nb-crate = {row("b")}\n')
        (self.family / "a/m").mkdir()
        (self.family / "a/m/Cargo.toml").write_text(
            '[package]\nname = "m"\nversion = "0.1.0"\n[target.\'cfg(unix)\'.dependencies]\nb-crate = { workspace = true }\n')
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("a/m/Cargo.toml", out)

    def test_unused_workspace_row_is_not_an_edge(self):
        (self.family / "a").mkdir()
        (self.family / "a/Cargo.toml").write_text(
            f'[workspace]\nmembers = []\n[workspace.dependencies]\nb-crate = {row("b")}\n')
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n")
        code, out = self.run_check()
        self.assertEqual(code, 0, out)

    def test_spellings_are_one_repository(self):
        self.manifest("a", f"[dependencies]\nb-crate = {row('b', 'www')}\n")
        self.manifest("b", f"[dependencies]\na-crate = {row('a', 'ssh')}\n")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertEqual(checker.repo_of_url("https://WWW.github.com/merely-made/mere.git#abc"), "mere")
        self.assertEqual(checker.repo_of_url("https://github.com/other/mere.git"), None)
        self.assertEqual(checker.repo_of_url("https://github.com/merely-made/mere-extra"), "mere-extra")

    def test_fixture_receipts_and_patches_are_not_scanned(self):
        self.manifest("a", f"[dependencies]\nb-crate = {row('b')}\n")
        self.manifest("b", "")
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n", where="fixtures/x")
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n", where="receipts/x")
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n", where="support/patches/p")
        code, out = self.run_check()
        self.assertEqual(code, 0, out)

    def test_allowed_edge_breaks_the_cycle_and_a_stale_entry_fails(self):
        self.pair()
        self.allow('[[edge]]\nfrom = "a"\nto = "b"\ndecision = "D1"\nreason = "ruled"\n')
        code, out = self.run_check()
        self.assertEqual(code, 0, out)
        # The cycle is gone: the entry now sits on no cycle and must be deleted.
        self.manifest("a", f"[dependencies]\nb-crate = {row('b')}\n")
        self.manifest("b", "")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("edge a -> b (D1)", out)

    def test_isolated_root_stays_out_of_the_union(self):
        self.manifest("a", f"[dependencies]\nb-crate = {row('b')}\n")
        self.manifest("b", f"[dependencies]\na-crate = {row('a')}\n", where="apps/desktop")
        code, _ = self.run_check()
        self.assertEqual(code, 1)
        self.allow('[[isolated]]\nroot = "b/apps/desktop"\ndecision = "D17"\nreason = "own workspace"\n')
        code, out = self.run_check()
        self.assertEqual(code, 0, out)
        # Without the back edge the isolated entry is stale.
        self.manifest("b", "", where="apps/desktop")
        code, out = self.run_check()
        self.assertEqual(code, 1, out)
        self.assertIn("isolated b/apps/desktop", out)

    def test_allowlist_entry_without_decision_is_refused(self):
        self.allow('[[edge]]\nfrom = "a"\nto = "b"\nreason = "no decision"\n')
        self.manifest("a", "")
        with self.assertRaises(SystemExit):
            self.run_check()

    def test_lock_cycle_inside_one_root_warns_and_fails_when_strict(self):
        self.manifest("a", "")
        (self.family / "a/Cargo.lock").write_text(
            'version = 3\n\n[[package]]\nname = "a-crate"\nversion = "0.1.0"\ndependencies = ["b-crate"]\n\n'
            f'[[package]]\nname = "b-crate"\nversion = "0.1.0"\nsource = "git+{url("b")}?branch=main#abc"\n'
            'dependencies = ["a-lib"]\n\n'
            f'[[package]]\nname = "a-lib"\nversion = "0.1.0"\nsource = "git+{url("a")}?branch=main#def"\n')
        self.manifest("b", "")
        code, out = self.run_check()
        self.assertEqual(code, 0, out)
        self.assertIn("warning resolved cycle inside a/Cargo.lock", out)
        code, _ = self.run_check("--strict-locks")
        self.assertEqual(code, 1)

    def test_pair_witness_is_shortest_cycle(self):
        found = checker.witnesses({("a", "b"), ("b", "a"), ("b", "c"), ("c", "a")})
        self.assertEqual(found[0], ["a", "b", "a"])


if __name__ == "__main__":
    unittest.main()
