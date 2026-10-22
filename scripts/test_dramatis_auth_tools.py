# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "dramatis_auth_freeze", Path(__file__).resolve().parents[1] / "support/dramatis_auth_freeze.py"
)
freeze = importlib.util.module_from_spec(spec)
spec.loader.exec_module(freeze)


class AuthenticationArtifactTests(unittest.TestCase):
    def test_windows_drive_paths_keep_their_backslashes_and_make_escaped_spaces(self):
        source = r"C:\t\app.exe: C:\Users\mark_\main.rs C:\Program\ Files\café\lib.rs" + "\n"
        self.assertEqual(
            freeze.depfile_sources(source),
            [r"C:\Users\mark_\main.rs", "C:\\Program Files\\café\\lib.rs"],
        )

    def test_posix_make_escapes_and_continued_rules_keep_source_identity(self):
        source = "/tmp/app: /repo/a\\ b.rs /repo/c\\\\d.rs \\\n /repo/café.rs\n\n/repo/a\\ b.rs:\n"
        self.assertEqual(
            freeze.depfile_sources(source),
            ["/repo/a b.rs", "/repo/c\\d.rs", "/repo/café.rs"],
        )

    def test_windows_executable_format_checks_the_header_and_architecture(self):
        data = bytearray(160)
        data[:2] = b"MZ"
        data[60:64] = (80).to_bytes(4, "little")
        data[80:84] = b"PE\0\0"
        data[84:86] = (0x8664).to_bytes(2, "little")
        data[104:106] = (0x20b).to_bytes(2, "little")
        with tempfile.TemporaryDirectory() as directory, patch.object(freeze.platform, "system", return_value="Windows"):
            path = Path(directory) / "fixture.exe"
            path.write_bytes(data)
            self.assertEqual(freeze.binary_format(path), "PE32+ x86-64 Windows executable")
            data[84:86] = (0xaa64).to_bytes(2, "little")
            path.write_bytes(data)
            with self.assertRaisesRegex(AssertionError, "architecture"):
                freeze.binary_format(path)
            path.write_bytes(b"not an executable")
            with self.assertRaisesRegex(AssertionError, "executable"):
                freeze.binary_format(path)


if __name__ == "__main__":
    unittest.main()
