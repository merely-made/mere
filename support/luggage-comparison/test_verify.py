# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Controls for explicit historical-source verification."""
import contextlib
import io
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

import verify


class HistoricalSource(unittest.TestCase):
    def test_snapshot_is_explicit_and_independent_of_current_source(self):
        output = io.StringIO()
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(verify, "REPO", Path(directory)), contextlib.redirect_stdout(output):
                verify.verify(verify.INPUTS / "retained-source")
        self.assertIn("explicit historical source snapshot", output.getvalue())

    def test_changed_retained_source_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            snapshot = Path(directory) / "source"
            shutil.copytree(verify.INPUTS / "retained-source", snapshot)
            source = snapshot / "crates/system/luggage/src/lib.rs"
            source.write_bytes(source.read_bytes() + b"\n// changed\n")
            with self.assertRaisesRegex(AssertionError, "lib.rs"):
                verify.verify(snapshot)


if __name__ == "__main__":
    unittest.main()
