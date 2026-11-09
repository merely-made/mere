#!/usr/bin/env python3

# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Refuse repository-level dependency cycles in the merely-made family.

The plan is design_docs/mere_docs/implementation_strategy/2026-11-06_cycle_breaks_plan.md
(P0, "Keeping cycles out: the check", and D6). Doctrine Stack 3: "Mere consumes
producers, never the reverse".

Two scopes:

* Family-wide, from Cargo manifests. The nodes are workspace roots (D35, "Each root is a
  node"): a manifest belongs to the deepest `[workspace]` directory above it, by path
  prefix, and a repository with one root is one node named after it. Every
  `git = "https://github.com/merely-made/<repo>"` dependency row (dependencies,
  build-dependencies, dev-dependencies, target tables and `workspace = true` inheritance)
  is an edge `owner root -> root holding the named package` tagged normal, build or dev. Optional
  rows count (the all-feature graph, which is what a cycle costs). Normal and build
  cycles fail. Dev-only cycles also fail (D6, "Count them"); `--dev-warn` lists them
  as warnings instead.
* Per lock root, from Cargo.lock. Each lock is one resolved graph; a repository cycle inside
  one lock fails (a lock cannot tell dev from normal edges, so it counts them all).

`support/family-cycles/allowed.toml` lists the ruled back edges (a directed pair of
repositories, covering every root pair between them, or of root nodes) and isolated roots (by path), each citing a decision. The graph is checked with those left
out: any cycle that remains fails, and an entry that sits on no cycle fails too, so the list
only shrinks.

Modes: `--family-dir DIR` reads checkouts from DIR (default: the directory holding this
checkout), `--remote` shallow-clones every `merely-made` repository from GitHub.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile

try:
    import tomllib
except ImportError:  # Python before 3.11
    try:
        import tomli as tomllib
    except ImportError:
        try:
            import pip._vendor.tomli as tomllib
        except ImportError:
            sys.exit("check_family_cycles.py needs Python 3.11+ (tomllib) or the tomli package")

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from family_roots import Roots, repo_of_node  # noqa: E402  (D35 workspace-root logic)

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_ALLOWLIST = ROOT / "support/family-cycles/allowed.toml"
ORG = "merely-made"
REPO_URL = re.compile(
    r"github\.com[/:]" + re.escape(ORG) + r"/([A-Za-z0-9._-]+?)(?:\.git)?(?:[/#?]|$)",
    re.IGNORECASE,
)
SKIP_DIRS = {".git", ".claude", "target", "node_modules", "fixtures", "receipts"}
SKIP_PATHS = {"support/patches"}
DEP_TABLES = {
    "dependencies": "normal",
    "build-dependencies": "build",
    "dev-dependencies": "dev",
    "build_dependencies": "build",
    "dev_dependencies": "dev",
}
STRICT_KINDS = ("normal", "build")


def repo_of_url(url):
    """Normalise a git source URL to a family repository name, or None if external."""
    match = REPO_URL.search(url or "")
    return match.group(1) if match else None


def load_toml(path):
    try:
        return tomllib.loads(path.read_text(encoding="utf-8-sig"))
    except (tomllib.TOMLDecodeError, OSError, UnicodeDecodeError) as error:
        print(f"warning: cannot read {path}: {error}", file=sys.stderr)
        return None


# ---------------------------------------------------------------- repository discovery


def git(*args, cwd=None):
    result = subprocess.run(["git", *args], cwd=cwd, text=True, capture_output=True)
    return result.stdout.strip() if result.returncode == 0 else None


def identify(path):
    """The family name of a checkout: its origin's merely-made name, else its directory name.

    Returns None for a checkout whose origin is some other organisation."""
    origin = git("config", "--get", "remote.origin.url", cwd=path)
    if origin:
        return repo_of_url(origin)
    return path.name


def discover_local(family_dir):
    repos = {}
    for child in sorted(family_dir.iterdir()):
        if not child.is_dir() or child.name.startswith("."):
            continue
        name = identify(child)
        if name is None:
            continue
        if name in repos:
            sys.exit(f"two checkouts of {name}: {repos[name]} and {child}; name one family directory")
        repos[name] = child
    return repos


def discover_remote(workdir, only):
    listing = subprocess.run(
        ["gh", "repo", "list", ORG, "--limit", "300", "--no-archived", "--json", "name"],
        text=True, capture_output=True, check=True,
    ).stdout
    names = sorted(entry["name"] for entry in json.loads(listing))
    if only:
        names = [name for name in names if name in only]
    env = dict(os.environ, GIT_TERMINAL_PROMPT="0")

    def clone(name):
        target = workdir / name
        done = subprocess.run(
            ["git", "clone", "-q", "--depth", "1", f"https://github.com/{ORG}/{name}.git", str(target)],
            text=True, capture_output=True, env=env,
        )
        return name, target, done

    repos = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        for name, target, done in pool.map(clone, names):
            if done.returncode != 0:
                print(f"warning: cannot clone {name}: {done.stderr.strip()}", file=sys.stderr)
                continue
            repos[name] = target
    return repos


# ---------------------------------------------------------------------- manifest scan


def walk(repo_path):
    """Yield (relative dir, filenames) for every directory outside the skipped ones."""
    for dirpath, dirnames, filenames in os.walk(repo_path):
        relative = os.path.relpath(dirpath, repo_path).replace(os.sep, "/")
        relative = "" if relative == "." else relative
        dirnames[:] = sorted(
            d for d in dirnames
            if d not in SKIP_DIRS and ((relative + "/" + d) if relative else d) not in SKIP_PATHS
        )
        yield relative, filenames


def line_of(text, name, table_hint):
    """Best-effort 1-based line of a dependency row, for the report."""
    patterns = [
        rf'^\s*"?{re.escape(name)}"?\s*[.=]',
        rf'^\s*\[[^\]]*{re.escape(table_hint)}[^\]]*\.\s*"?{re.escape(name)}"?\s*\]',
    ]
    for index, line in enumerate(text.splitlines(), 1):
        if any(re.search(pattern, line) for pattern in patterns):
            return index
    return 0


def dep_tables(data):
    """Yield (kind, table name, {dep name: spec}) for every dependency table of a manifest."""
    for table, kind in DEP_TABLES.items():
        if isinstance(data.get(table), dict):
            yield kind, table, data[table]
    for cfg, body in (data.get("target") or {}).items():
        if not isinstance(body, dict):
            continue
        for table, kind in DEP_TABLES.items():
            if isinstance(body.get(table), dict):
                yield kind, f"target.{cfg}.{table}", body[table]


def scan_repo(name, repo_path):
    """Return the manifests of a repository: [(relative path, data, text)]."""
    manifests = []
    for relative, filenames in walk(repo_path):
        if "Cargo.toml" in filenames:
            path = repo_path / relative / "Cargo.toml"
            data = load_toml(path)
            if data is not None:
                manifests.append(((relative + "/" if relative else "") + "Cargo.toml", data,
                                  path.read_text(encoding="utf-8-sig", errors="replace")))
    return manifests


def manifest_edges(name, manifests, roots):
    """Edges between workspace-root nodes, from one repository's git rows: a row naming a
    family repository is an edge from the manifest's root to the root that holds the named
    package, which may be another root of the same repository. Path rows are not edges."""
    workspace_deps = {}
    for relative, data, _ in manifests:
        table = (data.get("workspace") or {}).get("dependencies")
        if isinstance(table, dict):
            workspace_deps[os.path.dirname(relative)] = table

    def inherited(relative, dep_name):
        directory = os.path.dirname(relative)
        while True:
            if directory in workspace_deps and dep_name in workspace_deps[directory]:
                return workspace_deps[directory][dep_name]
            if not directory:
                return None
            directory = os.path.dirname(directory)

    edges = []
    for relative, data, text in manifests:
        source = roots.node(name, os.path.dirname(relative))
        for kind, table, deps in dep_tables(data):
            for dep_name, spec in deps.items():
                if not isinstance(spec, dict):
                    continue
                optional = bool(spec.get("optional"))
                if spec.get("workspace"):
                    base = inherited(relative, dep_name)
                    spec = base if isinstance(base, dict) else {}
                target_repo = repo_of_url(spec.get("git", ""))
                if target_repo is None:
                    continue
                target = roots.of_package(target_repo, spec.get("package", dep_name))
                if target == source:
                    continue
                edges.append({
                    "from": source, "to": target, "kind": kind, "optional": optional,
                    "manifest": f"{name}/{relative}", "dep": dep_name,
                    "line": line_of(text, dep_name, table.split(".")[-1]),
                })
    return edges


# ------------------------------------------------------------------------- lock scan


def lock_roots(repo_path):
    return [
        (relative + "/" if relative else "") + "Cargo.lock"
        for relative, filenames in walk(repo_path) if "Cargo.lock" in filenames
    ]


def lock_edges(name, repo_path, relative_lock, roots):
    """Root-level edges inside one lock: [(from, to, package, dependency)]."""
    data = load_toml(repo_path / relative_lock)
    if data is None:
        return None
    packages = data.get("package") or []

    def owner(package):
        source = package.get("source")
        if source is None:
            return roots.of_package(name, package["name"])
        if source.startswith("git+"):
            repo = repo_of_url(source)
            return None if repo is None else roots.of_package(repo, package["name"])
        return None

    by_name = {}
    for package in packages:
        by_name.setdefault(package["name"], []).append(package)

    edges = []
    for package in packages:
        src = owner(package)
        if src is None:
            continue
        for dep in package.get("dependencies") or []:
            parts = dep.split(" ")
            candidates = by_name.get(parts[0], [])
            if len(parts) > 1:
                candidates = [c for c in candidates if c["version"] == parts[1]]
            if len(parts) > 2:
                source = parts[2].strip("()")
                candidates = [c for c in candidates if c.get("source") == source]
            for candidate in candidates[:1]:
                dst = owner(candidate)
                if dst is not None and dst != src:
                    edges.append((src, dst, package["name"], candidate["name"]))
    return edges


# ----------------------------------------------------------------------------- graphs


def sccs(nodes, edges):
    """Strongly connected components of size above one (Tarjan)."""
    graph = {node: set() for node in nodes}
    for a, b in edges:
        graph.setdefault(a, set()).add(b)
        graph.setdefault(b, set())
    index, low, on_stack, stack, result = {}, {}, set(), [], []
    counter = [0]

    def visit(node):
        index[node] = low[node] = counter[0]
        counter[0] += 1
        stack.append(node)
        on_stack.add(node)
        for nxt in sorted(graph[node]):
            if nxt not in index:
                visit(nxt)
                low[node] = min(low[node], low[nxt])
            elif nxt in on_stack:
                low[node] = min(low[node], index[nxt])
        if low[node] == index[node]:
            component = set()
            while True:
                member = stack.pop()
                on_stack.discard(member)
                component.add(member)
                if member == node:
                    break
            if len(component) > 1:
                result.append(component)

    sys.setrecursionlimit(10000)
    for node in sorted(graph):
        if node not in index:
            visit(node)
    return result


def cyclic_pairs(pairs):
    """The directed repository pairs whose two ends sit in one strongly connected component."""
    pairs = set(pairs)
    comp_of = {}
    for number, component in enumerate(sccs({n for p in pairs for n in p}, pairs)):
        for member in component:
            comp_of[member] = number
    return {(a, b) for a, b in pairs if a in comp_of and comp_of[a] == comp_of.get(b)}


def witnesses(pairs, through=None, limit=8):
    """Shortest cycle through each of the `through` pairs (default: all), deduplicated, shortest first."""
    graph = {}
    for a, b in pairs:
        graph.setdefault(a, []).append(b)
    found = {}
    for a, b in sorted(pairs if through is None else set(pairs) & set(through)):
        previous, queue = {b: None}, [b]
        while queue and a not in previous:
            node = queue.pop(0)
            for nxt in sorted(graph.get(node, [])):
                if nxt not in previous:
                    previous[nxt] = node
                    queue.append(nxt)
        if a not in previous:
            continue
        route, node = [], a
        while node is not None:
            route.append(node)
            node = previous[node]
        route.reverse()  # b ... a
        body = [a] + route[:-1]  # a -> b -> ... -> (the node before a)
        turn = body.index(min(body))
        key = tuple(body[turn:] + body[:turn])
        found[key] = list(key) + [key[0]]
    return sorted(found.values(), key=lambda c: (len(c), c))[:limit]


# -------------------------------------------------------------------------- allowlist


def load_allowlist(path):
    if path is None or not path.exists():
        return {"edge": [], "isolated": []}
    data = load_toml(path) or {}
    for entry in data.get("edge", []):
        if not entry.get("from") or not entry.get("to") or not entry.get("decision") or not entry.get("reason"):
            sys.exit(f"{path}: each [[edge]] needs from, to, decision and reason: {entry}")
    for entry in data.get("isolated", []):
        if not entry.get("root") or not entry.get("decision") or not entry.get("reason"):
            sys.exit(f"{path}: each [[isolated]] needs root, decision and reason: {entry}")
    return {"edge": data.get("edge", []), "isolated": data.get("isolated", [])}


# ----------------------------------------------------------------------------- the check


def analyse(repos, allowlist, dev_warn=False):
    """Scan the repositories and judge the graphs against the allowlist."""
    edges = []
    locks = {}
    scanned = {name: scan_repo(name, path) for name, path in sorted(repos.items())}
    roots = Roots(scanned)
    for name, path in sorted(repos.items()):
        edges.extend(manifest_edges(name, scanned[name], roots))
        for relative_lock in lock_roots(path):
            found = lock_edges(name, path, relative_lock, roots)
            if found is not None:
                locks[f"{name}/{relative_lock}"] = found
    return judge(sorted(repos), edges, locks, allowlist, dev_warn)


def judge(repo_names, edges, locks, allowlist, dev_warn=False):
    isolated = allowlist["isolated"]
    entries = {(e["from"], e["to"]): index for index, e in enumerate(allowlist["edge"])}

    def entry_for(pair):
        """An [[edge]] entry names root nodes or repositories; a repository pair covers
        every root pair between them."""
        if pair in entries:
            return entries[pair]
        return entries.get((repo_of_node(pair[0]), repo_of_node(pair[1])))

    class Allowed:
        def __contains__(self, pair):
            return entry_for(pair) is not None

    allowed = Allowed()

    def isolated_for(edge):
        return [i for i, e in enumerate(isolated) if edge["manifest"].startswith(e["root"].rstrip("/") + "/")]

    def pairs_of(rows, kinds):
        return {(e["from"], e["to"]) for e in rows if e["kind"] in kinds}

    everything = ("normal", "build", "dev")
    included = [e for e in edges if not isolated_for(e)]

    def remaining(rows, kinds):
        return {p for p in pairs_of(rows, kinds) if p not in allowed}

    strict_left = cyclic_pairs(remaining(included, STRICT_KINDS))
    all_left = cyclic_pairs(remaining(included, everything))
    dev_left = all_left - strict_left

    # Which allowlist entries sit on a cycle? Judge with everything present.
    used = set()
    full = cyclic_pairs(pairs_of(included, everything))
    for pair in full:
        index = entry_for(pair)
        if index is not None:
            used.add(("edge", index))
    full_with_isolated = cyclic_pairs(pairs_of(edges, everything))
    for index, entry in enumerate(isolated):
        root = entry["root"].rstrip("/") + "/"
        mine = {(e["from"], e["to"]) for e in edges if e["manifest"].startswith(root)}
        if mine & full_with_isolated:
            used.add(("isolated", index))
    stale = []
    for index, entry in enumerate(allowlist["edge"]):
        if ("edge", index) not in used:
            stale.append(f"edge {entry['from']} -> {entry['to']} ({entry['decision']})")
    for index, entry in enumerate(isolated):
        if ("isolated", index) not in used:
            stale.append(f"isolated {entry['root']} ({entry['decision']})")

    lock_cycles = {}
    for root, rows in sorted(locks.items()):
        left = cyclic_pairs({(a, b) for a, b, _, _ in rows if (a, b) not in allowed})
        if left:
            lock_cycles[root] = left

    def rows_for(pairs, kinds):
        return sorted((e for e in included if (e["from"], e["to"]) in pairs and e["kind"] in kinds),
                      key=lambda e: (e["from"], e["to"], e["manifest"], e["dep"]))

    return {
        "repos": repo_names,
        "edges": edges,
        "strict": strict_left,
        "strict_rows": rows_for(strict_left, STRICT_KINDS),
        "strict_witness": witnesses(remaining(included, STRICT_KINDS) & strict_left),
        "dev": dev_left,
        "dev_rows": rows_for(dev_left, ("dev",)),
        "dev_witness": witnesses(remaining(included, everything), through=dev_left),
        "dev_is_warning": dev_warn,
        "lock_cycles": lock_cycles,
        "stale": stale,
        "lock_roots": len(locks),
    }


def describe(edge):
    where = f"{edge['manifest']}:{edge['line']}" if edge.get("line") else edge["manifest"]
    optional = ", optional" if edge.get("optional") else ""
    return f"{edge['from']} -> {edge['to']} [{edge['kind']}{optional}] {edge['dep']} ({where})"


def report(result, heads, verbose):
    out = [f"family repositories scanned: {len(result['repos'])}; lock roots read: {result['lock_roots']}; "
           f"cross-repository manifest edges: {len(result['edges'])}"]
    if heads:
        out.append("heads: " + ", ".join(f"{n}@{s[:9]}" for n, s in sorted(heads.items()) if s))

    def section(title, pairs, rows, witness):
        if not pairs:
            return
        out.append(f"{title} ({len({n for p in pairs for n in p})} repositories: "
                   + ", ".join(sorted({n for p in pairs for n in p})) + ")")
        out.append("  shortest cycles:")
        out.extend("    " + " -> ".join(cycle) for cycle in witness)
        out.append("  closing rows (first per repository pair; --verbose for all):")
        by_pair = {}
        for edge in rows:
            by_pair.setdefault((edge["from"], edge["to"]), []).append(describe(edge))
        for pair, lines in sorted(by_pair.items()):
            shown = lines if verbose else lines[:1]
            out.extend(f"    {line}" for line in shown)
            if len(lines) > len(shown):
                out.append(f"    ... {len(lines) - len(shown)} more {pair[0]} -> {pair[1]} rows")

    section("FAIL normal/build cycle(s) after the allowlist", result["strict"], result["strict_rows"],
            result["strict_witness"])
    if result["dev"]:
        title = ("warning dev-only cycle(s) (--dev-warn)" if result["dev_is_warning"]
                 else "FAIL dev-only cycle(s) (D6: dev edges count)")
        section(title, result["dev"], result["dev_rows"], result["dev_witness"])
    for root, pairs in sorted(result["lock_cycles"].items()):
        names = sorted({n for p in pairs for n in p})
        out.append(f"{'FAIL' if result.get('strict_locks') else 'warning'} resolved cycle inside {root}: "
                   + ", ".join(names))
    if result["stale"]:
        out.append("FAIL allowlist entries on no cycle (delete them):")
        out.extend(f"  {entry}" for entry in result["stale"])
    if not (result["strict"] or result["dev"] or result["lock_cycles"] or result["stale"]):
        out.append("ok: no repository cycle beyond the allowlist")
    return "\n".join(out)


def exit_code(result, strict_locks=False):
    if result["strict"] or result["stale"]:
        return 1
    if result["dev"] and not result["dev_is_warning"]:
        return 1
    if strict_locks and result["lock_cycles"]:
        return 1
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--family-dir", type=pathlib.Path,
                        help="directory of family checkouts (default: the directory holding this checkout)")
    parser.add_argument("--remote", action="store_true", help="shallow-clone every merely-made repository")
    parser.add_argument("--repo", action="append", default=[], help="limit to these repositories (repeatable)")
    parser.add_argument("--allowlist", type=pathlib.Path, default=DEFAULT_ALLOWLIST)
    parser.add_argument("--dev-warn", action="store_true", help="report dev-only cycles as warnings")
    parser.add_argument("--strict-locks", action="store_true",
                        help="fail on a cycle inside one Cargo.lock (default: warn; a stale pin shows as one)")
    parser.add_argument("--verbose", action="store_true", help="print every closing row")
    parser.add_argument("--json", action="store_true", help="print the edges and findings as JSON")
    args = parser.parse_args(argv)

    allowlist = load_allowlist(args.allowlist)
    temp = None
    if args.remote:
        temp = tempfile.TemporaryDirectory(prefix="family-cycles-")
        repos = discover_remote(pathlib.Path(temp.name), set(args.repo))
    else:
        family_dir = (args.family_dir or ROOT.parent).resolve()
        repos = discover_local(family_dir)
        if args.repo:
            repos = {n: p for n, p in repos.items() if n in set(args.repo)}
        if not repos:
            print(f"no family checkouts under {family_dir}; skipping the family-wide scope", file=sys.stderr)
            return 0
    heads = {name: git("rev-parse", "HEAD", cwd=path) for name, path in repos.items()}
    result = analyse(repos, allowlist, args.dev_warn)
    result["strict_locks"] = args.strict_locks
    if args.json:
        print(json.dumps(result, indent=2, default=sorted))
    else:
        print(report(result, heads, args.verbose))
    if temp:
        temp.cleanup()
    return exit_code(result, args.strict_locks)


if __name__ == "__main__":
    sys.exit(main())
