#!/usr/bin/env python3
"""Generate an isolated Cargo consumer using the current root patch tables.

The output is deliberately outside the Mere workspace. Pass a new or empty
output directory; existing receipts are never overwritten. The root lock is a
starting point, not evidence that the generated dependency graph is resolved.
"""
import argparse
import json
from pathlib import Path
import re
import shutil
import tomllib

fixture = Path(__file__).resolve().parent
root = fixture.parents[3]
parser = argparse.ArgumentParser()
parser.add_argument("output", type=Path)
args = parser.parse_args()
output = args.output.resolve()
if output == root or root in output.parents:
    parser.error("output must be outside the Mere workspace")
if output.exists() and any(output.iterdir()):
    parser.error("output must be a new or empty directory")
output.mkdir(parents=True, exist_ok=True)
source = (root / "Cargo.toml").read_text()
dependency = tomllib.loads(source)["workspace"]["dependencies"]["djinn"].copy()
if "path" in dependency:
    dependency["path"] = str((root / dependency["path"]).resolve())
dependency["default-features"] = False
dependency["features"] = ["resident", "custody-client"]
def toml_value(value):
    return str(value).lower() if isinstance(value, bool) else json.dumps(value)
djinn_dependency = "{ " + ", ".join(key + " = " + toml_value(value) for key, value in dependency.items()) + " }"
# Root patch sections are top-level tables; preserve their exact entries and
# rewrite relative path values against the root that owns those entries.
sections = re.split(r"(?m)(?=^\[)", source)
patches = []
for section in sections:
    if not section.startswith("[patch."):
        continue
    section = re.sub(r'\bpath\s*=\s*"([^"\n]+)"',
                     lambda m: "path = " + json.dumps(str((root / m[1]).resolve())), section)
    patches.append(section.rstrip())
manifest = '''[workspace]
[package]
name = "combined-resident-consumer-fixture"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
mere-resident = { path = %s, default-features = false }
djinn = %s
graphshell-endpoint = { path = %s, default-features = false, features = ["projection-door", "projection-provider"] }
''' % (json.dumps(str(root / "ports/djinn")), djinn_dependency,
       json.dumps(str(root / "crates/graphshell/graphshell-endpoint")))
manifest += "\n\n".join(["", *patches]) + "\n"
# Fail before writing if extraction accidentally produced invalid TOML.
tomllib.loads(manifest)
(output / "Cargo.toml").write_text(manifest)
(output / "src").mkdir()
shutil.copyfile(fixture / "src/lib.rs", output / "src/lib.rs")
shutil.copyfile(root / "Cargo.lock", output / "Cargo.lock")
print(output)
