"""Append current Registry/shell/surface/UX source review without production edits."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]; NAME='registry_ownership_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda p:json.loads((HERE/p).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('upstream_ownership_review_receipt.json'); head=git('rev-parse','HEAD'); remote=git('rev-parse','origin/main')
assert head==old['basis_commit'] and remote==old['upstream_commit']
assert git('rev-list','--left-right','--count','HEAD...origin/main').split()==['0','8']
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']}
aux=dict(old['auxiliary_read_sha256']); runtime=old['runtime_input_sha256']; shared=old['shared_working_file_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
doc=ROOT/old['document']['path']; prefix=doc.read_bytes(); assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
inventory=load('source_inventory.json')['files']; coverage=load('read_coverage.json')
prefixes=['crates/system/registry/','crates/system/shell-state/','crates/system/surface-api/','crates/system/ux-events/']
sources={p:sha(ROOT/p) for p in inventory if any(p.startswith(area) for area in prefixes)}
assert len(sources)==72
for p,h in sources.items():
    assert inventory[p]['sha256']==h==coverage[p]['sha256']
    assert coverage[p]['through']==inventory[p]['chars']==len((ROOT/p).read_bytes().decode('utf-8-sig',errors='replace'))
assert load('pending_read.json')==[]
asset='crates/system/registry/assets/knowledge/udc_seed.json'; aux[asset]=sha(ROOT/asset)
probe=load('registry_owner_probe_v2_receipt.json'); initial=load('registry_owner_probe_receipt.json')
assert probe['exit']==0 and probe['passed']==89 and probe['original_tests']==84 and probe['ignored']==1
assert initial['exit']==101 and initial['passed']==0
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['copied_tree_sha256'].items())
assert all(sha(ROOT/v['source'])==v['sha256'] for v in probe['copied_source'].values())
assert all(not Path(row['path']).exists() for row in probe['removed_outputs'])
catalog=load('registry_ux_catalog_check_receipt.json'); assert len(catalog['missing'])==4
assert sha(ROOT/catalog['source'])==catalog['source_sha256']
notes=load('review_notes.json'); authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q385' and authority['entries'][-1]['id']=='A1152'
findings=[
 ('Diagnostic descriptor sampling is ignored by the default emission gate',
  'crates/system/registry/src/diagnostics/descriptor/registry.rs:72,159,216; descriptor.rs:164',
  'Default tile-activity descriptor declares SampleRate(0.25) and KeepRecent(256), but its separate ChannelConfig defaults to1.0 and100. The actual gate admits8/8events; setting config0.25 explicitly admits2/8. Registration never derives effective config from the supplied descriptor. Retention mismatch is observed metadata only; no downstream retention-store behavior is claimed.',
  'CPU owner observation on unchanged source. No in-repo production should_emit_and_observe caller found. A Registry API/configuration defect, not a demonstrated product telemetry overload.',
  'Bounded configuration precedence fix candidate; preserve explicit persisted/user overrides and distinguish descriptor defaults from actual collector retention.'),
 ('Absent native runtime produces Active mods and an independent global capability success',
  'crates/system/registry/src/mod_loader/loader/registry/registry_impl.rs:384; loader/registry.rs:63-82',
  'ModRegistry::new with no injected runtime resolves and loads mod:web-runtime asActive and publishes viewer:webview. An injected refusing NativeModRuntime produces no loaded native mods or webview capability. runtime_has_capability nevertheless returns true because it initializes a separate no-runtime registry once. Existing tests intentionally expect the no-op success, while some API comments promise activation errors.',
  'Actual public registry/global query; no native hook, web renderer or host is invoked in the absent-runtime case. Explicit rejecting runtime is a synthetic control. Only document-host implements the WASM trait locally; no in-repo production consumer of the global capability query found. No present product availability failure asserted.',
  'High-effect ownership migration: derive readiness from the live host registry or explicitly separate declaration/discovery from activation. Audit documents this rather than selecting host policy or redesigning capability ownership.'),
 ('Failed provider does not prevent dependent activation or capability publication',
  'crates/system/registry/src/mod_loader/loader/registry/registry_impl.rs:214-305',
  'A two-mod dependency order is resolved successfully. The documented activation callback refuses the provider; load_all_with_extensions still invokes the dependent callback and records itActive, emits load_succeeded and publishes itscapability while the provider isFailed. All-success and missing-requirement controls behave as expected. Topological presence/order is not an active-dependency check.',
  'Actual ModRegistry public extension seam using synthetic manifests and explicit injected activation failure. No real mod dependency or product host was activated.',
  'Bounded skip/refusal for unsatisfied active requirements, with precise blocked status/diagnostic; multiple-provider selection and rollback policies remain broader followups.'),
 ('Repeated load bypasses quarantine and loses the record for an unremoved extension',
  'crates/system/registry/src/mod_loader/loader/registry/registry_impl.rs:231-268,289-299,321-364',
  'First activation installs a synthetic oldAction then fails; rollback refuses and leaves its extension record quarantined. A second load_all_with_extensions activates it again without removal, overwrites records with a newAction, and marksActive. Successful unload removes only the newAction and marksUnloaded while the old callback-owned effect remains. There is no lifecycle status guard before reactivation.',
  'Real registry states/records and documented synthetic action-install/removal callbacks in a local vector. No real action registry/native effect, WASM instance or product extension leak demonstrated. Existing first-failure quarantine and unload controls pass.',
  'Bounded refuse/skip Active orQuarantined reactivation until explicit recovery; retain all outstanding records. Recovery across host runtimes is a separate policy decision.'),
 ('Input remap and chord string codecs cannot represent legal delimiter characters',
  'crates/system/registry/src/input.rs:246,348,424-440',
  'Keycode::Char accepts anychar. A remap whose old key ispipe encodes then fails decode because the same delimiter splitsfields. A chord containing greater-than likewise labels then fails parse because that char splits chordsteps. Ordinary letterremap and ordinarychord controls round-trip; a standalone greater-than key remap also round-trips.',
  'Actual unchanged public InputBinding/InputBindingRemap codecs; no UI, key-event adaptation or persisted settings file executed. Default bindings do not contain these delimiter chars.',
  'Bounded versioned escaped/structured encoding or explicit validation. Preserve existing stored bindings and distinguish display labels from serialization.'),
 ('UX channel catalogue omits four IDs that its canonical mapper emits',
  'crates/system/ux-events/src/ux_diagnostics.rs:108-158,172-218,429-444',
  'Both exhaustive surface match tables include InspectorPane and StewardPane opened/dismissed channels. all_channel_ids omits allfour, listing36surface channels while mapper has40. Plus2dispatch IDs gives38listed versus42emittable. Existing test named covers_every_surface_and_dispatch_kind asserts the stale fixed38 and selected old examples.',
  'Complete source read and exact function-body literal reconciliation, not a fresh Rust bridge/host test. No current in-repo production consumer of all_channel_ids found. No live dropped UX event claimed.',
  'Bounded derive catalogue from the same surface mapping and exercise allSurfaceId variants, rather than retaining a second list and fixed count.')]
new_findings=[]
for n,(title,source,evidence,scope,reason) in enumerate(findings,386):
    finding={'id':f'Q{n}','title':title,'source':source,'evidence':evidence,'scope':scope,
      'status':'reproduced scoped CPU owner behavior; documented, not repaired' if n<391 else 'source-confirmed catalogue mismatch; documented, not repaired',
      'verification':NAME,'diagnostics':['registry_owner_probe_v2_receipt.json'] if n<391 else ['registry_ux_catalog_check_receipt.json'],
      'repair_triage':{'disposition':'documented under read-only full-source audit','reason':reason}}
    notes['findings'].append(finding);new_findings.append(finding['id'])
rows=[
 ('Registry extraction and actual current adoption','Nine old registries are now modules of one feature-gated crate, with theme delegated to Tabard.',
  ['crates/system/registry/src/lib.rs','crates/system/registry/Cargo.toml','crates/script/document-host/Cargo.toml','crates/script/document-host/src/runtime.rs'],
  'An actual command-menu dependency shares default command IDs. Current production consumer search finds document-host implementing the WASM DI trait; no in-repo product ModRegistry construction, global sender installation, should_emit_and_observe or global capability query call. Historical Graphshell/iced startup prose is not proof of current adoption. External consumers are not censused here.'),
 ('Diagnostic metadata versus effective policy','Descriptor sampling/retention/schema and mutable ChannelConfig coexist.',
  ['crates/system/registry/src/diagnostics/descriptor.rs','crates/system/registry/src/diagnostics/descriptor/registry.rs','crates/system/registry/src/diagnostics/descriptor/types.rs'],
  'Q386 confirms default sampling is not applied. Payload schemas are declared/validated as descriptors, not actual event payload admission. Mod/Verse namespace checks exist; generic runtime registration trusts caller-supplied ownership, so this is not a sandbox capability boundary.'),
 ('Correlated invariant ownership is explicit but optional','One bounded tracker separates keyed monotonic and legacy epoch/FIFO observations.',
  ['crates/system/registry/src/diagnostics/correlation.rs','crates/system/registry/src/diagnostics/descriptor/invariant.rs','crates/system/registry/src/diagnostics/descriptor/registry.rs'],
  'Exact run/source/operation matching, late-terminal timeout, bounds/loss/time reversal controls all pass in scoped tests. The global compatibility wrapper still invokes uncorrelated epoch observation. No current host adoption of either observation API found; legacy time-error conversion quietly produces an empty violations vector, distinct from richer keyed error/stats.'),
 ('Diagnostic staging and downstream storage remain separate owners','Portable producers use one configurable nonblocking bounded staging queue; independent readers belong downstream.',
  ['crates/system/registry/src/diagnostics/emit.rs','crates/system/registry/src/diagnostics/emit/queue.rs'],
  'One-time sender install refuses replacement, scoped thread guards restore prior sender, and rejection/expiry/disconnect counters distinguish loss from admitted sequence gaps. Original controls pass. No in-repo host installation found; descriptions do not prove product collection. This is useful shared ingress, not another durable observation authority.'),
 ('Input command identity is shared while adaptation remains stranded','Default key bindings call command-menu catalogue IDs; two input-only actions are explicitly distinguished.',
  ['crates/system/registry/src/input.rs','crates/system/registry/src/input/defaults.rs','crates/system/registry/src/input/registry.rs'],
  'All21 shared default IDs are checked against actual command-menu. egui conversion bodies remain cfg(egui-host), but the crate has neither that feature nor egui dependency. No current product InputRegistry caller found. This is dormant donor adaptation; do not treat its presence as a functioning universal keyboard path. Q390 records legal-character encoding failure.'),
 ('Knowledge tags have one local seed and a lossy compact representation','UDC seed travels with registry; schema is explicitly unknown rather than silently treated as UDC.',
  ['crates/system/registry/src/knowledge.rs','crates/system/registry/assets/knowledge/udc_seed.json'],
  'Seed data fully read as auxiliary input. Numeric/dot normalization strips compound separators and compact distance strips dots, so complex UDC expressions can collapse. No product caller or semantic misclassification executed. Define accepted tag grammar before claiming complete UDC validation; schema provider/mod seed registration remain explicitly future work.'),
 ('Profile metadata is distinct from runtime conformance','Canvas/workbench share ProfileRegistry, while Viewer repeats subsystem capability shape.',
  ['crates/system/registry/src/layout/profile_registry.rs','crates/system/registry/src/layout/canvas.rs','crates/system/registry/src/layout/workbench_surface.rs','crates/system/registry/src/viewer.rs'],
  'Resolution exposes requested/resolved IDs and fallback explicitly. Default profiles and unknown viewer capability lookups reportFull across allsubsystems without runtime measurement. Partial/None reason requirement is doc-only on public/serde fields. These declarations do not establish accessibility/security/storage/history acceptance. Registration/lookup trim conventions and profile_id duplication need admission followup.'),
 ('Lens presets remain separate from the active physics implementation','Registry owns donor physics intent types; consumers must map them to the active runtime.',
  ['crates/system/registry/src/lens.rs','crates/system/registry/src/lens/physics.rs','crates/system/registry/src/lens/physics_config.rs','crates/system/registry/src/lens/registry.rs'],
  'Source explicitly admits ForceDirectedState bridge was dropped. ThemeData and glyph types are real re-exports from Tabard/kernel. LensDefinition retains layout mode plus algorithm ID; requires_graph_context is declared but not checked by resolve_for_content. Composition preserves base physics/layout and merges filters/first available overlay/theme. No current product resolver adoption found; avoid conflating these legacy presets with live cartography physics.'),
 ('Live activation and global capability declarations have split authority','ModRegistry has injected runtime ownership but runtime_has_capability builds an independent once-cached registry.',
  ['crates/system/registry/src/mod_loader/loader/registry.rs','crates/system/registry/src/mod_loader/loader/registry/registry_impl.rs','crates/system/registry/src/mod_loader/loader/registry/registry_default.rs'],
  'Q387 reproduces no-runtimeActive and separate globaltrue despite explicit runtime refusal elsewhere. Hardcoded core/web manifests remain compiled inventory declarations. Some no-runtime error comments conflict with deliberate no-op implementation and tests. Actual host adoption and late lifecycle/environment changes are not represented by thatOnceLock.'),
 ('Dependency planning and runtime readiness are separate checks','Topological sort resolves declared providers, but load loop does not require them to beActive.',
  ['crates/system/registry/src/mod_loader/loader/free_fns.rs','crates/system/registry/src/mod_loader/loader/registry/registry_impl.rs'],
  'Q388 proves failed provider followed by dependentActive. Duplicate providers select first manifest; registry constructs that input fromHashMap values, while duplicate modIDs are collapsed before the free resolver duplicate check. No duplicate-provider product nondeterminism executed. Provider choice and actual active readiness need one owner.'),
 ('Quarantine does not own later activation admission','Extension records support reverse rollback and quarantine, yet load loop re-enters every resolved mod regardless of status.',
  ['crates/system/registry/src/mod_loader/loader/registry/registry_impl.rs','crates/system/registry/src/mod_loader/loader/types.rs'],
  'Existing rollback/unload failure controls preserve outstanding records on first failure. Q389 demonstrates repeatload overwrites them and laterUnloaded leaves a synthetic effect. NativeRuntime has activate only; native default activation returns no teardown records, whereas WASM uses a deactivation record. Cross-runtime recovery is documented for later ownership work.'),
 ('Protocol contract map is a declaration seam, not request execution','One map stores scheme-to-handler IDs; providers append registrations and resolution reports only scheme/fallback.',
  ['crates/system/registry/src/protocol.rs','crates/system/registry/src/protocol/contract.rs','crates/system/registry/src/protocol/provider.rs'],
  'The tower ProtocolHandler trait is not stored or invoked by thisregistry, and registered handler ID is not exposed by resolution. No in-repo runtime caller found. Modern fetch/document engine routing must be compared at actual consumers before this dormant catalogue is called the canonical runtime dispatch authority.'),
 ('Viewer selection retains multiple independent rules and permissive declarations','URI selector, MIME/kind selector and Plaintext handler use different tables/precedence.',
  ['crates/system/registry/src/viewer.rs','crates/system/registry/src/viewer/tests.rs'],
  'select_for_uri forces smolweb schemes before MIME and respects configured fallback differently from select_for; extension scan uses raw URL suffix and does not normalize its case. Unknown capabilities defaultFull; unknown runtime viewer ID mapsPlaceholder but still receives fullsubsystem declaration. Scheme logic bypasses registration. Source-only, no current in-repo product caller found and no live wrong renderer/security acceptance claimed.'),
 ('Shell-state migration is actual re-export consolidation','Every session/chrome/UX module now delegates to Chrome or UX Events.',
  ['crates/system/shell-state/src/lib.rs','crates/system/shell-state/src/authorities.rs','crates/system/shell-state/src/frame_model.rs','crates/system/shell-state/src/ux_observability.rs'],
  'All12files read. These are compatibility imports, not duplicated state stores or independent focus authority. The crate root still says itowns those types, and per-module docs retain future destination notes. Correct ownership prose alongside removal/adoption work; do not invent duplicate implementations from duplicate import paths.'),
 ('Surface and settings contracts keep authority with products','Data-only surface descriptions and typed settings projections do not own factories, settings storage or an executable globalregistry.',
  ['crates/system/surface-api/lib.rs','crates/system/surface-api/settings.rs','crates/system/surface-api/surface.rs'],
  'All4files read. SettingsProjection::resolve delegates one provider.describe call; provider.apply owns validation/persistence. Surfaceaccepted_source consistency is an explicit host registration obligation, not enforced by the DTO. Existing Cambium surface/Distillery descriptor and Pelt appearance consumer references are found, without fresh external/headed acceptance.'),
 ('UX mapping has one shared function but repeats an incomplete catalogue','Kernel SurfaceId is re-exported; UX event_channel defines event lowering, and all_channel_ids repeats channelstrings.',
  ['crates/system/ux-events/src/ux_observability.rs','crates/system/ux-events/src/ux_diagnostics.rs','crates/graph/graph-kernel/src/accessibility.rs'],
  'Q391 reconciles40mapped surfacechannels versus36listed plus2dispatch. Canonical single-function claim applies to actual lowering only, not automatic catalogue completeness. No local production host UxObservers or diagnostics bridge caller found. External consumer adoption is not audited by that negative search.'),
 ('UX observations and probes are passive contracts with caller-owned stream scope','Observers fan out synchronously; count/ring observers and four probes consume caller-provided events.',
  ['crates/system/ux-events/src/command_surface_telemetry.rs','crates/system/ux-events/src/ux_observability.rs','crates/system/ux-events/src/ux_probes.rs','crates/system/ux-events/src/ux_probes/productive_selection.rs'],
  'Per-runtime telemetry replaces oldglobals; snapshot/events are separate mutexcells, not one atomic snapshot. Rings bound count only, with no bytes/age/lossaccounting, unlike Registry ingress. Probe failure vectors are unbounded until drained. Surface IDs lack pane/window/run keys, and productive pending state has no final inspection/flush; empty drain alone cannot establish final productivity or stream completeness. Confirmation probe onlyobserves and is not command authorization. Source-only thispass.')]
entries=[]
for number,(domain,claim,paths,assessment) in enumerate(rows,1153):
    entry={'id':f'A{number}','domain':domain,'claim':claim,'implementations':paths,'assessment':assessment,
      'state':'complete current source read; execution limited to separately named scoped tests; full repository audit ongoing','verification':NAME}
    authority['entries'].append(entry);entries.append(entry['id'])
followups=[
 ('Registry adoption and capability migration','Finish live-registry readiness ownership before enabling the dormant global capability query. Keep discovery, supplied runtime activation, active dependency checks, quarantine recovery and teardown distinct. Q387-Q389 are scoped API reproductions; no current product caller is established.'),
 ('Viewer/lens/profile migration','Compare these dormant donor tables at actual consumers. Viewer unknownFull, distinct selection precedence, layout mode/algorithm IDs, unused graph requirement and physics mapping need ownership decisions; avoid another mirrored runtime table.'),
 ('UX terminal and bounded observation scope','Determine pane/window/run identity and explicit end-of-stream/pending semantics for productive-selection checks. Keep recorder count bounds, unbounded probe failure vectors and observation loss distinct from newer Registry/Apparatus byte/time limits. No live UI or exact stream omission probe was run.'),
 ('Compatibility documentation ownership','Shell-state modules consolidate via re-export. Root ownership claims and Registry historic extraction/startup references are stale. Correct docs in the owning migration rather than deleting shared compatibility surfaces during this read-only review.')]
for area,detail in followups:notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2721 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
for area in prefixes:
    paths=[p for p in sources if p.startswith(area)]
    notes['completed_source_areas'].append({'prefix':area,'files':len(paths),'chars':sum(inventory[p]['chars'] for p in paths),
      'verification':NAME,'basis_commit':head,'qualification':'Every inventoried source/manifest read at exact current hashes. Only separately named Registry owner composition tests executed; no full native product/runtime/optional acceptance.'})
next_read={'path':'crates/system/tabard/Cargo.toml','start':0,'reason':'Continue full source review through Tabard, then remaining port/scripts/patch/evidence source; UX Events was read next to its Registry/shell-state ownership seam.'}
notes['current_area']='Registry, shell-state, surface-api and UX Events complete source read; scoped Registry owner verification'
notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),
 'authority_entries':len(authority['entries']),'next_read':next_read,
 'pending_verification':'No audit-owned process live. Shared workspace has unrelated active compilers; their work is preserved.89scoped Registry composition tests pass,1ignored; UX catalogue is source-only. Upstream fetched unchanged00e44428, all38diffs previously reviewed, not integrated due shared report conflict. Tabard and remaining source unread. Full repository audit incomplete.',
 'audit_authored_production_edits_this_turn':False})
notes['checkpoint']['new_receipts'].extend([NAME,'registry_owner_probe_receipt.json','registry_owner_probe_v2_receipt.json','registry_ux_catalog_check_receipt.json'])
addition=f'''

### Registry, session wrappers, surface contracts and UX audit, 2026-10-17

Read all45Registry source/manifest files (400,034characters), all12shell-state files (8,340), all4surface-api files (14,366), and all11UX Events files (78,278). UDC seed JSON is separately read auxiliary input, not source-denominator credit. Current full-source coverage is{stats['complete_files']:,}/{stats['total_files']:,}files and{stats['read_chars']:,}/{stats['total_chars']:,}characters, with no stale or pending chunks. The remaining full repository source is still unread, including Tabard, remaining product ports and supplier patches. A fresh fetch leaves origin unchanged at{remote}; the already-reviewed38upstream diffs remain separate from current checkout{head}. Fast-forward still overlaps the shared report. No stash/revert/commit/push or production edits.

Q386-Q391 record six issues. Q386: declared25percent tile-activity sampling is ignored by generic configdefaults, admitting8/8events; explicit config25percent admits2/8. Retention256versus100 is a metadata mismatch, not a downstream storage reproduction. Q387: absent native runtime marks the web-runtime modActive and publishes viewer capability; an explicit refusing runtime does not. The global query still returns true from its separate once-cached no-runtime registry. Q388: a failed provider does not stop its dependent callback, Active status, success diagnostic or published capability. Q389: a secondload bypasses quarantine and replaces the outstanding old extension record; later successful unload reportsUnloaded while the synthetic old effect remains. Q390: legalpipe remap / greater-than chord characters fail string round-trip; ordinary controls pass. Q391: UX mapper handles40surface channels but all_channel_ids lists36, omitting Inspector and Steward opened/dismissed IDs; the completeness-named test pins the stale38total with dispatch channels instead of exercising every surface.

Verification:89tests passed,84original and5audit, with1existing native-inventory test ignored. The initial external-path audit harness failed compilation and is preserved with zero testcredit. Corrected composition compiles a byte-identical current Registry source tree, links diagnostics/input/layout/mod_loader with their originaltests and actual command-menu, and uses only the exact data-only ViewerSubsystemCapabilities fragment needed by extension records. It does not execute viewer routing, knowledge/lens/protocol, graph-kernel, real native mods, WASM instances, host startup, browser, network orGPU. Activation/removal failure cases use the documented injected callback seam with synthetic effects; the absent-runtime case calls no native hook atall. UX catalogue reconciliation is a source-body literal comparison, not a fresh Rust/host test. Audit executable/PDB removed; reusable supplier cache stays under the stable shared Mere target/build path. Other active workspace compilers are unrelated and untouched.

A1153-A1169 track the ownership seams, including real consolidation and unfinished adoption. Default input command IDs share command-menu; lens theme and glyph types re-export Tabard/kernel. Registry's correlated bounded watchdog and nonblocking staging queue have meaningful exact-key/loss controls; the compatibility observer remains uncorrelated. Descriptor schemas/conformance metadata do not themselves validate livepayloads or guarantee runtime behavior. Native declarations, separate global readiness, topological presence and lifecycle state still divide activation authority. Viewer URI/MIME-kind/plaintext handlers repeat selection rules, and lens donor physics/config IDs have no current local runtime consumer. Those broader migrations are documented rather than choosing host policy in this pass.

Current production reference search finds document-host implementing the WASM runtime trait, but no local product ModRegistry construction, global capability query, sender installation, diagnostic observation gate, UxObservers host or UX channel bridge call. That is a bounded repository census, not a claim about external Turnstone/other consumers. Shell-state is actual re-export consolidation into Chrome/UX Events, not duplicate focus/session state; its root owns-types prose is stale. Surface/settings DTOs deliberately leave admission consistency, apply validation and storage with hosts/products. UX recording/probes remain passive: count-only rings, unbounded failure lists, missing pane/window/run correlation and no productive-pending terminal inspection are separate documented followups. Empty probe failures do not prove event completeness or runtime success.

Evidence: [source and ownership checkpoint](../../../support/receipts/authority-success-audit/{NAME}), [scoped Registry tests](../../../support/receipts/authority-success-audit/registry_owner_probe_v2_receipt.json), [preserved compiler failure](../../../support/receipts/authority-success-audit/registry_owner_probe_receipt.json), and [UX catalogue source check](../../../support/receipts/authority-success-audit/registry_ux_catalog_check_receipt.json). Previous report bytes, guarded source, shared WIP and index remain unchanged. No isolated target, Cargo home or worktree retained. Next source: Tabard.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for p,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/p).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_registry_ownership_review.py','registry_owner_probe.rs','run_registry_owner_probe.py',
 'registry_owner_probe_receipt.json','registry_owner_probe_output.log','registry_viewer_capabilities_projection.rs',
 'cpu-registry-runtime/Cargo.toml','cpu-registry-runtime/Cargo.lock','run_registry_owner_probe_v2.py',
 'registry_owner_probe_v2_receipt.json','registry_owner_probe_v2_output.log','cpu-registry-source-runtime/Cargo.toml',
 'cpu-registry-source-runtime/Cargo.lock','registry_ux_catalog_check.py','registry_ux_catalog_check_receipt.json']
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':remote,
 'prior_receipt':{'path':'upstream_ownership_review_receipt.json','sha256':sha(HERE/'upstream_ownership_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,
 'auxiliary_read_sha256':aux,'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,
 'new_findings':new_findings,'authority_entries':entries,'next_read':next_read,'test_receipt':'registry_owner_probe_v2_receipt.json',
 'upstream_fetch_this_pass':'success; no new commits; existing38diff review remains valid, no integration or upstream test rerun',
 'pending_verification':notes['checkpoint']['pending_verification'],'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'git_index_sha256':sha(index),'evidence_sha256':{p:sha(HERE/p) for p in evidence},'retained_isolation':[],
 'shared_generated_cache':{'path':'C:/t/cargo-targets/mere/build','owner':'shared Mere build cache; audit reused/refreshed CPU Registry suppliers; unrelated workspace compilers also active',
 'reason':'Reusable approved shared target, explicit build.build-dir. Test executable/PDB removed; no isolated build output or local target directory. Size not a stable attribution while shared compilers run.'},
 'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'qualification':'Progress checkpoint; full repository remains incomplete. Exact72-file current source read; Registry owner composition89passes1ignored. Six findings include five scoped CPU cases and one source-only UX catalogue discrepancy. Negative consumer census limited to this repository. No production edits, external host/model/GPU/runtime or installer execution.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':new_findings,'authority_entries':entries,
 'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
