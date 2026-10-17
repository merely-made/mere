"""Save actual Armillary reading and bounded pool evidence."""
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

old = load('seiche_completion_review_receipt.json')
probe = load('armillary_pool_probe_receipt.json')
assert probe['test_exit'] == 0 and probe['tests'] == 2 and probe['generated_output_removed']
assert load('pending_read.json') == []
preserved = {**old['preserved_production_and_test_hashes'], **old['source_sha256']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in probe['source_sha256'].items())
notes, authority = load('review_notes.json'), load('authority_map.json')
findings = [
    {'id': 'Q112', 'title': 'The actor pool can strand queued actors by reusing the same unreserved idle count',
     'status': 'reproduced through unchanged public Pool.submit; documented, not repaired',
     'source': 'crates/armillary/src/pool.rs:Pool::submit and spawn_worker',
     'evidence': 'submit checks idle==0 but does not reserve a waiting worker against the newly queued job. Two submissions can both see idle1, enqueue two jobs and wake only the same worker. The first native bounded attempt reproduced this: workers1, first actor started and blocked, second actor did not start during the150ms observation; releasing the first let the second run. Both were then released and finished. Busy-worker control waits for the first start before submitting the second and observes workers2 with both started before release. This contradicts the module assertion that growth is race-free and a job is never stranded with every worker busy.',
     'scope': 'One reproduction plus one control in a generated native harness compiling the original production pool module by path. Up to32 bounded attempts were allowed; first attempt sufficed. No private scheduling/state edits. Original package tests were filtered out. No hosted product actor workflow was executed.',
     'diagnostic': 'armillary_pool_probe_receipt.json',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'bounded queue/worker accounting correction; qualify concurrency before repair',
        'reason': 'Account for queued reservations when deciding whether available workers can serve admitted long-lived jobs. Preserve a single idle-count transition owner, avoid over-spawn and stranded jobs, and cover multiple submissions against an idle worker. Broader pool shutdown/recovery work should not extend this audit beyond the user one-hour limit.'}},
    {'id': 'Q113', 'title': 'Dropping the last Pool handle leaves its worker threads and state alive indefinitely',
     'status': 'source-confirmed lifecycle defect; no dedicated native drop probe; documented, not repaired',
     'source': 'crates/armillary/src/pool.rs:Pool, Inner, spawn_worker loop',
     'evidence': 'Each spawned worker retains Arc<Inner>, loops forever and waits on Condvar when its queue empties. Pool has no Drop implementation, stop state or shutdown notification, and worker JoinHandles are discarded. Dropping all public Pool clones cannot drop Inner because waiting workers own it; no condition tells those workers to exit. Repeatedly constructing, using and discarding pools therefore retains workers beyond each pool owner lifetime. workers() counts cumulative spawns, not currently live workers after a job panic.',
     'scope': 'Complete source inspection of all six Armillary files. The pool native probe releases every job; its idle worker threads terminate only when the short-lived probe process exits. No production host drop sequence or dedicated leak/panic test was executed. Peak-concurrency bounds apply only to a deliberately retained single pool and do not establish reclamation of discarded pools.',
     'repair_triage': {'disposition': 'documented; active goal forbids production edits',
        'size': 'broader lifecycle policy; may exceed one-hour audit diversion limit',
        'reason': 'Choose explicit owner shutdown/drain behavior, treatment of long-lived jobs and worker panic recovery. Keep public-handle lifetime distinct from worker-owned state; avoid a last-handle test based on raw Arc strong counts that also include workers. Do not block the read-only audit on a lifecycle redesign.'}},
]
for value in findings:
    value['verification'] = 'armillary_review_receipt.json'
upsert(notes['findings'], findings, 'id')
def entry(n, domain, claim, implementations, assessment):
    return {'id': f'A{n:03}', 'domain': domain, 'claim': claim,
        'implementations': implementations, 'assessment': assessment,
        'state': 'Armillary6/6 source read; two generated pool observations; no product edits',
        'verification': 'armillary_review_receipt.json'}
entries = [
    entry(156, 'Pool admission versus worker reservation',
        'Idle count and queue share one lock, so growth is race-free and no queued job is stranded.',
        ['submit reads idle before enqueue without reserving it',
         'Only a woken worker decrements idle after reacquiring the lock',
         'Condvar notifications do not accumulate per queued job',
         'A long-lived actor occupies a worker until it returns'],
        'Q112 proves the same-lock explanation is insufficient. The worker count and queued reservation count are separate facts; a second admission can reuse the first admission idle capacity. A busy-first control grows correctly and isolates the missed reservation.'),
    entry(157, 'Worker lifetime and peak-count claims',
        'Reuse bounds OS threads and leaked per-thread state to peak concurrent jobs.',
        ['Worker loops retain Inner forever', 'No public pool shutdown or last-handle exit exists',
         'Worker JoinHandles are detached', 'workers counter increments on spawn and never decrements',
         'A job panic exits that worker outside any recovery boundary'],
        'Q113 establishes missing reclamation from source. Historical spawn count also diverges from live-worker count after panic, and rapid submissions can spawn before already-created workers reach idle. Peak wording is not an exact live-resource receipt. Shutdown/drain/panic policy remains broader work.'),
    entry(158, 'Actor join and terminal diagnostics',
        'join waits for the actor; a crashed actor reports a finish.',
        ['Dedicated-thread handle stores JoinHandle and ignores its join Result',
         'Pooled handle stores None, so ActorHandle.join only drops its sender and returns',
         'Lifecycle span guard exits on unwind',
         'actor finished event is emitted only after run returns normally'],
        'The single join method does not await pooled execution despite its unqualified wait wording. A dropped span is not the actor finished event and the normal-only lifetime event does not run on panic. These are source-confirmed diagnostic/API qualifications, without native pooled-join or panic execution in this pass. Command success only means enqueue succeeded; Emitter deliberately drops updates on receiver closure.'),
    entry(159, 'Thread pinning versus canonical state authority',
        'KernelThread structurally prevents authority from moving to an actor.',
        ['PhantomData<Rc<()>> makes an embedding value !Send and !Sync',
         'KernelThread::new is available on any current thread',
         'Actor closures are Send and construct potentially !Send internals on their own thread',
         'Generation accepts compares nav/viewport equality, with acceptance performed by the host',
         'RequestIds/Correlated carry identity but define no application success'],
        'The marker proves an auto-trait property of values that embed it; it does not identify a designated host thread or prevent separately shareable state from being cloned into actors. This is a useful structural seam, not a global authority capability. Hosts still own inbox admission and canonical writes. No live host boundary violation was demonstrated.'),
]
upsert(authority['entries'], entries, 'id')
upsert(notes['candidates'], [{
    'title': 'Pooled actor join and panic finish claims exceed their implementation',
    'source': 'crates/armillary/src/actor.rs:ActorHandle::join, spawn_named_on, run_with_lifecycle_span',
    'note': 'Pooled join carries no JoinHandle and returns after dropping its sender. Crash unwinding exits a span but bypasses the actor finished event below run. Dedicated join discards the panic result. Qualify terminal state versus channel closure and normal finish before describing successful completion; no native pooled-join/panic trace diagnostic was run here.'
}], 'title')
inventory, coverage = load('source_inventory.json')['files'], load('read_coverage.json')
complete = sorted(p for p, v in inventory.items() if p.startswith('crates/armillary/')
    and coverage.get(p, {}).get('sha256') == v['sha256'] and coverage[p]['through'] == v['chars'])
assert len(complete) == 6 and sum(inventory[p]['chars'] for p in complete) == 31662
assert all(sha(ROOT / p) == inventory[p]['sha256'] for p in complete)
stats = json.loads(subprocess.check_output(['python', str(HERE / 'read_coverage.py'), 'status'], cwd=ROOT))
assert stats['complete_files'] == 852 and stats['read_chars'] == 12350851
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path = next(p for p, v in sorted(inventory.items()) if p.startswith('crates/conatus/numen/')
    and coverage.get(p, {}).get('through', 0) < v['chars'])
next_read = {'path': next_path, 'start': coverage.get(next_path, {}).get('through', 0)}
notes['current_area'] = 'Seiche91/91 and Armillary6/6 fully read; physics runtime and generic actor ownership qualified'
notes['next_area'] = 'crates/conatus/numen'
notes['checkpoint'] = {'classification': 'progress', 'goal_status': 'active; full-source objective incomplete',
    'coverage': stats, 'finding_count': len(notes['findings']), 'authority_entries': len(authority['entries']),
    'new_receipts': ['seiche_completion_review_receipt.json', 'armillary_review_receipt.json'],
    'next_read': next_read, 'production_edits_this_turn': False,
    'diagnostic_outcomes': 'Five native tests: tide inline+actor, empty Density inline, pool queue starvation and busy control. Q110-Q112 native; Q113 source-confirmed.',
    'classification_of_previous_turn': 'progress', 'coverage_checkpoint': stats,
    'user_instruction': 'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note': 'This goal continuation adds57 full files and618481 chars. Full Mere review remains active.'}
upsert(notes['source_qualifications'], [{
    'area': 'Armillary complete source review and public pool observation',
    'note': 'All six Armillary files read,31662 chars. Two generated native tests compile the unchanged production pool module by path; source package tests are filtered out. Queue starvation reproduced with busy-worker control. Source identifies missing pool shutdown and pooled-join/panic diagnostic scope. All submitted diagnostic jobs finish; detached idle pool workers end with the diagnostic process. Generated executable/PDB removed; no isolated build path.'
}], 'area')
save('review_notes.json', notes)
save('authority_map.json', authority)

doc = old['document']['path']
destination = ROOT / doc
before = destination.read_bytes()
assert sha(destination) == old['document']['sha256']
addition = '''

### Read-only Armillary actor and pool review, 2026-10-16

Armillary is fully read: six files,31,662 characters. Mere coverage is now
852/3352 files and12,350,851/36,854,820 characters. Together with Seiche's
completion this continuation adds57 full files and618,481 characters. The
full-source goal remains active; resume at crates/conatus/numen. Production
and concurrent dirty work are preserved.

Q112: Pool.submit checks idle workers without reserving one for its queued job.
Multiple admissions can see the same idle1 before that worker reacquires the
lock, enqueue two actors and notify only the same worker. The first actor then
blocks indefinitely in normal long-lived work while the other remains queued.
This contradicts the module's never-stranded/race-free claim. A bounded native
probe of the unchanged production module reproduced on its first attempt:
workers1, first actor started, second absent for150ms, then second starts when
first is released. A busy-first control grows to two workers and starts both
before either is released. Every submitted job was released and finished.
Repair admission/reservation accounting with one owner of idle transitions;
qualify overspawn and concurrent submission before calling it fixed. This is
a contained concurrency defect, separate from redesigning pool lifecycle.

Q113: source inspection confirms workers are never reclaimed when the final
public Pool handle drops. Detached worker loops retain Arc<Inner> and wait on
a Condvar forever; no stop state, Drop path or notification allows exit.
Repeated creation/use/discard of pools therefore retains resources beyond
each owner's lifetime. No dedicated native leak or host teardown test ran.
The probe process exits after completing its jobs, which ends its idle worker
threads. Public workers() is cumulative spawns, not a live count after panic.
Peak-concurrency language needs that qualification. Shutdown, draining,
long-lived actor cancellation and panic recovery are broader policy work;
document them without extending this audit beyond the user's one-hour limit.

Two other source claims need qualification. Pooled ActorHandle carries no join
handle, so join returns after dropping its command sender rather than waiting
for its actor. A panic exits the lifecycle span but bypasses the actor finished
event below run; span closure is not a normal terminal-success event. Dedicated
join discards the panic Result. These were source-reviewed, not natively probed.
Command true means channel enqueue succeeded; it does not certify application.
Emitter's silent update drop after receiver closure is explicitly documented.

KernelThread correctly makes embedding values !Send/!Sync. It does not designate
one globally authoritative thread or police separately shared state. Hosts own
canonical writes, generation acceptance and request outcome interpretation.
Correlated IDs intentionally do not define applied/rejected states. Preserve
those bounded contracts when removing overbroad boundary-drift claims.

Two generated pool observations passed; the module's own two tests were filtered
out. The executable/PDB was removed. No package, product-host, shutdown or panic
suite ran. No isolated target, Cargo home or worktree was created. Preservation
hashes and exact diagnostic limits are in the
[Armillary review receipt](../../../support/receipts/authority-success-audit/armillary_review_receipt.json).
'''
assert b'### Read-only Armillary actor and pool review, 2026-10-16' not in before
destination.write_bytes(before + addition.encode())
diff = subprocess.run(['git', 'diff', '--check'], cwd=ROOT, capture_output=True, text=True)
(HERE / 'armillary_review_diff_check.txt').write_text(diff.stdout + diff.stderr, encoding='utf-8')
assert diff.returncode == 0
evidence = ['armillary_pool_probe.rs', 'run_armillary_pool_probe.py',
    'armillary_pool_probe_build.log', 'armillary_pool_probe_output.log',
    'armillary_pool_probe_receipt.json', 'armillary_review_diff_check.txt']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'kind': 'Complete Armillary source review; full Mere source objective incomplete',
    'production_edits_this_pass': False, 'new_findings': [x['id'] for x in findings],
    'authority_entries': [x['id'] for x in entries], 'coverage': stats,
    'complete_armillary_files': complete, 'source_sha256': {p: sha(ROOT / p) for p in complete},
    'preserved_production_and_test_hashes': preserved, 'next_read': next_read,
    'diagnostic_tests': 2, 'diagnostic_exit': 0,
    'package_tests': 'source read only; not run', 'generated_output_removed': True,
    'retained_isolation': [],
    'document': {'path': doc, 'sha256': sha(destination), 'preserved_prefix_bytes': len(before),
        'preserved_prefix_sha256': hashlib.sha256(before).hexdigest()},
    'diff_check': 0, 'evidence_sha256': {p: sha(HERE / p) for p in evidence},
    'limits': ['Full Mere source coverage incomplete; no production fixes.',
        'Two generated native tests are not a full package or host suite.',
        'Pool drop/shutdown, pooled join and panic claims were source reviewed only.']}
assert all(sha(ROOT / p) == h for p, h in preserved.items())
assert all(sha(ROOT / p) == h for p, h in receipt['source_sha256'].items())
save('armillary_review_receipt.json', receipt)
print(json.dumps({'coverage': stats, 'armillary_complete': len(complete), 'findings': len(notes['findings']),
    'authority_entries': len(authority['entries']), 'next_read': next_read}))
