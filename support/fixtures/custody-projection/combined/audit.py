#!/usr/bin/env python3

# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Audit a cargo metadata --locked JSON receipt for the combined fixture."""
import json
from pathlib import Path
import sys

if len(sys.argv) != 2:
    sys.exit("usage: audit.py METADATA.json")
data = json.loads(Path(sys.argv[1]).read_text())
packages = {item["id"]: item for item in data["packages"]}
nodes = {item["id"]: item for item in data["resolve"]["nodes"]}
root = data["resolve"]["root"]
if not root or packages[root]["name"] != "combined-resident-consumer-fixture":
    sys.exit("metadata root is not the combined fixture")

def edge(node, name):
    found = [dep["pkg"] for dep in nodes[node]["deps"] if dep["name"] == name]
    if len(found) != 1:
        sys.exit(f"expected one {name} edge from {packages[node]['name']}: {found}")
    return found[0]

resident = edge(root, "mere_resident")
djinn = edge(root, "djinn")
ostiary = edge(root, "ostiary")
custody = edge(root, "castellan_client")
endpoint = edge(root, "graphshell_door")
if edge(resident, "djinn") != djinn:
    sys.exit("resident and the fixture must share one Djinn package")
if edge(resident, "ostiary") != ostiary or edge(endpoint, "ostiary") != ostiary or edge(djinn, "ostiary") != ostiary:
    sys.exit("resident, endpoint and Djinn must share the fixture's ostiary package")
if edge(resident, "castellan_client") != custody:
    sys.exit("resident and the fixture must share one castellan-client package")
for node, expected in [(djinn, {"resident"}), (ostiary, {"native"}), (custody, {"client"}),
                       (endpoint, {"projection-door", "projection-provider"})]:
    missing = expected - set(nodes[node]["features"])
    if missing:
        sys.exit(f"{packages[node]['name']} missing features: {sorted(missing)}")
if "default" in nodes[resident]["features"]:
    sys.exit("fixture unexpectedly enabled mere-resident default features")
print("fixture direct edges and shared door identities:")
for node in (root, resident, djinn, ostiary, custody, endpoint):
    pkg = packages[node]
    print(f"  {pkg['name']} {pkg['version']} {pkg['manifest_path']}")
    print(f"    features: {', '.join(nodes[node]['features'])}")
print("reachable package cone:")
seen = set()
stack = [root]
while stack:
    node = stack.pop()
    if node in seen:
        continue
    seen.add(node)
    stack.extend(dep["pkg"] for dep in nodes[node]["deps"])
for node in sorted(seen, key=lambda value: (packages[value]["name"], value)):
    pkg = packages[node]
    print(f"  {pkg['name']} {pkg['version']} {pkg['source'] or pkg['manifest_path']}")
