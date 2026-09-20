"""R3-C: what Redshank's lockfile would gain from the stack fetch and lose with ureq.

Gain = packages (by name) a netfetcher variant locks that Redshank's lock lacks.
Lose = packages in Redshank's lock reachable only through ureq.
Names, not versions: a second version of a crate already present is reported
separately, because it still costs a build.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REDSHANK = os.path.join(
    os.path.expanduser("~"), "Code", "repos", "woodshed", "ports", "redshank", "Cargo.lock"
)
PROBE_ONLY = {"cost-server", "cost-simple", "cost-stack", "cost-lean"}


def parse(path):
    text = open(path, encoding="utf-8").read()
    packages = {}
    for block in text.split("[[package]]")[1:]:
        name = re.search(r'name = "([^"]+)"', block).group(1)
        version = re.search(r'version = "([^"]+)"', block).group(1)
        deps = re.search(r"dependencies = \[(.*?)\]", block, re.S)
        deps = [d.split(" ") for d in re.findall(r'"([^"]+)"', deps.group(1))] if deps else []
        packages[(name, version)] = deps
    return packages


def resolve(packages, dep):
    name = dep[0]
    if len(dep) > 1:
        return (name, dep[1])
    return next(key for key in packages if key[0] == name)


def reachable(packages, roots, blocked=()):
    seen, stack = set(), list(roots)
    while stack:
        key = stack.pop()
        if key in seen or key[0] in blocked:
            continue
        seen.add(key)
        stack.extend(resolve(packages, dep) for dep in packages[key])
    return seen


redshank = parse(REDSHANK)
roots = [key for key in redshank if key[0].startswith("redshank-")]
with_ureq = reachable(redshank, roots)
without_ureq = reachable(redshank, roots, blocked={"ureq"})
lost = sorted({name for name, _ in with_ureq - without_ureq})

report = {"redshank_lock_packages": len(redshank), "lost_with_ureq": lost}
redshank_names = {name for name, _ in redshank}
for variant in ("stack", "lean"):
    probe = parse(os.path.join(HERE, variant, "Cargo.lock"))
    names = {name for name, _ in probe} - PROBE_ONLY
    new_names = sorted(names - redshank_names)
    new_versions = sorted(
        f"{name} {version}"
        for name, version in probe
        if name in redshank_names and name not in PROBE_ONLY and (name, version) not in redshank
    )
    report[variant] = {
        "probe_lock_packages": len(probe),
        "new_crates": new_names,
        "new_versions_of_present_crates": new_versions,
    }

json.dump(report, sys.stdout, indent=1)
print()
