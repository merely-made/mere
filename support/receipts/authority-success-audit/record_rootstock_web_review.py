"""Checkpoint completed Rootstock/browser source reads and qualified diagnostics."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAME = 'rootstock_web_review_receipt.json'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def save(n, value):
    (HERE/n).write_text(json.dumps(value, indent=2)+'\n', encoding='utf-8')
def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()
assert not (HERE/NAME).exists()
old = load('rootstock_layout_review_receipt.json')
head = git('rev-parse', 'HEAD')
assert head == old['basis_commit']
guards = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert len(guards) == 1608 and all(sha(ROOT/n) == h for n,h in guards.items())
assert all(sha(HERE/n) == h for n,h in old['evidence_sha256'].items())
doc = ROOT/old['document']['path']
prefix = doc.read_bytes()
assert sha(doc) == old['document']['sha256']
shared = old['shared_working_file_sha256']
assert all(sha(ROOT/n) == h for n,h in shared.items() if n != old['document']['path'])
index = Path(git('rev-parse', '--git-path', 'index'))
index = index if index.is_absolute() else ROOT/index
index_hash = sha(index)
assert index_hash == old['git_index_sha256']
inv, cov = load('source_inventory.json'), load('read_coverage.json')
assert inv['commit'] == head and load('pending_read.json') == []
prefixes = ['crates/cambium/cambium-rootstock/', 'crates/cambium/cambium-genet-web-host/']
areas = []
for area, count, chars in zip(prefixes, [55,30], [714015,273208]):
    files = [n for n in inv['files'] if n.startswith(area)]
    assert len(files) == count and sum(inv['files'][n]['chars'] for n in files) == chars
    assert all(cov.get(n,{}).get('sha256') == inv['files'][n]['sha256'] and cov[n]['through'] == inv['files'][n]['chars'] for n in files)
    areas.append({'prefix':area,'files':count,'chars':chars,'verification':NAME,'basis_commit':head,'qualification':'Every inventoried file read at current hashes; execution scope is limited to the separately named diagnostics.'})
whole = sorted(n for n in inv['files'] if n.startswith(tuple(prefixes)) and n not in guards)
assert len(whole) == 45 and sum(inv['files'][n]['chars'] for n in whole) == 437815
assert all(sha(ROOT/n) == inv['files'][n]['sha256'] for n in whole)
stats = json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files'] == 2125 and stats['read_chars'] == 25487790
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
suite, failed, lifetime = [load(n) for n in ['rootstock_receipt_suite_v3_probe_receipt.json','rootstock_receipt_suite_v2_probe_receipt.json','browser_lifetime_probe_receipt.json']]
first = load('rootstock_receipt_suite_failure_receipt.json')
assert first['build_exits'] == [0,0,1] and first['tests_run'] == 0 and first['generated_output_removed']
assert all(sha(HERE/n) == h for n,h in first['evidence_sha256'].items())
assert failed['build_exits'] == [0,0,1] and failed['run_exit'] is None and failed['checks'] == 0
assert 'E0463' in (HERE/'rootstock_receipt_suite_v2_probe_compile_2.log').read_text()
assert suite['build_exits'] == [0,0,0] and suite['run_exit'] == 0 and suite['checks'] == 15
assert lifetime['build_exits'] == [0] and lifetime['run_exit'] == 0 and lifetime['checks'] == 5
for probe in [suite, failed, lifetime]:
    assert probe['generated_output_removed'] and all(not Path(o['path']).exists() for o in probe['removed_outputs'])
    assert all(sha(ROOT/n) == h for n,h in probe['source_sha256'].items())
    assert all(sha(Path(v['path'])) == v['sha256'] for v in probe.get('libraries',{}).values())
notes, authority = load('review_notes.json'), load('authority_map.json')
assert notes['findings'][-1]['id'] == 'Q308' and authority['entries'][-1]['id'] == 'A789'
findings = [
 {'id':'Q309','title':'Browser key conversion calls Unicode scalars graphemes and discards multi-scalar text keys',
  'source':'crates/cambium/cambium-genet-web-host/src/input.rs:59,74; crates/cambium/cambium-genet-web-host/src/events.rs:43',
  'status':'source-verified conversion counterexample; browser/OS key delivery not executed',
  'evidence':'The exported key_press_from_dom contract and match comment promise exactly one grapheme, but the admission test is other.chars().count() == 1. A key string a followed by combining acute has two Unicode scalar values and is consequently Key::Named(Other) with text None; the same is true of a multi-scalar emoji grapheme. Dead and Unidentified are explicitly special-cased, so neither fallback restores that text. The installed canvas and mirror keydown listeners call this exact converter. This establishes loss for such an input value at the API boundary; it does not claim an observed physical keyboard, browser layout or IME reproducer.',
  'repair_triage':{'disposition':'documented under active read-only audit goal','reason':'Use the shared text/grapheme policy for deciding text keys, preserving named/dead-key handling. Verify real browser key and composition routes separately rather than treating scalar count as grapheme segmentation.'}},
 {'id':'Q310','title':'Public browser file chooser permanently forgets callbacks that retain its removed input',
  'source':'crates/cambium/cambium-genet-web-host/src/files.rs:35,119,140,164',
  'status':'source-verified ownership leak in exported non-lifetime constructor; no browser heap measurement',
  'evidence':'WebFileChooser::new delegates to new_inner with no MountLifetime. That branch installs change/cancel listeners and calls Closure::forget on both, with no stored callback or removal token. The change closure captures a clone of input and pending; Drop only takes pending and removes the element. Removing the DOM element cannot drop the deliberately forgotten Rust closure, so each such construction leaves callback/input state allocated for the remaining module lifetime. The normal mount.rs call uses new_with_lifetime and registers removable callbacks; repository search found no call to the public new path. The separate owned implementation thus has materially different cleanup semantics despite returning the same RAII-looking chooser.',
  'repair_triage':{'disposition':'documented under active read-only audit goal','reason':'Store and remove standalone callbacks on chooser drop, or make this constructor explicitly owned by a supplied lifetime. Do not change group-owned device or mount disposal authority.'}}
]
for finding in findings: finding['verification'] = NAME
notes['findings'].extend(findings)
rows = [
 ('Producer resource identities are translated at one registry boundary','Producer-local keys and physical renderer keys have distinct owners.', ['crates/cambium/cambium-rootstock/src/producer/registry.rs','crates/cambium/cambium-rootstock/src/producer/shadow_namespace_tests.rs'], 'Physical keys use a process-wide nonrecycled atomic allocator below the translator shadow band. GPU remapping validates the complete plan before rewriting producer/image/pattern/mask references. Source tests distinguish actual translator command properties from GPU pixel acceptance; those GPU tests were read, not run.'),
 ('Producer validation still has explicit open seams','Output refusal, view metadata and whole-frame refusal must be distinguished.', ['crates/cambium/cambium-rootstock/src/producer/registry.rs'], 'validate_output inspects the underlying texture, not visibly the exact view subresource range. Invalid producer output unregisters its image and continues; commands can be Some(empty). Keep these as policy/consumer and exact-view follow-ups, not a proven silent-success or GPU validation bug.'),
 ('SceneProducer retains caller authority but cannot pass export purpose','The adapter owns GPU output while caller Rc state, callbacks and revision remain source authority.', ['crates/cambium/cambium-rootstock/src/scene_producer.rs'], 'Changing source Rc invalidates output despite equal revisions, and checked rendering preserves attributed failure. portable_scene ignores ProducerFrameInfo::purpose and invokes the same mutable scene callback. Trace supported mutable consumers before claiming export advances presentation time. The legacy render Option erases refusal, while current registry uses checked rendering.'),
 ('Rootstock adapters delegate actual host state','WindowDom, HostTree, wake and command wrappers are bounded projections.', ['crates/cambium/cambium-rootstock/src/window_dom.rs','crates/cambium/cambium-rootstock/src/tree.rs','crates/cambium/cambium-rootstock/src/wake.rs','crates/cambium/cambium-rootstock/src/window_verbs.rs','crates/cambium/cambium-rootstock/src/spatial.rs'], 'WindowDom cuts traversal at the mount and preserves shadow traversal within it, but is not a foreign-node security sandbox. HostTree is sealed unit-action delegation; wake coalesces an atomic signal. Commands are caller/event-source performed. Spatial prose says key repeat while actual input arms on initial Tab down.'),
 ('Successful headless suites have explicit presentation limits','Current Host capture, hover and prepared-frame tests use genuine state with a fake presentation driver.', ['crates/cambium/cambium-rootstock/tests/capture_owner.rs','crates/cambium/cambium-rootstock/tests/hover_motion.rs','crates/cambium/cambium-rootstock/tests/prepared_frame.rs'], 'Fifteen original current tests pass over freshly compiled Cambium/Rootstock and retained exact supplier libraries. This proves the selected capture lifetime, hover reentrancy and frame stamp/refusal cases, not GPU pixels, native surfaces, optional embedding or a fresh current supplier graph. Two initial harness failures are retained with no test-pass credit.'),
 ('Browser-only code is accurately excluded from native evidence','lib.rs explicitly warns that native workspace checks omit the browser modules.', ['crates/cambium/cambium-genet-web-host/src/lib.rs','crates/cambium/cambium-genet-web-host/Cargo.toml'], 'This is an honest qualification, not a false success claim. Native mirror/lifetime logic and optional portable lowering are distinct from wasm DOM/event/backend code. All thirty files were source read, but no wasm Cargo check or browser runtime was run in this pass.'),
 ('Browser input conversion contains independent policy','The event source claims only conversion but chooses scalar admission, wheel normalization and composition mapping.', ['crates/cambium/cambium-genet-web-host/src/input.rs','crates/cambium/cambium-genet-web-host/src/events.rs','crates/cambium/cambium-genet-web-host/src/surface.rs'], 'Q309 records the scalar/grapheme mismatch. Keydown default suppression is broad; no composition-aware suppression is visible. IME enable/cursor methods accurately document their no-op status and missing editable host. Browser/OS composition behavior remains unexecuted.'),
 ('Browser accessibility shares projection but maintains its own mirror','Neutral projection is lowered to ARIA and applied incrementally; author and producer naming have distinct precedence.', ['crates/cambium/cambium-genet-web-host/src/mirror.rs','crates/cambium/cambium-genet-web-host/src/mirror/tests.rs','crates/cambium/cambium-genet-web-host/src/a11y.rs'], 'Producer semantics name their slot; ordinary leaf semantics yield to an author name. Drawn action IDs are stable hashes in a reserved domain and map to current produced targets. Sync routes real host focus without stealing external page focus. Pure source tests are not live DOM or screen-reader proof. Canvas aria-hidden restoration on disposal and transformed positioning remain follow-ups.'),
 ('File chooser cleanup has two implementations','Lifetime-owned mounting and the exported standalone constructor return the same chooser with different callback ownership.', ['crates/cambium/cambium-genet-web-host/src/files.rs','crates/cambium/cambium-genet-web-host/src/mount.rs'], 'Q310 identifies the forgotten standalone closures. Owned listeners live until mount disposal, even if a chooser is replaced earlier. Async file reads filter individual failures into omission and completion is phase admitted; do not misreport this as filesystem or error-bearing acquisition.'),
 ('Browser mount completion is not a first-pixel receipt','Mount starts listeners and the frame loop after a void first draw.', ['crates/cambium/cambium-genet-web-host/src/mount.rs'], 'Startup returns a mounted lifetime after draw even though redraw failure is stored in host state rather than propagated. Hidden/zero-size startup intentionally defers presentation. The claim that a first frame exists before events is broader than the signature proves; do not equate mount Ok with a successful PresentedFrame.'),
 ('Backend selection precedes app initialization','Fallback replaces a context-bound canvas and initializes application state once after selection.', ['crates/cambium/cambium-genet-web-host/src/presentation/boot.rs','crates/cambium/cambium-genet-web-host/examples/presentation_page/index.html'], 'Forced choices attribute failure; Auto can fall back after a fresh canvas clone that preserves attributes/focus. The fixture asserts route, stamps, capture, independence and disposal, but was only read. It is not a fresh browser acceptance receipt.'),
 ('Portable lowering composes complete producer sources','The browser adapter remaps local images and delegates flattening/admission to SparseFragmentRegistry.', ['crates/cambium/cambium-genet-web-host/src/portable.rs','crates/cambium/cambium-genet-web-host/tests/portable_presentation.rs'], 'Stable source/key order prevents ledger ordering from changing identities; source clipping, opacity, placement and device scale are composed. Generated shadow masks are explicitly refused. CPU pixel and refusal tests were read, not run; the resolver is supplied by retained Netrender code outside this pass.'),
 ('WebGL presentation owns a context and turn-qualified capture','Generation/lifetime checks precede checked recording, then current-turn readback copies into a bounded PBO.', ['crates/cambium/cambium-genet-web-host/src/presentation/webgl.rs','crates/cambium/cambium-genet-web-host/src/presentation/readback.rs','crates/cambium/cambium-genet-web-host/src/capture.rs','crates/cambium/cambium-genet-web-host/src/lane.rs'], 'Pack-state restoration, fence polling, row flip and loss cancellation are explicit. A capture after the presentation turn is refused. Raw readback cancellation and disposed capture-owner attribution remain different layers; these browser/GL paths were not run.'),
 ('Mount disposal and group recovery have separate authority','Mount lifetime revokes callbacks immediately, then releases owned resources; shared replacement belongs to the recovery group.', ['crates/cambium/cambium-genet-web-host/src/lifetime.rs','crates/cambium/cambium-genet-web-host/src/recovery.rs','crates/cambium/cambium-rootstock/src/recovery.rs'], 'Five original lifetime tests pass directly with the standard library, with no supplier artifacts. This is native phase/future/cleanup evidence, not browser microtask/listener proof. Recovery detaches/acknowledges hidden hosts independently of RAF, claims a single finite replacement task and retains CPU source state.'),
 ('Large recovery fixture limits its own success claims','Two full Workshop mounts compare retained authority and complete source/presented captures around controlled owned device destruction.', ['crates/cambium/cambium-genet-web-host/examples/shared_recovery/main.rs','crates/cambium/cambium-genet-web-host/examples/shared_recovery/index.html','crates/cambium/cambium-genet-web-host/src/start.rs'], 'The fixture checks byte/deadline bounds, physical key separation, same replacement Arc, source/runner identity, stamped pixel comparisons and explicit group closure. It labels synthetic DOM controls, logical Host input, async quality, physical reset and OS accessibility limits. Source reading is not execution. Constructor helper uses an atomic once guard; its Node fixture is also source-only here.')
]
entries = [{'id':f'A{790+i}','domain':d,'claim':claim,'implementations':paths,'assessment':assessment,'state':'current source read with separately qualified diagnostics','verification':NAME} for i,(d,claim,paths,assessment) in enumerate(rows)]
authority['entries'].extend(entries)
notes['completed_source_areas'].extend(areas)
notes['review_followups'].extend([
 {'verification':NAME,'area':'Producer admission and texture views','detail':'Trace invalid-output empty commands and exact texture view range/format validation before promoting a whole-frame or GPU correctness finding.'},
 {'verification':NAME,'area':'SceneProducer export purpose','detail':'Trace mutable supported scene callbacks because the adapter ignores purpose; do not select failed-frame or export clock policy without the owner contract.'},
 {'verification':NAME,'area':'Browser focus, IME and startup claims','detail':'Check live DOM focus/aria-hidden restoration, transformed mirror placement, composition handling and mounted-versus-presented status. Initial draw returns void; mount success alone does not prove pixels.'}
])
next_read = {'path':'crates/cambium/cambium-genet-winit-host/Cargo.toml','start':0,'reason':'Read the native host and all remaining Mere source; full audit remains active.'}
notes['current_area'] = 'Rootstock and browser host fully source read at current hashes'
notes['next_area'] = next_read['reason']
notes['checkpoint'] = {'classification':'progress','classification_of_previous_turn':'progress','goal_status':'active; full source incomplete','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),'new_receipts':[NAME,'rootstock_receipt_suite_v3_probe_receipt.json','rootstock_receipt_suite_v2_probe_receipt.json','rootstock_receipt_suite_failure_receipt.json','browser_lifetime_probe_receipt.json'],'audit_authored_production_edits_this_turn':False,'next_read':next_read,'pending_verification':None,'diagnostic_outcomes':'15 original headless Host integration tests and 5 original native lifetime tests pass. Initial filename and E0463 harness failures retained. Browser/GPU/optional feature paths not executed.'}
diff = subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
assert diff.returncode == 0
with (HERE/'rootstock_web_diff_check.txt').open('x',encoding='utf-8') as f: f.write(diff.stdout+diff.stderr)
evidence = ['record_rootstock_web_review.py','rootstock_web_diff_check.txt','rootstock_receipt_suite_failure_receipt.json',*first['evidence_sha256']]
for stem in ['rootstock_receipt_suite_v2_probe','rootstock_receipt_suite_v3_probe']:
    evidence += [stem+'.rs','run_'+stem+'.py',stem+'_receipt.json',*(stem+f'_compile_{i}.log' for i in range(3))]
evidence += ['rootstock_receipt_suite_v3_probe_output.log','run_browser_lifetime_probe.py','browser_lifetime_probe_receipt.json','browser_lifetime_probe_compile.log','browser_lifetime_probe_output.log']
addition = f'''

### Rootstock completed; browser host authority and cleanup, 2026-10-17

The complete current Rootstock source area is read (55 files / 714,015 characters), as is the browser host (30 files / 273,208 characters). Since the preceding checkpoint, reading coverage advanced by 46 files / 449,720 characters to {stats['complete_files']:,}/{stats['total_files']:,} files and {stats['read_chars']:,}/{stats['total_chars']:,} characters. One of these files already had a protected source hash from an earlier narrow finding; 45 newly guarded complete files total 437,815 characters. All current inventory hashes match. HEAD remains {head}; a fresh origin/main fetch found no newer commit, and HEAD includes origin/main (ahead 10, behind 0). Full-source review remains incomplete.

Q309 records browser key conversion's scalar/grapheme contradiction: a multi-scalar grapheme input value becomes Named(Other) with no text, despite the contract promising one grapheme. This is a source-proven API conversion loss, not an observed OS/browser/IME event. Q310 records the exported standalone file chooser's forgotten closures retaining input/callback state after its Drop removes the DOM element. The actual mount uses the lifetime-owned path; no in-tree standalone constructor consumer was found. Both are documented without production changes.

The [checkpoint](../../../support/receipts/authority-success-audit/{NAME}) binds 15 passing original current capture/hover/prepared-frame tests. Cambium and Rootstock were compiled fresh over exact retained supplier artifacts; their fake-driver receipts do not establish GPU/native presentation or a whole-current Cargo graph. Five original browser-lifetime tests also pass directly with the standard library. These establish native phase/startup/future/cleanup behavior, not actual browser microtask/listener removal. First harness failure requested the wrong filename; the second failed E0463 dependency resolution. Both failed attempts are retained with zero test credit; a distinct v3 ran all 15 successfully. Generated rlibs/executables/PDBs were recorded and removed.

A790-A804 distinguish producer-local/physical keys and validated remapping; texture-view and whole-frame admission questions; SceneProducer callback/source/purpose authority; bounded DOM/wake/command adapters; test receipt limits; native versus wasm compilation; duplicated input/file ownership policy; shared accessibility projection versus DOM mirror state; mounted versus presented status; backend fallback; portable materialization; GL turn-qualified readback; shared recovery; and the finite Workshop fixture's precise success conditions. The browser crate accurately warns that native checks omit most code. Its mount returns after a void draw, so mount success is not a PresentedFrame receipt. IME methods explicitly remain no-ops without an editable host. Browser, screen-reader, GPU and full optional-feature acceptance remain unexecuted.

Producer validation of underlying textures versus exact views, invalid-output empty commands, export-purpose callbacks, focus/aria-hidden cleanup and transformed mirror positioning remain named follow-ups rather than speculative proven bugs. Shared production/WIP hashes and index are unchanged; prior audit text is an exact prefix. No isolated target, Cargo home or worktree was introduced. Continue with the native host and every remaining unread Mere source file.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
save('review_notes.json', notes)
save('authority_map.json', authority)
budget = sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file())
assert budget < 100_000_000
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'kind':'Completed Rootstock and browser source review; full repository incomplete','prior_receipt':{'path':'rootstock_layout_review_receipt.json','sha256':sha(HERE/'rootstock_layout_review_receipt.json')},'coverage':stats,'completed_source_areas':areas,'whole_current_files':whole,'source_sha256':{n:sha(ROOT/n) for n in whole},'preserved_production_and_test_hashes':guards,'shared_working_file_sha256':shared,'new_findings':['Q309','Q310'],'authority_entries':[e['id'] for e in entries],'audit_authored_production_edits_this_pass':False,'diagnostics':{'rootstock_receipt_suite_v3_probe_receipt.json':{'build_exits':[0,0,0],'run_exit':0,'checks':15},'rootstock_receipt_suite_v2_probe_receipt.json':{'build_exits':[0,0,1],'run_exit':None,'checks':0,'qualification':'E0463 harness dependency loading failure'},'rootstock_receipt_suite_failure_receipt.json':{'build_exits':[0,0,1],'run_exit':None,'tests_run':0,'qualification':'harness filename mismatch'},'browser_lifetime_probe_receipt.json':{'build_exits':[0],'run_exit':0,'checks':5}},'pending_verification':None,'next_read':next_read,'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},'evidence_sha256':{n:sha(HERE/n) for n in evidence},'git_index_sha256':index_hash,'retained_isolation':[],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},'limits':'Source conversion/ownership findings, not browser/OS reproduction. Fifteen current Host tests over retained exact suppliers and five std-only lifetime tests. No GPU/native surface/wasm/optional feature/full-current supplier graph acceptance.'}
save(NAME,receipt)
assert sha(index) == index_hash and doc.read_bytes().startswith(prefix)
assert all(sha(ROOT/n) == h for n,h in {**guards,**receipt['source_sha256']}.items())
assert all(sha(HERE/n) == h for n,h in receipt['evidence_sha256'].items())
assert all(sha(ROOT/n) == h for n,h in shared.items() if n != old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'new_findings':receipt['new_findings'],'authority_entries':receipt['authority_entries'],'protected_sources':len({**guards,**receipt['source_sha256']}),'retained_bytes':budget},indent=2))
