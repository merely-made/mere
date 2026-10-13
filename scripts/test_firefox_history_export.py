# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Synthetic source export checks. Never opens a user's browser profile."""
import sqlite3
import unittest

from firefox_history_export import export_visits


class HistoryExportTests(unittest.TestCase):
    def test_branch_parent_identity_and_original_microseconds(self):
        connection = sqlite3.connect(":memory:")
        self.addCleanup(connection.close)
        connection.executescript("""
            create table moz_places(id integer, url text, title text, visit_count integer);
            create table moz_historyvisits(id integer, place_id integer, visit_date integer, visit_type integer, from_visit integer);
            insert into moz_places values (1, 'https://example.org/a', 'A', 1), (2, 'https://example.org/b', 'B', 2);
            insert into moz_historyvisits values (10, 1, 1000001, 2, 0), (11, 2, 1000002, 1, 10), (12, 2, 1000003, 9, 11), (13, 2, 1000004, 1, 999);
        """)
        scope = {"product": "firefox", "profile": "fixture", "generation": "g1"}
        visits = export_visits(connection, {}, scope)
        self.assertEqual([item["visit_id"] for item in visits], ["10", "11", "12", "13"])
        self.assertEqual(visits[1]["parent"], {"kind": "visit", "visit_id": "10"})
        self.assertEqual(visits[2]["parent"], {"kind": "visit", "visit_id": "11"})
        self.assertEqual(visits[3]["parent"]["visit_id"], "999")
        self.assertIsNone(visits[3]["referring_url"])
        self.assertEqual(visits[2]["visited_at_unix_micros"], 1000003)
        self.assertEqual(visits[2]["source_scope"], scope)


if __name__ == "__main__":
    unittest.main()
