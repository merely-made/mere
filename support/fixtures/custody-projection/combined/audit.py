#!/usr/bin/env python3
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
endpoint = edge(root, "graphshell_endpoint")
if edge(resident, "djinn") != djinn or edge(endpoint, "djinn") != djinn:
    sys.exit("resident and endpoint must share the fixture's Djinn package")
for node, expected in [(djinn, {"resident", "custody-client"}),
                       (endpoint, {"projection-door", "projection-provider"})]:
    missing = expected - set(nodes[node]["features"])
    if missing:
        sys.exit(f"{packages[node]['name']} missing features: {sorted(missing)}")
if "default" in nodes[resident]["features"]:
    sys.exit("fixture unexpectedly enabled mere-resident default features")
print("fixture direct edges and shared Djinn identity:")
for node in (root, resident, djinn, endpoint):
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
