"""One-shot host lifecycle and receipt-path source-review checkpoint."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'community_host_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
def save(n, v):
    with (HERE / n).open('w', encoding='utf-8', newline='\n') as stream:
        stream.write(json.dumps(v, indent=2) + '\n')

assert not (HERE / NAME).exists()
old = load('community_owner_review_receipt.json')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
assert head == old['basis_commit']
upstream = subprocess.check_output(['git', 'rev-parse', 'origin/main'], cwd=ROOT, text=True).strip()
ahead_behind = subprocess.check_output(['git', 'rev-list', '--left-right', '--count', 'HEAD...origin/main'], cwd=ROOT, text=True).strip()
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1079
assert all(sha(ROOT / n) == h for n, h in guards.items())
assert all(sha(HERE / n) == h for n, h in old['evidence_sha256'].items())
doc = ROOT / old['document']['path']
prefix = doc.read_bytes()
assert hashlib.sha256(prefix).hexdigest() == old['document']['sha256']
assert load('pending_read.json') == [] and not (HERE / 'pending_transition.json').exists()
inv, cov = load('source_inventory.json')['files'], load('read_coverage.json')
whole = sorted([
    'ports/graphshell/src/bin/moot_resources_host.rs',
    *['ports/graphshell/web/applets/' + n + '.mjs' for n in ('host', 'runtime', 'verify', 'wasi', 'worker')],
    *['ports/graphshell/web/' + n for n in ('moot_resource_host.js', 'mount.js', 'loader.js',
      'scenario_qualification_server.py', 'co_op.css', 'co_op.html', 'co_op.js',
      'co_op_gateway.py', 'co_op_receipt.py', 'viewer.js')],
    *['ports/graphshell/web/tests/' + n for n in ('presentation-mount.mjs', 'receipt_sink.py',
      'run-headed-receipt.py', 'run-presentation-webgl.py', 'capture-page.mjs',
      'compare-presentation-captures.py', 'host-input.mjs', 'presentation-webgl-fixture.html')],
    *['ports/graphshell/web/scripts/' + n for n in ('build-viewer.py', 'package.json', 'test_viewer_budget.py')],
    *['ports/graphshell/docs/receipts/moot_resources_host_20261012/harness/' + n
      for n in ('Cargo.toml', 'src/lib.rs')],
])
assert all(n not in guards and cov[n]['sha256'] == inv[n]['sha256'] and cov[n]['through'] == inv[n]['chars'] for n in whole)
assert len(whole) == 29 and sum(inv[n]['chars'] for n in whole) == 178117
assert all(sha(ROOT / n) == inv[n]['sha256'] for n in whole)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 1578 and stats['total_files'] == 3727
assert stats['read_chars'] == 21257175 and not stats['changed_since_inventory'] and not stats['pending_chunks']
diagnostics = ['community_host_lifecycle_probe_receipt.json', 'community_receipt_delivery_probe_v2_receipt.json',
               'community_headed_exit_probe_receipt.json', 'community_sink_path_probe_receipt.json']
assert [load(n)['tests'] for n in diagnostics] == [8, 4, 2, 3]
for n in diagnostics:
    assert all(sha(ROOT / path) == digest for path, digest in load(n)['source_sha256'].items())
for path, info in load('community_headed_exit_probe_receipt.json')['fixture_files'].items():
    assert sha(HERE / path) == info['sha256']
assert load('community_receipt_delivery_probe_initial_failure.json')['tests_executed'] == 0
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q261' and authority['entries'][-1]['id'] == 'A661'
rows = [
    ('Q262', 'Finite native community proof can exit successfully before qualification completes',
     'ports/graphshell/src/bin/moot_resources_host.rs:after_wake/main', 'source-only control-flow defect',
     'The 35-second finite deadline only sets ctx.close. Normal run return is checked for host errors, but no completion flag/result-file assertion survives after run. A deadline or user close before the final stage can therefore return normally without writing a new qualification.json. create_dir_all also permits reuse of an existing receipt directory, so a previous qualification may survive an incomplete new run. This does not invalidate the recorded historical successful run.', [],
     'Contained: require explicit final-stage completion for finite success, refuse receipt-directory reuse and preserve incomplete-run evidence. A fresh headed positive/timeout/early-close control is still required.'),
    ('Q263', 'Closed applet host handles still own review and close operations on later mounts',
     'ports/graphshell/web/applets/host.mjs:review/close; ports/graphshell/src/web_tree/applet.rs:review_applet/applet_close',
     'current JS adapter counterexamples with synthetic callbacks',
     'disposed gates only pump. Three current-module counterexamples show review after close calls the Rust review interface, a pending fetch/review installs after close, and repeated close on an old handle calls global applet_close and removes the WeakSet guard for a live remount. The duplicate-live-root control still refuses before the stale close. Rust review/close entry points are host operations without a prior-handle ownership token; generation checks on guest output do not qualify stale host lifecycle ownership. No forged signature or guest authorization bypass is claimed.',
     [diagnostics[0]],
     'Contained lifecycle correction: make close idempotent, validate live registration ownership before mutations and again after asynchronous preparation, and reject closed-handle review/turn. Bind current host registration rather than using a shared-root boolean alone.'),
    ('Q264', 'Auto renderer fallback drops the visible refusal when its second artifact fails to load',
     'ports/graphshell/web/mount.js:defineElements/_attach MutationObserver',
     'current element-module counterexample with synthetic observer and route supplier',
     'The async observer disposes WebGPU and removes data-mount-error before awaiting source(webgl2), outside connectedCallback error handling. A rejected second load leaves data-mount-error absent and data-mounted true; another connectedCallback returns immediately. The focused diagnostic proves this DOM/latch state. Receipt pages can still catch the resulting unhandled rejection through loader.js; this is not proof that the full scenario gate passes. Original presentation-mount checks all pass.',
     [diagnostics[0]],
     'Contained: handle second-route load/attach failure in the observer with the same epoch checks and visible failure/reset semantics as initial mounting. Preserve disposal ownership across disconnect/remount.'),
    ('Q265', 'Scenario receipt delivery reports HTTP rejection as delivered',
     'ports/graphshell/web/loader.js:graphshell-scenario-complete listener',
     'exact current listener counterexample and controls',
     'The listener awaits fetch but never checks response.ok before setting scenarioSink=delivered. Its exact unchanged handler reports delivered for a synthetic HTTP 500. A 204 control reports delivered, a transport rejection reports failed, and an existing planted page-gate failure still forces the scenario verdict to fail. External drivers waiting for a new scenario.done file do not thereby accept a missing receipt; the defect is the browser delivery claim.',
     [diagnostics[1]],
     'Contained: distinguish successful HTTP acknowledgement from HTTP refusal and transport failure. Do not weaken the existing scenario error gate.'),
    ('Q266', 'Headed receipt driver records a failed scenario but returns normally',
     'ports/graphshell/web/tests/run-headed-receipt.py:main',
     'unchanged current main exercised with synthetic dependencies',
     'After scenario.done appears, main copies scenario.result.result into environment.json, captures the page and prints the file, without raising or returning a failing exit status for fail. Running actual unchanged main against synthetic failed and successful receipts returns normally for both, while preserving the correct metadata verdict. This is a status propagation defect; no browser or headed acceptance ran. The adjacent run-presentation-webgl.py explicitly asserts its final pass verdict.',
     [diagnostics[2]],
     'Contained: finish evidence capture and process cleanup, then make failed or missing scenario verdicts fail the command. Keep successful receipt control and infrastructure failures distinct.'),
    ('Q267', 'One receipt sink writes capture names outside its output directory',
     'ports/graphshell/web/tests/receipt_sink.py:Handler.do_POST',
     'exact current Handler counterexample with in-memory write interception',
     'For an allowed-origin JSON receipt, capture.name is interpolated directly into out/name.png. The exact Handler accepts ../escaped, attempts a write outside --out, then returns 204. An ordinary-name control stays inside; a wrong-origin control returns 403 without writes. The diagnostic intercepts all writes in memory. This is a local receipt-server containment defect, not an internet-exposed service claim. scenario_qualification_server sanitizes names and co_op_gateway rejects invalid names, so those separate implementations have different protections.',
     [diagnostics[3]],
     'Contained: validate a bounded capture name and resolved containment before writes, reject duplicate/colliding names, and preserve receipt evidence. Share a small receipt-path policy without merging application owners.'),
]
for ident, title, source, status, evidence, receipts, repair in rows:
    notes['findings'].append({'id': ident, 'title': title, 'source': source, 'status': status,
        'evidence': evidence, 'verification': NAME, 'diagnostics': receipts,
        'repair_triage': {'disposition': 'documented; production unchanged while shared build remains active',
                         'reason': repair}})
arows = [
    ('Native proof completion', 'Native rendering success and completed community qualification are different facts.',
     ['moot_resources_host::after_wake', 'after_frame', 'run result'], 'Q262: deadline/early-close has no surviving finite completion gate. Color diversity checks reject blank frames but do not independently prove every UI claim.'),
    ('Applet host ownership', 'Per-root WeakSet registration is separate from the Rust pane generation.',
     ['mountCapsuleApplet', 'review_applet', 'applet_close'], 'Q263: stale closed handles retain host mutation powers. Guest-result generation checks cannot identify which JS registration owns review/close.'),
    ('Applet verification duplication', 'Rust pack verification is followed by a separately maintained worker verifier.',
     ['CapsuleMount::review', 'eidetic::pack::verify_pack', 'verify.mjs::canonical/verifyPack', 'worker::init'], 'Worker canonicalization explicitly mirrors Rust serde struct field order. This is duplicate policy/schema maintenance; no differing valid signature verdict was demonstrated. Component and locally trusted generated-binding hashes remain separate.'),
    ('Guest containment', 'Worker grants and Rust lowering are separate checks around one reviewed guest.',
     ['worker::emit', 'Runtime::call/close', 'wasi'], 'Actual runtime controls pass for exact pending completion, close rejection/late results, and deadline termination. Restricted WASI supplies no ambient file/network/storage. These Node fakes do not prove physical worker isolation.'),
    ('Resource browser bridge', 'Explicit local resource-owner HTTP fixture carries requests to existing Rust token gates.',
     ['moot_resource_host.js', 'Host::prepare_dispatch', 'scenario_qualification_server::owner_response'], 'Same-origin configured endpoint, fresh owner read and Rust preparation precede dispatch. Fixture transport is not a signed remote owner, implicit peer, or universal application authority.'),
    ('Viewer and harness mount', 'viewer.js and loader.js reuse the same element setup in mount.js.',
     ['defineElements', 'viewerArtifact', 'loadHostDataset'], 'Real shared mount code exists. Q264 exposes incomplete asynchronous fallback error ownership. ConnectedCallback already checks epochs during primary asynchronous preparation.'),
    ('Scenario error and delivery gates', 'Page error gating and HTTP delivery acknowledgement are separate.',
     ['loader.js::graphshellGateFailures', 'scenario-complete listener'], 'Existing page gate forces failed scenario verdicts; Q265 separately overstates delivery for non-2xx responses. Chronicle report validation checks supplied shape/generation, not reconstructed Djinn/Distillery authority.'),
    ('Receipt collection duplication', 'Three receipt collectors implement their own names, origins, sizes and persistence rules.',
     ['tests/receipt_sink.py', 'scenario_qualification_server.py', 'co_op_gateway.py'], 'Q267: the first lacks capture-name containment although others sanitize or reject. First uses exact configured Origin; qualification server has a body bound; gateway uses its fixture-specific JSON/role/status flow. Do not describe them as one uniform sink policy.'),
    ('Proof driver outcomes', 'Receipt collection, native host completion and command exit status have separate implementations.',
     ['run-headed-receipt.py', 'run-presentation-webgl.py', 'co_op_receipt.py'], 'Q266: headed scenario driver records fail without command failure. Presentation driver asserts pass; two-peer driver checks signed retention/admission/refusal controls. Reading those drivers is not a new headed or two-peer trial.'),
    ('Co-op owner and projection', 'The Woodshed practice proof discloses Rust effective records into a local canvas dataset.',
     ['co_op.js::disclose/render', 'co_op_gateway::Peer', 'co_op_receipt.py'], 'Gateway transports canonical bytes and delegates authority to the Rust peer, with explicit fixture identities/plaintext limits. UI labels are local interpretation; lifecycle and retained/effective counts remain distinct.'),
    ('Delivery budgets', 'Packaging enforces per-route compressed byte caps over its explicitly listed assets.',
     ['build-viewer.py::measure_assets/require_budget', 'test_viewer_budget.py'], 'Four original budget tests pass. The mount row measures viewer.js; imported mount.js is not in that row. This checkpoint makes no complete transitive payload or fresh Wasm/package acceptance claim.'),
    ('Browser evidence plumbing', 'Outer screenshot/accessibility collection and allowlisted external input use separate CDP clients.',
     ['capture-page.mjs', 'host-input.mjs', 'presentation-webgl-fixture.html', 'compare-presentation-captures.py'], 'Read-only captures, explicit input acknowledgement and actual surface tests have scoped roles. Mocked diagnostic callbacks and whole-source reading are not browser input, pixel or device qualification.'),
]
entries = [{'id': f'A{662+i:03d}', 'domain': d, 'claim': c, 'implementations': impl,
            'assessment': a, 'state': 'current complete source reading; diagnostics scoped per entry', 'verification': NAME}
           for i, (d, c, impl, a) in enumerate(arows)]
authority['entries'].extend(entries)
next_read = {'path': 'ports/graphshell/web/Cargo.toml', 'start': 0,
             'reason': 'Finish unread web manifests/configuration, entry pages, styles and all scenarios, then continue all remaining Mere source.'}
notes['current_area'] = 'Community native proof, applet worker/host lifecycle, web mount, co-op fixtures, receipt collectors and drivers'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification': 'progress', 'classification_of_previous_turn': 'progress',
    'goal_status': 'active; full-source objective incomplete', 'coverage': stats,
    'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': [NAME, *diagnostics], 'production_edits_this_turn': False,
    'next_read': next_read, 'pending_verification': None,
    'diagnostic_outcomes': '17 focused diagnostic cases pass; 4 original budget tests pass; original mount script reports all 4 check groups passed. One initial line-ending extraction failure executed zero tests and is preserved.'}
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
with (HERE / 'community_host_diff_check.txt').open('x', encoding='utf-8') as stream:
    stream.write(diff.stdout + diff.stderr)
evidence = ['record_community_host_review.py', 'community_host_diff_check.txt',
    'community_host_lifecycle_probe.mjs', diagnostics[0], 'community_receipt_delivery_probe.mjs',
    'community_receipt_delivery_probe_initial_failure.json', 'community_receipt_delivery_probe_v2.mjs', diagnostics[1],
    'community_headed_exit_probe.py', diagnostics[2], 'community_sink_path_probe.py', diagnostics[3]]
evidence_hashes = {n: sha(HERE / n) for n in evidence}
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
addition = f'''

### Community hosts, applet lifecycle and receipt completion, 2026-10-17

At {head}, this pass fully reads 29 additional files / 178,117 characters: the native community proof, all five web applet modules, resource-owner HTTP bridge, shared mount and receipt loader, co-op page/gateway/driver, both principal receipt collectors, all remaining web test helpers and presentation fixture, viewer budget tooling and entry, and the historical native proof harness. Coverage is {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. Full Mere source review remains incomplete. Prior 1,079 protected source hashes, 24 cited evidence hashes and the complete existing document prefix are preserved.

Q262 is a source-only native proof completion defect: finite timeout/early close can return normally without final-stage qualification; reusing an existing output directory can retain a stale qualification file. This does not invalidate a historical successful run. No fresh native-headed proof ran here.

Q263 has three current-JS counterexamples: a closed handle can review/install, an in-flight review can install after close, and an old handle's repeated close can disturb a live remount and clear its root guard. Q264 reproduces rejected Auto fallback loading leaving its refusal absent while the element remains marked mounted. These use actual unchanged modules with synthetic root, Rust callbacks, fetch, Worker and observer. Existing guest-output generation checks do not establish JS host-registration ownership; receipt pages can still catch the unhandled fallback rejection.

Q265 executes the exact current loader completion listener: HTTP 500 is labelled delivered; 204, transport failure and planted page-gate controls distinguish the boundary. Q266 executes actual unchanged headed-driver main with synthetic process/OS/HTTP/file receipts: failed and successful scenario verdicts both return normally, though metadata retains the correct verdict. Q267 executes the exact current nested receipt Handler with in-memory path/write interception: an allowed-origin ../escaped capture attempts a write outside --out and returns 204; an ordinary name stays contained and a wrong origin refuses. These are adapter/status/containment proofs, not browser, server, device or actual durable receipt acceptance.

There are 17 passing focused diagnostic cases (8 lifecycle/mount, 4 delivery, 2 driver-result, 3 sink-path), four passing original Python delivery-budget tests, and all four original presentation-mount script check groups pass. The first delivery diagnostic assumed LF while source uses CRLF and failed extraction before executing any test; its original source and failure receipt remain, with a distinct successful diagnostic. No original tests or production source changed. Native timeout behavior is source-only; no full Graphshell feature build, browser, GPU, real worker process, network or two-peer runtime was rerun.

A662-A673 track duplicated ownership and genuine sharing separately: Rust and worker pack verification maintain two canonicalization/verifier implementations; host JS lifecycle registration is separate from Rust pane generation; viewer and loader really share mount.js; three receipt collectors and multiple proof drivers maintain divergent naming, origin, size, write and exit-status policies. Resource-owner HTTP transport still delegates tokens/preparation to Rust. Co-op transport still delegates signed authority to its Rust peer; the practice canvas is a disclosed projection. The packaging mount row measures viewer.js and does not measure imported mount.js, so it is not a complete transitive-payload receipt.

Upstream was fetched to {upstream}; checkout/upstream divergence was {ahead_behind.replace(chr(9), ' ahead / ')} behind. The shared dirty checkout and another live Mere build keep integration deferred. This checkpoint stays on its exact source basis; no isolated target, Cargo home or worktree was created, and no audit-owned job remains pending. The next read is the web Cargo manifest and remaining configuration/pages/scenarios, then every other unread source. The [community host checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds the evidence and complete file list. Goal remains active.
'''
doc.write_bytes(prefix + addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': head,
    'prior_receipt': {'path': 'community_owner_review_receipt.json', 'sha256': sha(HERE / 'community_owner_review_receipt.json')},
    'kind': 'Community host/receipt source review; full Mere goal incomplete',
    'coverage': stats, 'whole_current_files': whole, 'whole_current_chars': 178117,
    'source_sha256': {n: sha(ROOT / n) for n in whole}, 'preserved_production_and_test_hashes': guards,
    'new_findings': [r[0] for r in rows], 'authority_entries': [e['id'] for e in entries],
    'diagnostic_cases_passed': 17, 'original_budget_tests_passed': 4,
    'original_mount_script_check_groups_passed': 4,
    'original_test_execution': 'Observed tool command exit0 for unchanged presentation-mount.mjs and test_viewer_budget.py; no browser qualification.',
    'failed_preliminary_attempt': 'community_receipt_delivery_probe_initial_failure.json',
    'production_edits_this_pass': False, 'pending_verification': None, 'next_read': next_read,
    'upstream_fetch': {'origin_main': upstream, 'ahead_behind': ahead_behind, 'merged': False,
                       'reason': 'Concurrent shared Mere build and dirty main'},
    'document': {'path': old['document']['path'], 'sha256': sha(doc), 'preserved_prefix_bytes': len(prefix),
                 'preserved_prefix_sha256': hashlib.sha256(prefix).hexdigest()},
    'evidence_sha256': evidence_hashes, 'retained_isolation': [],
    'receipt_budget': {'cap_bytes': 100_000_000, 'retained_bytes_before_receipt': budget},
    'limits': 'Q262 source-only. Actual JS/Python handlers are exercised with synthetic dependencies as scoped in diagnostic receipts. No new browser, server, network, worker process, GPU, signed community owner or installed-profile acceptance.'}
save(NAME, receipt)
print(json.dumps({'receipt': NAME, 'coverage': stats, 'new_findings': receipt['new_findings'],
                  'protected_paths': len(guards), 'new_whole_files': len(whole),
                  'diagnostic_cases_passed': 17, 'retained_bytes': budget}, indent=2))
