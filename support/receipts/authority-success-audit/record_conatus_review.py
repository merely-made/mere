"""Record complete Conatus reading and current CPU evidence without product edits."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
def save(n,value):
    (HERE/n).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
def upsert(items,values,key):
    for value in values:
        at=next((i for i,x in enumerate(items) if x.get(key)==value[key]),None)
        if at is None: items.append(value)
        else: items[at]=value

old=load('nisus_tenant_review_receipt.json')
probe=load('conatus_cpu_probe_receipt.json')
tests=load('conatus_existing_cpu_tests_receipt.json')
assert probe['test_exit']==0 and probe['tests']==5 and probe['generated_output_removed']
assert all(r['test_exit']==0 and r['generated_output_removed'] for r in tests['results'])
assert [r['passed'] for r in tests['results']]==[19,25]
assert load('pending_read.json')==[]
preserved={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in tests['library_sha256'].values())
assert all(sha(HERE/p)==h for p,h in tests['evidence_sha256'].items())
notes,authority=load('review_notes.json'),load('authority_map.json')
findings=[
    {'id':'Q117','title':'Engine system failure consumes unexecuted time and discards applied-change publication',
     'status':'reproduced through public CPU Engine; documented, not repaired',
     'source':'crates/conatus/conatus/src/engine.rs:advance, run_steps, run_phase; clock.rs:FixedClock::advance; world.rs:step and drains',
     'evidence':'An injected one-time AfterPhysics system error during advance50000us grants three60Hz steps up front. clock.steps_taken becomes3 but BodyWorld.tick is1; subsequent advance0 reports steps0/deferred0 and never executes the other two. A separate run applies a queued spawn and has a preexisting direct despawn, then errors after physics. The spawn exists in live state, but command result, changed-body and prior removal notification are absent from the error and the next successful zero-step frame. No-error controls execute all three steps and publish the spawn result/change/removal. run_steps drains notifications into locals and returns Err without a partial update or restoring them.',
     'scope':'Two generated public CPU observations with a deterministic single failing system and no-error controls. No rollback is presumed: the demonstrated defect is lost admitted time and unobservable already-applied outcomes. A full state snapshot can rebuild current bodies, but cannot reconstruct discarded command results or interaction history. Resource/system side effects and product recovery were not exercised.',
     'diagnostic':'conatus_cpu_probe_receipt.json',
     'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'broader scheduler failure/publication policy; may exceed one-hour audit diversion',
        'reason':'Keep unexecuted granted steps pending and preserve or return partial publication for already-applied work. Choose exact phase retry/resume, command outcome and resource side-effect semantics before repair; do not invent transactional rollback. This larger design must not stall all-source reading.'}},
    {'id':'Q118','title':'Immediate impulses change BodyState velocity without marking zero-step publication dirty',
     'status':'reproduced for linear and angular impulses through public CPU Engine; documented, not repaired',
     'source':'crates/conatus/conatus/src/world.rs:apply_impulse, apply_torque_impulse, drain_changes; engine.rs:zero-step publication',
     'evidence':'After an initial body step and drained publication, apply_impulse changes actual linear speed to0.47746482 and apply_torque_impulse changes angular speed to1.193662. advance0 returns changed[] in both cases. Calling set_velocity with the identical already-current velocity makes the following zero-step frame publish that state. Impulse methods increment world revision but omit dirty_bodies insertion, while set_velocity marks it.',
     'scope':'One generated native test covers both public impulse methods, a finite dynamic sphere, zero gravity and identical-state set_velocity controls. No renderer, network or save consumer was executed. A later moving physics step can publish the transform; that does not publish the immediate state on a zero-step frame.',
     'diagnostic':'conatus_cpu_probe_receipt.json',
     'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'contained dirty-state publication repair',
        'reason':'Mark successful immediate impulse changes for BodyState publication consistently with set_velocity. Qualify no-op, sleeping and fixed-body refusal behavior without turning force accumulation into an unrelated durable event stream.'}},
    {'id':'Q119','title':'Finite component admission permits overflowing norms to corrupt rotations and ray answers',
     'status':'reproduced through public CPU body and query APIs; documented, not repaired',
     'source':'crates/conatus/conatus/src/world.rs:validate_transform, squared_length, raycast; world/rapier.rs:pose and ray direction normalization',
     'evidence':'A fixed body with finite quaternion[f32::MAX,0,0,1] is accepted, then public state returns rotation[0,0,0,0], norm squared0, because squared norm overflowed before normalization. Moderate nonunit[2,0,0,0] correctly normalizes. A finite ray from(0,4,0) toward a present unit box hits at3 units for directions(0,-1,0) and(0,-10,0), but direction(0,-f32::MAX,0) is accepted as Ok(None). Finite component checks and the nonzero threshold do not reject an infinite derived norm; division turns the latter ray direction into zero.',
     'scope':'Two generated native tests using freshly built default Conatus and actual Rapier-backed public APIs. Finite inputs and moderate controls only; no renderer, character movement, GPU or deliberate panic execution. Returned zero rotation is an admitted unusable state, not a demonstrated downstream crash.',
     'diagnostic':'conatus_cpu_probe_receipt.json',
     'repair_triage':{'disposition':'documented; active goal forbids production edits',
        'size':'contained norm/admission repair with shared-helper review',
        'reason':'Compute stable scaled norms or reject unusable derived norms before mutation/query execution; preserve documented normalization of reasonable nonunit inputs. Character-up and Tenant normalization share the source-qualified numeric pattern but were not independently executed here.'}},
]
for value in findings: value['verification']='conatus_review_receipt.json'
upsert(notes['findings'],findings,'id')

def entry(n,domain,claim,implementations,assessment):
    return {'id':f'A{n:03}','domain':domain,'claim':claim,'implementations':implementations,
        'assessment':assessment,'state':'Conatus23/23 source read;44 existing CPU tests and5 diagnostic observations; resident source only',
        'verification':'conatus_review_receipt.json'}
entries=[
    entry(176,'Clock grants versus accepted physical steps','The fixed clock defers excess work instead of dropping time.',
        ['FixedClock uses integer microseconds times rate and retains capped backlog',
         'Engine.advance consumes its grant before running systems',
         'Exact Engine.step deliberately does not change the host clock',
         'A system error aborts run_steps without returning unused grant'],
        'The clock primitive itself has meaningful drift/cap tests, freshly passed. Q117 proves Engine failure breaks the end-to-end retention claim: granted-step count and actual world tick become different authorities. Explicit exact stepping is documented, not an independent clock defect.'),
    entry(177,'Applied command results versus failed frame reporting','A failed command does not prevent unrelated systems advancing; frame results identify outcomes.',
        ['CommandQueue preserves each ordinary command Result',
         'System errors instead return EngineError immediately',
         'Frame aggregation drains world events/removals/voxel changes into temporary locals',
         'BodyWorld.step also drains dirty state into a StepUpdate'],
        'Q117 distinguishes honest per-command errors from the system-error publication hole. Already-applied command outcomes and mutation notifications disappear when the containing frame errors. No full transactional rollback contract is inferred.'),
    entry(178,'Live BodyState versus dirty publication','Frame changes carry current backend-neutral state for consumers.',
        ['set_velocity marks dirty_bodies', 'Impulse methods alter backend velocity but only bump revision',
         'Zero-step frames read the dirty set', 'Physical-step change detection otherwise compares transforms'],
        'Q118 proves immediate linear/angular state and published changes diverge. Transform-only step comparisons may also omit velocity/sleep changes with an unchanged pose; that wider case was not separately executed.'),
    entry(179,'Finite input versus usable geometry','Public APIs validate transforms and query values before reaching Rapier.',
        ['Finite checks precede quaternion and ray squared norms',
         'Quaternion admission checks only norm greater than epsilon',
         'Backend pose normalizes again; ray divides by the supplied norm',
         'Character up similarly normalizes a finite vector'],
        'Q119 qualifies actual admitted body and ray failures. Finite components do not prove a finite norm. The earlier Tenant candidate remains source-only; this CPU proof does not qualify its pinned kiss3d normalization implementation.'),
    entry(180,'Public bodies versus tactile backend ownership','Conatus owns IDs/publication and a private Rapier adapter owns backend handles.',
        ['Generational BodyId and collider part identity are Conatus types',
         'RapierBodyBackend owns body/collider handles and all conversion',
         'BodyWorld owns dirty sets, query stale set, interaction normalization and revisions',
         'Query conversion checks user_data identity against current owned handles'],
        'This is an actual shared owner seam. Snapshot, sorted queries and immediate despawn invalidation are useful existing behavior, now exercised by CPU tests. Generation/revision arithmetic has finite wrap/saturation limits; ordinary stale-slot tests do not establish infinite identity uniqueness.'),
    entry(181,'CPU bodies versus resident n-body lane','A future resident backend can replace private tactile machinery.',
        ['Current Engine owns BodyWorld and Rapier3D',
         'Resident is a separate padded-position n-body integrator with CSR springs and no BodyId/collision query contract',
         'Exclusion is a staged accelerator explicitly meant for a Rapier host',
         'Seiche retains separate2D layout force/runtime APIs'],
        'These lanes share concepts and some contracts but are not one current drop-in body backend. The source introduction correctly calls replacement future work. Do not claim the resident integrator already implements stable bodies, tactile contacts, layers, character movement or phase publication.'),
    entry(182,'Query refresh versus physical advancement','refresh_queries publishes touched colliders without ticking or changing the next simulation.',
        ['query_stale tracks spawns/teleports/kind and voxel edits',
         'Backend refresh writes cached collider pose and broad-phase AABB only',
         'Contacts normalize side, prediction filtering and deterministic ordering',
         'Voxel reads walk canonical8-cell blocks without copying the whole source grid'],
        'The current CPU integration tests actually pass: refresh/control trajectories agree, stale queries differ before refresh, every25 shape pair is checked at14 boundary cases, and grid read order agrees across reversed/edited/carved construction. This supports these bounded CPU behaviors, not arbitrary extreme numeric inputs or a GPU backend.'),
    entry(183,'Resident allocation identity versus metadata agreement','Burn and raw views borrow one CubeCL allocation rather than a copy.',
        ['ResidentClient registers caller host handles once and exposes its compute client',
         'ResidentChunk stores one handle per plane',
         'Raw view retains a Handle lease and identity includes buffer/offset/size',
         'Test resolves the actual Burn primitive handle independently and compares the concrete resource range'],
        'There is real implementation sharing and a stronger source test than merely comparing two producer-made metadata records. These resident tests were read only. No current GPU allocation identity, second-consumer device integration or zero-copy performance receipt was executed.'),
    entry(184,'Resident stamps and host read epochs','ReadEpoch is host-issued and every view carries authoritative snapshot metadata.',
        ['Chunk receives caller revision/read epoch',
         'Commit patches refuse stale/nonadvancing stamps and old views retain old stamps',
         'Resident n-body lease instead fills ReadEpoch from its own revision counter',
         'repulse_only changes forces through &self without incrementing that revision',
         'Raw/Burn views can retain aliases while another view writes the allocation'],
        'One structural stamp type is not one uniform host-scheduler authority. Old-view rejection and queue ordering remain explicit consumer duties; immutable stamps do not make aliased GPU bytes immutable. Host-epoch versus local n-body revisions and force-only writes need qualified identity policy before an all-writers freshness claim.'),
    entry(185,'Resident patch admission versus durable commit','Patches publish a new resident revision only after checking every range.',
        ['Single-plane operation refuses multi-plane bundles',
         'Stamp, dirty bounds, nonempty ranges, types, byte overflow, alignment and overlap checked before writes',
         'Accepted writes use the supplied queue and publish stamp/dirt together',
         'Caller owns source authorization and submission/read boundary'],
        'The code has concrete validation-before-write and explicit unsupported bundle refusal, rather than silently restamping unrelated planes. It does not prove durable record mutation, correct caller queue identity, dirty coverage of every changed element or completion of GPU writes. Tests read include retained allocation/content and rejected second-range controls; not run here.'),
    entry(186,'Lease shape guard and resident dispatch admission','A consumer cannot read stale or ill-fitting views when it uses the shared lease.',
        ['Validated plane construction uses checked shape multiplication',
         'Public SpatialLease.byte_len uses unchecked products and fits checks only against stated size',
         'Resident.new checks CSR offset count but not monotonic/end/target bounds or a zero-body guard',
         'Public binning dimensions and general n-body Params have no common admission layer',
         'Exclusion explicitly handles empty input and rejects unsuitable cell grids by fallback'],
        'Construction checks and consumer guard have different scopes. Overflowing public lease metadata, malformed CSR, empty n-body dispatch and extreme grid arithmetic are source-qualified candidates; no malformed GPU workload was launched. A stamp/range DTO alone cannot certify valid-read timing, buffer usages or semantic coordinates.'),
    entry(187,'Force formula ownership and GPU quality bars','Resident kernels compute the CPU law, with relative-error receipts.',
        ['Softened resident repulsion is distinct from hard-floor/cutoff NodeExclusion',
         'CubeCL pair and cell kernels repeat the explicit backend formula',
         'CPU reference and direct Seiche implementations remain separate qualified anchors',
         'Cell binning atomically scatters neighbours so sum order need not be bit-identical',
         'Tests count actual dispatches, cutoff-pruned pairs, scan values and sign-flipped overlap failures'],
        'This is deliberate backend duplication requiring parity, not one executable force authority. Relative tolerance and exact candidate enumeration do not mean bitwise GPU determinism. Softened-law test asserts mean error only; settle test asserts late speed less than early rather than an absolute rest threshold. Source quality bars were read, not rerun.'),
    entry(188,'Resident evidence, failures and migration claims','Device dispatch/readback proves execution and a dropped reader does not leave mapped staging storage.',
        ['PendingExclusion exposes ready/error/taken states and adopts unfinished reads on Drop',
         'ResidentClient.poll_device reaps adopted futures',
         'dispatch counters count launches rather than completed successful answers',
         'GPU test admission failure returns with explicit skip messages',
         'Migration comment says no committed SPIR-V artifact means the fallback question no longer exists'],
        'Completion, successful answer and accepted device step remain distinct even with runtime shader compilation. Removing a committed artifact does not prove the selected adapter/backend ran, nor eliminate fallback/error questions. Current pass executes CPU only; dropped-read/GPU tests remain historical or source evidence, not fresh GPU receipts.'),
]
upsert(authority['entries'],entries,'id')
upsert(notes['candidates'],[
    {'title':'Resident lease shape guard does not use checked byte arithmetic',
     'source':'crates/conatus/conatus/src/resident/chunk.rs:SpatialLease::byte_len and fits',
     'note':'Plane construction checks shape multiplication, but public lease fields are independently constructible and byte_len multiplies usize without checked arithmetic. Extreme metadata can panic with overflow checks or wrap with them disabled, weakening fits. No GPU buffer was created merely to exercise this metadata edge. Checked admission should cover the exported contract rather than relying only on producer constructors.'},
    {'title':'Resident n-body and public binning admission is narrower than safe dispatch assumptions',
     'source':'crates/conatus/conatus/src/resident.rs:Resident::new/repulse_only; resident/binning/mod.rs:Grid/bin; resident/exclusion.rs:grid',
     'note':'N-body constructor validates CSR offset length only, uses unsafe unchecked launches, and lacks explicit empty-input and CSR target/range checks. Exclusion handles empty input but extreme finite layout differences can overflow to infinity before width as usize plus1, preceding checked_mul fallback. No malformed GPU workload or feature build was attempted. Common bounded admission and refusal policy may exceed one-hour audit diversion; retain this larger follow-up.'},
    {'title':'Resident local field revision is not a host schedule read epoch',
     'source':'crates/conatus/conatus/src/resident.rs:repulse_only and lease_of; resident/chunk.rs:ReadEpoch and views',
     'note':'ReadEpoch documentation explicitly assigns host schedule authority, while n-body leases construct epochs from local step revision. Force-only writes do not advance that revision. Consumer stamp checks and aliased plane bytes need a defined write/submission/read lifetime contract. No stale GPU consumer was executed; this is a source-qualified ownership inconsistency, not a demonstrated synchronization race.'},
],'title')
inventory,coverage=load('source_inventory.json')['files'],load('read_coverage.json')
complete=sorted(p for p,v in inventory.items() if p.startswith('crates/conatus/conatus/')
    and coverage.get(p,{}).get('sha256')==v['sha256'] and coverage[p]['through']==v['chars'])
assert len(complete)==23 and sum(inventory[p]['chars'] for p in complete)==274139
assert all(sha(ROOT/p)==inventory[p]['sha256'] for p in complete)
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==901 and stats['read_chars']==12839219
assert not stats['changed_since_inventory'] and not stats['pending_chunks']
next_path=next(p for p,v in sorted(inventory.items()) if p.startswith('crates/conatus/modulus/')
    and coverage.get(p,{}).get('through',0)<v['chars'])
next_read={'path':next_path,'start':coverage.get(next_path,{}).get('through',0)}
notes['current_area']='Conatus23/23 fully read; CPU runtime/queries and optional resident source reviewed'
notes['next_area']='crates/conatus/modulus'
notes['checkpoint']={'classification':'progress','goal_status':'active; full-source objective incomplete',
    'coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
    'new_receipts':['conatus_review_receipt.json'],'next_read':next_read,'production_edits_this_turn':False,
    'classification_of_previous_turn':'progress','coverage_checkpoint':stats,
    'diagnostic_outcomes':'44 unchanged existing CPU tests and5 generated observations pass. Q117/Q118/Q119 reproduced. Resident GPU not executed.',
    'user_instruction':'Audit all Mere source; preserve concurrent work; do not change production code.',
    'coverage_note':'23 more full files and274139 chars this continuation. Full Mere review remains active.'}
upsert(notes['source_qualifications'],[{'area':'Conatus complete source review and public CPU observations',
    'note':'All23 inventoried files read,274139 chars. Current default library Cargo build succeeds.19 original unit tests and25 original CPU integration tests compile/run with rustc against current dependencies; optional resident files/doctests not included, so not a Cargo/package/GPU gate. Five generated native observations pass with controls. All owned executables/PDBs removed; no isolated target/home/worktree.'}],'area')
assert len({x['id'] for x in notes['findings']})==len(notes['findings'])
assert len({x['id'] for x in authority['entries']})==len(authority['entries'])
save('review_notes.json',notes); save('authority_map.json',authority)
doc=old['document']['path']; destination=ROOT/doc; before=destination.read_bytes()
assert sha(destination)==old['document']['sha256']
addition='''

### Read-only Conatus complete source review, 2026-10-16

All23 Conatus source files are fully read,274,139 characters: body vocabulary,
fixed clock, commands/phases, private Rapier adapter, queries/refresh, resident
allocation/kernel/exclusion/binning code and every test source. Mere coverage
is901/3352 files and12,839,219/36,854,820 characters. The full-source goal remains
active. Production/concurrent work are preserved. Resume at crates/conatus/modulus.

Q117: a system failure consumes unexecuted fixed time and discards publication
for work already applied. With one injected AfterPhysics failure, advance50000us
grants3 ticks, executes1 and returns Err; advance0 has no backlog and never runs
the remaining2. A second observation applies a queued spawn and has a pending
despawn, then fails: live state contains the spawn but its command result/change
and prior removal disappear from both the error and next successful zero-step
frame. No-error controls publish everything and execute3 steps. The primitive
clock correctly retains capped backlog; Engine consumes its grant before work
and loses local aggregates on error. Preserve unused grant and partial evidence.
Retry/resume phases, resource side effects and command outcome recovery need a
larger policy decision; no transactional rollback requirement is invented here.
That work may exceed the user's one-hour audit diversion and stays documented.

Q118: immediate impulses alter BodyState without marking publication dirty.
Native linear/angular impulses produce speeds0.47746482 and1.193662, but advance0
reports changed[]. Setting the identical already-current velocity then publishes
it. Both impulse methods bump revision without dirty_bodies insertion; set_velocity
does insert. This is a contained publication repair, independent of when a later
moving transform happens to be emitted. No renderer/network/save consumer ran.

Q119: finite components can overflow a norm and produce invalid admitted geometry.
Quaternion[f32::MAX,0,0,1] is accepted but public state returns[0,0,0,0]; moderate
nonunit input normalizes correctly. A ray at(0,4,0) through a present unit box hits
at3 units with directions(0,-1,0) and(0,-10,0), but finite(0,-f32::MAX,0) returns
Ok(None). Squared norm becomes infinity, then normalization collapses the data.
Use stable norm arithmetic or refuse unusable derived norms before admission.
Character-up and Tenant share a source pattern; they were not executed here.

The current default Conatus library built successfully.44 existing CPU tests
passed:19 original library tests and25 tests from the four original CPU integration
files, compiled directly with rustc against the current Cargo-built dependencies.
This is an explicit CPU gate, not a Cargo/package/doctest or resident feature gate.
Five generated public-API observations also pass and demonstrate the defects,
not fixes. No resident GPU or renderer was built/executed. Owned executables/PDBs
were removed; normal build cache remains at C:/t/cargo-targets/mere. No isolated
target, Cargo home or worktree was created.

Useful shared ownership is present. Conatus owns generational public IDs and
publication; one private Rapier adapter owns backend handles/conversions. Query
refresh touches colliders/broad-phase without stepping. Fresh CPU tests check
refresh/control trajectories, stale-versus-refreshed queries,25 contact shape
pairs at14 boundary cases, and canonical voxel order across reversed/edited/
carved grids. These bounded successes do not qualify extreme numeric admission.
The optional Resident integrator remains a separate n-body lane, not a current
replacement for BodyWorld's tactile/query/identity contract. Exclusion explicitly
accelerates a Rapier host; Seiche still owns its2D layout runtime.

Resident sharing has concrete source evidence: one CubeCL handle per plane,
raw views retain its lease, and Burn views use that handle. The source test
resolves Burn's actual primitive resource range independently, beyond comparing
two producer-made metadata records. Patch admission checks every range before
writes and explicitly refuses multi-plane restamping. None of these GPU tests
was run here. Adapter/device admission failures return with skip messages, so
a green count alone would not prove execution. Runtime shader compilation also
does not eliminate backend/fallback/completion questions merely because there
is no committed SPIR-V artifact.

Hairier resident issues stay qualified: n-body construction checks CSR length
but lacks complete range/target/zero-input admission; public binning/grid and
parameter arithmetic have different guards; public lease byte_len uses unchecked
multiplication despite the stronger checked plane constructor. ReadEpoch is
documented as host-issued, yet n-body leases use a local step revision and force-only
writes do not advance it. Immutable stamps do not freeze aliased GPU bytes.
No malformed GPU workload, stale consumer or overflow allocation was attempted.
Shared admission and writer/submission/read lifetime policy belong in separate
work. Atomic neighbour scatter also means exact candidate enumeration is not
bitwise GPU replay; current tests use relative error and distinct softened versus
hard-floor/cutoff laws. Existing quality bars are source-read only in this pass.

Exact source/preservation hashes, compiler commands and execution limits are in
the [Conatus review receipt](../../../support/receipts/authority-success-audit/conatus_review_receipt.json).
'''
assert b'### Read-only Conatus complete source review, 2026-10-16' not in before
destination.write_bytes(before+addition.encode())
diff=subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True,text=True)
(HERE/'conatus_review_diff_check.txt').write_text(diff.stdout+diff.stderr,encoding='utf-8')
assert diff.returncode==0
evidence=['conatus_cpu_probe.rs','run_conatus_cpu_probe.py','conatus_cpu_library_build.jsonl',
    'conatus_cpu_library_build_stderr.log','conatus_cpu_probe_build.log','conatus_cpu_probe_output.log',
    'conatus_cpu_probe_receipt.json','conatus_existing_cpu_tests.rs','run_conatus_existing_cpu_tests.py',
    'conatus_existing_library_build.log','conatus_existing_library_tests.log',
    'conatus_existing_integration_build.log','conatus_existing_integration_tests.log',
    'conatus_existing_cpu_tests_receipt.json','conatus_review_diff_check.txt']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'kind':'Complete Conatus source review; full Mere source objective incomplete',
    'production_edits_this_pass':False,'new_findings':[x['id'] for x in findings],
    'authority_entries':[x['id'] for x in entries],'coverage':stats,'complete_conatus_files':complete,
    'source_sha256':{p:sha(ROOT/p) for p in complete},'preserved_production_and_test_hashes':preserved,
    'next_read':next_read,'diagnostic_tests':5,'diagnostic_exit':0,'existing_cpu_tests':44,
    'existing_cpu_test_exit':0,'cargo_package_gate':False,'resident_execution':False,
    'generated_output_removed':True,'retained_isolation':[],
    'document':{'path':doc,'sha256':sha(destination),'preserved_prefix_bytes':len(before),
        'preserved_prefix_sha256':hashlib.sha256(before).hexdigest()},
    'diff_check':0,'evidence_sha256':{p:sha(HERE/p) for p in evidence},
    'limits':['Full Mere source coverage incomplete; no production fixes.',
        'Original CPU tests run through explicit rustc commands, not Cargo package/doctest gate.',
        'No resident GPU, renderer, model or product host execution.',
        'Broader scheduler recovery and resident admission/lifetime issues remain documented.']}
assert all(sha(ROOT/p)==h for p,h in preserved.items())
assert all(sha(ROOT/p)==h for p,h in receipt['source_sha256'].items())
save('conatus_review_receipt.json',receipt)
print(json.dumps({'coverage':stats,'conatus_complete':len(complete),'findings':len(notes['findings']),
    'authority_entries':len(authority['entries']),'next_read':next_read}))
