"""Finish package source coverage, keeping whole-repository audit active."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8'))
def save(n, value):
    (HERE / n).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')
def upsert(items, values, key):
    for value in values:
        at = next((i for i, x in enumerate(items) if x.get(key) == value[key]), None)
        if at is None:
            items.append(value)
        else:
            items[at] = value

old = load('pictograph_geometry_review_receipt.json')
probe = load('pictograph_overlap_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 1 and probe['generated_output_removed']
assert 'test result: ok. 1 passed; 0 failed;' in (HERE / 'pictograph_overlap_probe_output.log').read_text()
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
inventory = load('source_inventory.json')['files']
coverage = load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/canvas/pictograph/')
    and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
added = [p for p in complete if p not in old['source_sha256']]
assert len(complete) == 163 and len(added) == 29
assert sum(inventory[p]['chars'] for p in added) == 317822
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_read = {'path': 'crates/conatus/seiche/Cargo.toml', 'start': 0}
assert coverage.get(next_read['path'], {}).get('through', 0) == 0
notes, authority = load('review_notes.json'), load('authority_map.json')
findings = [{
    'id': 'Q106', 'status': 'documented from fully read source and bounded native diagnostic; not repaired',
    'title': 'Both overlap diagnostics keep a fixed 36px diameter after node resizing',
    'source': 'crates/canvas/pictograph/src/canvas/physics_catalog.rs:474,1328-1503; cartography.rs:node_size; tests/physics_catalog.rs:layout_stats_carry_the_laws_signatures',
    'evidence': 'layout_stats and layout_stats_without_stretch independently compare center distances against 2*NODE_HALF (36px). A native Canvas reports zero overlaps for two 80px bodies 50px apart and one for two 16px bodies 30px apart. Default-size 36px controls agree with CPU body bounds. Each body was measured in its own exact scope because node_body_rect intentionally clips occluded hits; both nodes were restored to full scope before counting. The same fixed-radius diagnostic can report false clearance or false overlap after size overrides. The public field describes pairs closer than a node body diameter, without declaring a fixed default-body proxy.',
    'scope': 'One native CPU observation with four size/distance cases. Body bounds measured separately to avoid occlusion; this is geometric intersection, not raster pixel measurement or a physical contact assertion. Both existing algorithms agree, but their agreement shares the same stale diameter assumption. No failure of default-size Density quality bars is inferred.',
    'verification': 'pictograph_completion_review_receipt.json',
    'diagnostic': 'pictograph_overlap_probe_receipt.json',
    'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'contained metric-contract repair; broader hull-aware overlap is separate',
        'reason': 'Choose and state the measured geometry. For the advertised body-radius proxy, feed both algorithms the resolved per-node half-size and cover unequal sizes. A grid with varying radii needs a matching search neighborhood. Exact hull, visual lift and minimum hit targets are separate contracts; do not silently claim collision truth.'}
}]
upsert(notes['findings'], findings, 'id')
def entry(number, domain, claim, implementations, assessment):
    return {'id': f'A{number:03}', 'domain': domain, 'claim': claim,
        'implementations': implementations, 'assessment': assessment,
        'state': 'Pictograph source fully read; package suites not freshly executed; one native diagnostic',
        'verification': 'pictograph_completion_review_receipt.json'}
entries = [
    entry(129, 'Catalog metadata and shared channel computations',
        'Catalog choices resolve through portable slots and one channel registry.',
        ['Catalog label/id lists repeat enum metadata', 'Parity tests check length, labels, parsing and IDs',
         'All-channel cache tests count actual producer runs and compare cached versus uncached values',
         'Raw-slot resolver acceptance is distinct from refused raw-Hold execution'],
        'Metadata duplication is real and tested for parity. Repetition alone does not establish current drift. Shared computations have positive controls. Do not promote raw parameter-family validation into a claim that raw dynamics terms execute.'),
    entry(130, 'Hide-test keys versus actual displayed link ownership',
        'Closing a selected link hides the displayed relationship without deleting source assertions.',
        ['node_state hide round-trip manually inserts a Surface key after Resource browse links',
         'physics_registry hide revision test similarly constructs a Surface key',
         'relations tests use canonical Resource keys and assert appearances disappear',
         'Exact assertion tests preserve an unhidden parallel assertion and compare source snapshots'],
        'The first two checks cover bookkeeping or revision changes, not disappearance of the actual Resource stroke. This is a localized coverage gap, not evidence that every hide path fails. The later relation tests directly check visible appearance removal and source preservation; those provide a stronger source-level check.'),
    entry(131, 'Test-only picker facade and production spec authority',
        'Tests exercise the same dynamics spec boundary as a host.',
        ['ThroughView reads the live flat choice then edits and installs DynamicsSpec',
         'Its pick helper applies admitted() before set_dynamics_spec and returns overlay refusal afterward',
         'Given partitions use the documented direct composition boundary',
         'Production admission refusal is covered separately by spec and raw-term tests'],
        'ThroughView is a test helper, not another production runtime. A convenience pick can change admitted state and return Err for dropped overlays, so its refusal is not an atomic rejection receipt. Tests using that helper must not stand in for direct production spec-admission tests. No new production mutation defect is inferred.'),
    entry(132, 'Device/model evidence and feature gating',
        'Physics and Meaning can share a host device and model grouping meets the stated fixture bar.',
        ['meaning_device compares engine identity and host-client bytes with a second-device control',
         'Meaning purity run is ignored and requires local models plus adapter',
         'Physics device tests explicitly return with a skip message without an adapter',
         'Partition sweeps print CPU/GPU partition equality; purity receipt asserts F-score tolerance',
         'Vello headless tests assert actual pixels when its feature is enabled'],
        'These sources include concrete device and pixel controls, but ordinary suite green does not prove a gated or skipped device receipt ran. CPU/GPU partition equality is printed rather than asserted in the sweep; score similarity is a narrower claim. No GPU/model/headed run occurred during this continuation.'),
    entry(133, 'Overlap diagnostics and resized geometry',
        'Overlap counts describe pairs closer than node bodies diameter.',
        ['Full all-pairs counter uses constant NODE_HALF', 'Grid counter copies the same fixed diameter',
         'Resolved node_size supports 16..160px overrides', 'Default-size parity tests compare both algorithms'],
        'Q106 demonstrates that two agreeing implementations can share a wrong geometry assumption. Both omit actual per-node size, yielding zero on overlapping enlarged bodies and one on separated small bodies. Keep the diagnostic contract explicit before expanding it to hull or visual-overlap truth.'),
    entry(134, 'Byte-identical projection claim versus checked properties',
        'Identity arrangement projects byte-identically and does not change a pixel.',
        ['underlay test compares ordered node keys and positions', 'It compares only edge vector length',
         'It compares content_bounds.min_x and max_y', 'No serialization, edge payload or rendered pixel comparison'],
        'The test establishes the listed structural properties only. Its name/comment exceed the assertions; it does not establish byte or pixel identity. This source gap is separate from prior Q073, whose client byte-identity failure remains open. No new rendering divergence was reproduced here.'),
    entry(135, 'Restored placement and independent scope authority',
        'Cartography restores geometry, source remains graph-owned, and exact empty scopes remain empty.',
        ['Restore tests seed UUID-keyed geometry through the sidecar',
         'Buffered strategy positions participate in fit-to-content before first frame',
         'Restored-score tests check host recompute gating and release on real inputs',
         'Exact rosters preserve unresolved IDs across graph replacement',
         'Forme integration tests check local holds, explicit pins, atomic malformed extents and graph swap cleanup'],
        'These are meaningful boundary checks, not merely success booleans. Empty scope_to_members clears by documented compatibility semantics; scope_to_exact_members([]) deliberately shows nothing. Preserve that distinction rather than calling both APIs competing graph truth. Source-read assertions were not freshly run.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['source_qualifications'], [{
    'area': 'Pictograph complete source review',
    'note': 'All163 inventoried package source files now fully read, including all tests, manifests and feature-gated sources. This continuation adds29 files and317822 chars. Mere coverage755/3352,11241938/36854820 chars. Full repository review remains active. Package test suites were not executed.'
}, {
    'area': 'Resized overlap diagnostic harness correction',
    'note': 'The first probe assumed node_body_rect returns full unoccluded geometry while both bodies were drawn. It intentionally reports uncovered hits, so an80px body read50px under occlusion. The corrected harness reads each body in isolation, restores full scope, then reads both diagnostic counters. This was a harness assumption error, not another product defect. Final one-test/four-case observation passed; executable/PDB removed.'
}], 'area')
notes['current_area'] = 'Pictograph source complete (163/163); Seiche next'
notes['next_area'] = 'crates/conatus/seiche'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': ['pictograph_completion_review_receipt.json'], 'next_read': next_read,
    'diagnostic_outcomes': 'Q106 reproduced; one native CPU test with four size/distance cases. Pictograph all source read; package suite not run.',
    'workspace': 'Stable C:/t/cargo-targets/mere reused; probe executable/PDB removed; no isolated target, Cargo home or worktree.'}
notes['audit_continuation'] = {'previous_turn': 'progress: Canvas geometry Q104/Q105 and134/163 package files',
    'classification_of_previous_turn': 'progress', 'production_edits_this_turn': False,
    'coverage_checkpoint': stats, 'next_read': next_read,
    'user_instruction': 'Continue full-source audit; preserve concurrent work; do not change production code.',
    'coverage_note': '29 additional files and317822 chars; Mere755/3352; Pictograph163/163. Repository objective remains active.'}
assert len({x['id'] for x in notes['findings']}) == len(notes['findings'])
assert len({x['id'] for x in authority['entries']}) == len(authority['entries'])
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = 'design_docs/mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md'
addition = '''

### Read-only Pictograph source completion, 2026-10-16

All 163 inventoried Pictograph source files are fully read. This continuation
adds 29 files and 317,822 characters, including the remaining Canvas and
feature-gated integration tests. Mere coverage is 755/3352 files and
11,241,938/36,854,820 characters. The repository audit remains active; Seiche
is next. Production is unchanged. Package suites and ignored receipts were
read, not freshly executed.

Q106: both layout_stats overlap implementations retain a fixed 36px diameter.
One native CPU test with four cases reports zero for two 80px bodies 50px
apart and one for two 16px bodies 30px apart; default 36px controls agree with
body geometry. The full and grid counters agree because both copy the same
constant, not because either reads resolved node sizes. The public field
describes node-body diameter without disclosing a default-size proxy. Measure
each body separately to avoid occlusion clipping in node_body_rect, then restore
full scope before reading counts. This establishes a geometric diagnostic
mismatch, not raster overlap or physical-contact truth. The initial probe's
full-body readback assumption was corrected to respect that API contract.

The contained repair is to define the metric and share its resolved radius
inputs. A grid using varying radii also needs an appropriate search range;
changing only the comparison threshold can still miss large bodies. Exact
hulls, visual lift, grammar scale and minimum pointer targets are separate
policies. No default-size Density quality-bar failure is inferred. Broader
geometry unification remains documented so it does not prolong source reading.

Evidence qualifications from the remaining tests:

- node_state's hide round-trip and one physics_registry revision check manually
  insert Surface keys for Resource-owned browse links. They check bookkeeping,
  not disappearance of those actual strokes. relations.rs supplies stronger
  canonical Resource-key disappearance checks, exact assertion filtering and
  source snapshot preservation. The localized test gap is not a claim that
  every hide implementation fails.
- The underlay identity-arrangement test promises byte identity and unchanged
  pixels, but compares node/position tuples, edge count and two bounds. It does
  not compare edge contents, serialized output or pixels. Keep this claim gap
  distinct from the earlier Q073 client failure, which remains open.
- Catalog metadata repeats enum IDs/labels, but parity tests cover the copies.
  Channel tests instrument producer reuse and compare cached/uncached results.
  Raw-slot resolution is explicitly distinct from raw-term execution refusal.
  ThroughView is a test picker facade over DynamicsSpec, not another production
  owner; its admitted partial update plus refusal return is not an atomic
  production rejection receipt.
- Meaning's device receipt has identity, allocation and second-device controls;
  its model/purity run is ignored and needs local artifacts. Physics device
  tests can explicitly skip without an adapter. CPU/GPU partition equality in
  the sweep is printed, while the purity receipt asserts score tolerance.
  Vello headless tests do inspect pixels when enabled. None ran in this pass.
- Restore, exact-scope and Forme tests check real boundary behavior: sidecar
  geometry, pre-frame buffered fitting, preserved restored scores, unresolved
  UUID rosters, explicit empty scopes, local holds and graph-swap cleanup.
  Their independent view authorities should not be mistaken for graph truth.

The native observation reused hash-verified Cargo artifacts. Its owned exe/PDB
were removed; no isolated target, Cargo home or worktree remains. Exact coverage,
preservation hashes and limits are in the
[package completion receipt](../../../support/receipts/authority-success-audit/pictograph_completion_review_receipt.json).
'''
destination = ROOT / doc
before = destination.read_bytes()
assert b'### Read-only Pictograph source completion, 2026-10-16' not in before
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'pictograph_completion_review_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['pictograph_overlap_probe.rs', 'run_pictograph_overlap_probe.py',
    'pictograph_overlap_probe_build.log', 'pictograph_overlap_probe_output.log',
    'pictograph_overlap_probe_receipt.json', 'pictograph_completion_review_diff_check.txt']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Pictograph complete source review; full Mere source review incomplete',
    'production_edits_this_pass': False, 'new_findings': ['Q106'],
    'authority_entries': [x['id'] for x in entries], 'coverage': stats,
    'complete_pictograph_files': complete, 'added_complete_files': added, 'next_read': next_read,
    'source_sha256': {p: sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes': preserved,
    'diagnostic_tests': 1, 'diagnostic_cases': 4, 'diagnostic_exit': 0,
    'package_tests': 'source read only; not run',
    'reused_library_build': old['reused_library_build'],
    'generated_output_removed': probe['generated_output_removed'], 'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before),
        'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['Full repository source objective incomplete; no production repair.',
        'No GPU, models, headed UI, raster measurements or package suite executed.',
        'Body bounds read one at a time to avoid occlusion; counters read at full scope.',
        'Default-size quality bars are not invalidated by resized-body observation.',
        'Previous Q073 and other documented findings remain open.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
save('pictograph_completion_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'added_files': len(added), 'findings': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'next_read': next_read}))
