"""Finish Tabard source/owner review, preserving the previous report prefix."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='tabard_ownership_review_receipt.json'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8-sig'))
def git(*args):return subprocess.check_output(['git',*args],cwd=ROOT,text=True).strip()
assert not (HERE/NAME).exists()
old=load('registry_ownership_review_receipt.json');head=git('rev-parse','HEAD');upstream=git('rev-parse','origin/main')
assert head==old['basis_commit'] and upstream==old['upstream_commit']
guards={**old['preserved_production_and_test_hashes'],**old['source_sha256']};aux=dict(old['auxiliary_read_sha256'])
runtime=old['runtime_input_sha256'];shared=old['shared_working_file_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**aux,**runtime}.items())
assert all(sha(HERE/p)==h for p,h in old['evidence_sha256'].items())
doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
index=Path(git('rev-parse','--git-path','index'))
if not index.is_absolute():index=ROOT/index
assert sha(index)==old['git_index_sha256']
inventory=load('source_inventory.json')['files'];coverage=load('read_coverage.json')
paths=[p for p in inventory if p.startswith('crates/system/tabard/')];assert len(paths)==28
sources={p:sha(ROOT/p) for p in paths}
for p,h in sources.items():
    assert h==inventory[p]['sha256']==coverage[p]['sha256']
    assert coverage[p]['through']==inventory[p]['chars']==len((ROOT/p).read_bytes().decode('utf-8-sig',errors='replace'))
assert sum(inventory[p]['chars'] for p in paths)==281469 and load('pending_read.json')==[]
fixture='crates/system/tabard/tests/fixtures/lagrange_palette.txt';aux[fixture]=sha(ROOT/fixture)
probe=load('tabard_owner_probe_receipt.json')
assert probe['exit']==0 and probe['passed']==81 and probe['original_tests']==79
assert len(probe['removed_outputs'])==18 and all(not Path(p['path']).exists() for p in probe['removed_outputs'])
assert all(sha(ROOT/p)==h for p,h in probe['source_sha256'].items())
assert all(sha(HERE/p)==h for p,h in probe['evidence_sha256'].items())
notes=load('review_notes.json');authority=load('authority_map.json')
assert notes['findings'][-1]['id']=='Q391' and authority['entries'][-1]['id']=='A1169'
new=[
 {'id':'Q392','title':'Raw theme registry can replace and remove an allegedly immutable built-in',
  'source':'crates/system/tabard/src/theme/registry.rs:300-321',
  'evidence':'add_user_theme accepts an existing theme:light identity, forces its source toUser and overwrites its definition. A subsequent rename and remove now succeed, and theme:light resolves to fallback. The same colliding definition is refused byThemeDraft::from_theme; fresh-ID registry.fork control leaves the originalbuilt-in unchanged.',
  'scope':'Actual unchanged Tabard public APIs over an in-memory registry and current Tinct. No user settings/library/UI changed. Current Graphshell appearance intake checks built-in collision before portable validation/add, while Pelt ThemeCatalog and Workshop load through ThemeLibraryStore collision validation. The lower API violates its documented immutable-built-in contract; this is not a demonstrated bypass of those guarded product paths.',
  'repair_triage':{'disposition':'documented in read-only full-source audit','reason':'Bounded central admission guard for existingBuiltIn identities while retaining user-ID replacement and fresh-ID forks. Keep inert portable built-in definitions exportable; do not replace the current host/library checks with weaker registry admission.'}},
 {'id':'Q393','title':'Application font preferences and authored typography disagree on generic family case',
  'source':'crates/system/tabard/src/typography.rs:65-77,130-147; src/theme_styles.rs:63-79',
  'evidence':'FontPreferences reading[SERIF] with no installednamed face and sans-serif hostdefault resolves to sans-serif with used_default=true and emits sans-serif CSS. Lowercase serif resolves/emits serif. ThemeFontStyle with the identical uppercaseSERIF request validates and emits canonicalserif because its helper normalizes before generic recognition. Requested spelling remains preserved in allcases.',
  'scope':'Actual shared public FontPreferences/ThemeFontStyle APIs and serialized input, with lowercase control. Pelt ThemeCatalog::application_font_css/notices calls this resolver, but no live Pelt setting, font catalogue, engine face realization, layout or screen rendering was run. This is inconsistent shared preference lowering, not a measured text appearance/metrics defect.',
  'repair_triage':{'disposition':'documented in read-only full-source audit','reason':'Bounded shared case-insensitive generic recognition with canonical CSS output; retain authored request spelling and named-family fallback/availability semantics. Avoid maintaining separate normalization rules for application and theme preferences.'}}]
for finding in new:
    finding.update({'status':'reproduced public CPU API behavior; documented, not repaired','verification':NAME,'diagnostics':['tabard_owner_probe_receipt.json']})
    notes['findings'].append(finding)
rows=[
 ('Shared appearance resolution now distinguishes source from projection','One selected Theme/Mode becomes either Derived ModePalette or AuthoredStylesheet, preserving requestedchoice and explicit fallback diagnostics.',
  ['crates/system/tabard/src/appearance.rs','crates/system/tabard/src/artifact.rs','crates/system/tabard/tests/choice_resolution.rs'],
  'Current strict mode artifacts share harmony/Tinct explicit-mode math, refuse authoredCSS/custommode derivation and retain full source provenance. Existing controls pass. Authored CSS retains final cascade authority; the shared resolver never claims arbitraryrules were evaluated as seedcolors. Graphshell/Pelt/Workshop current consumer calls are source-traced, not headed rerun.'),
 ('Legacy artifacts and host tokens coexist with the new appearance authority','No-argument exports intentionally keep raw authored seeds/normalcontrast; strict exports and old ThemeTokenSet have different role/mode semantics.',
  ['crates/system/tabard/src/lib.rs','crates/system/tabard/src/theme/seed.rs','crates/system/tabard/src/theme/registry.rs','crates/system/tabard/tests/mode_artifacts.rs'],
  'The strict versus legacy behavior is documented and tested, not a newly hidden export bug. Old mode_tokens/derive_from_def_for_mode still derive canonicaltokens regardless of authoredsheet and Custom falls back todark; hosts must select authoritative presentation first. Edge/chrome convenience palettes and token projections remain alongside base/syntax artifact roles. Singleauthoring-path prose is narrower than allsurviving publicpaths.'),
 ('Built-in immutability is enforced above a permissive registry','Workshop and library/intake admission check immutable identities while raw add_user_theme does not.',
  ['crates/system/tabard/src/theme/registry.rs','crates/system/tabard/src/library.rs','crates/system/tabard/src/workshop.rs','ports/graphshell/src/appearance.rs','ports/pelt/desktop/appearance.rs'],
  'Q392 demonstrates replacement/removal through raw public API. Current Graphshell checks built-in/source collision; Pelt/Workshop library load validates before add. Those are real mitigations and existing library/workshop controls pass. Move shared admission ownership carefully; do not claim a current host import exploit. Raw registry also omits some definition checks performed in portable/library paths, pending separate cases.'),
 ('Authored libraries and artifact exports share one real persistence writer','atomic_write coordinates cooperating writers, compares snapshots and publishes completefiles with explicit CreateNew/Replace.',
  ['crates/system/tabard/src/library.rs','crates/system/tabard/src/portable.rs','crates/system/tabard/tests/library.rs','crates/system/tabard/tests/portable.rs'],
  'Original current Windows CPU tests prove stale/busy/failed save refusal, preserved disk/memory, retry and complete artifact publication. Library anchors relativepath. Noncooperating external editor race remains explicitly qualified. Unix-only permission/symlink test did not execute; no crash/powercut or directory-fsync durability proof. Validation/persistence does not activate themechoice.'),
 ('Choice storage remains a separate implementation with weaker concurrency semantics','FileThemeChoiceStore uses its own temporary-file/renamewriter while library/export use snapshotchecked locking.',
  ['crates/system/tabard/src/theme/choice.rs','crates/system/tabard/src/library.rs'],
  'Strictload preserves malformed/UTF8 errors, legacyload deliberately defaults, and failedwrite preserves livechoice in passing originals. Choice load retains relativepath, temp uses File::create and there is no library-style stale/cooperating-writer guard. Potential cwd/temp/concurrentwriter cases remain sourcefollowups, not proven user-data loss. Different durable objects do not require a shared stateauthority, but shared safe file primitives may prevent drift.'),
 ('Application font intent and authored typography have explicit ownership but duplicated generic rules','ThemeChoice fonts and Theme.typography remain separate: application requests versus portable authoredstyle.',
  ['crates/system/tabard/src/typography.rs','crates/system/tabard/src/theme_styles.rs','crates/system/tabard/src/theme/choice.rs','ports/pelt/desktop/appearance.rs'],
  'Q393 proves uppercasegeneric divergence across twohelpers. Typed FontFamily validates/deserializes strings and quotes CSS; size/leading checks and version2envelopes preserve newer data. Host discovery/face realization/metrics remain product/engine-owned. Missingfamily diagnostics are read-only; generic acceptance is not actual glyphcoverage. No native font/render acceptance rerun.'),
 ('Custom calculators are separate from attached stylesheet resolution','mode_calc declares completeChrome role tables; appearance recognizes attached sheets, not a registeredcalculator store.',
  ['crates/system/tabard/src/theme/mode_calc.rs','crates/system/tabard/src/appearance.rs','crates/system/tabard/src/theme/registry.rs'],
  'Current original calculator role completeness/unknownrole controls pass. CustomModeDef dark/hc metadata and evaluator are a hostinjection path; Mode::Custom in older token derivation is canonicaldarkfallback, while strictartifacts refuse and appearance fallsback withdiagnostic when no sheetexists. Trace actual hostcustommode wiring before treating allthese APIs as onecustommode authority. Nonfinite RoleSpec admission remains sourcefollowup.'),
 ('Smolweb palette sharing is actual reuse with a bounded host parser caveat','One Site/fixed/App/System palette module replaced donor-local palettes; hosts choose the scheme and render.',
  ['crates/system/tabard/src/smolweb.rs','crates/system/tabard/src/lib.rs','crates/system/tabard/tests/theme.rs'],
  'Original simplehost/path and App/System controls pass. Site hue parser splits rawauthority text, not a URLhost parser; case/query/userinfo/port equivalence requires followup. Lagrange projection honestly reports collapsed/unrepresentedroles, alpha and pinnedstockloader omissions. Its golden input was read as auxiliarydata; full Genet/Livery integration tests and externalLagrange were not run.')]
ids=[]
for number,(domain,claim,implementations,assessment) in enumerate(rows,1170):
    ids.append(f'A{number}');authority['entries'].append({'id':ids[-1],'domain':domain,'claim':claim,'implementations':implementations,
      'assessment':assessment,'state':'complete current source read; exact CPU scope recorded separately; full repository audit ongoing','verification':NAME})
for area,detail in [
 ('Tabard registry admission migration','Q392 raw registry and guarded Workshop/Library/Graphshell disagree about immutableID admission. Keep guards until central behavior is corrected; directblank identity/nonfinite-harmony cases need separate probes.'),
 ('Tabard legacy authority convergence','Keep permissive legacy rawseed exports, older ThemeTokenSet canonicalfallback and strict authoredCSS-aware appearance distinct. Current consumers share resolve_theme_choice, but this does not retire every legacyentry point.'),
 ('Tabard choice writer and custommode scope','Review relativechoice paths, temporarycreation/stalewriter behavior and calculatorfinite inputs/currenthost adoption as documented sourcefollowups. No concurrency/crash/custommode host test was executed.')]:
    notes['review_followups'].append({'area':area,'detail':detail,'verification':NAME})
stats=json.loads(subprocess.check_output(['python',str(HERE/'read_coverage.py'),'status'],cwd=ROOT))
assert stats['complete_files']==2749 and not stats['changed_since_inventory'] and stats['pending_chunks']==0
notes['completed_source_areas'].append({'prefix':'crates/system/tabard/','files':28,'chars':281469,'verification':NAME,'basis_commit':head,
 'qualification':'All inventoried source/manifests/tests read. Actual current library and79originalCPUtests plus2auditcases execute. Original Genet/Livery integration file read but not executed.'})
next_read={'path':'design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/qualify_final_graphs.py','start':0,
 'reason':'All current crates/ inventory source is read; continue remaining executable design/receipt source, port source, scripts and supplier patches without narrowing the denominator.'}
notes['current_area']='Tabard source and public owner verification complete; all crates/ inventory source read'
notes['next_area']=next_read['reason']
notes['checkpoint'].update({'classification':'progress','coverage':stats,'finding_count':len(notes['findings']),'authority_entries':len(authority['entries']),
 'next_read':next_read,'pending_verification':'No audit-owned process live. Shared unrelated compilers preserved.100newsourcefiles read across thisturn;170CPUtests passed total with1Registrytestignored, including7auditcases; UX catalogue source-only. Localmain7c718105, fetchedorigin00e44428 unchanged and8commits not integrated due sharedreport overlap. Full audit remains incomplete: design/receipt executable source, remainingports/scripts/vendorpatches remain.'})
notes['checkpoint']['new_receipts'].extend([NAME,'tabard_owner_probe_receipt.json'])
assert all(coverage[p].get('sha256')==v['sha256'] and coverage[p].get('through')==v['chars'] for p,v in inventory.items() if p.startswith('crates/'))
addition=f'''

### Tabard authority and persistence review, 2026-10-17

All28Tabard source/manifest/test files (281,469characters) are now read at current hashes, plus its Lagrangegolden fixture as auxiliarydata. Together with Registry/session/surface/UX thisturn added100completefiles. Coverage is{stats['complete_files']:,}/{stats['total_files']:,}files and{stats['read_chars']:,}/{stats['total_chars']:,}characters. Every sourcefile under the current crates/ inventory isread; this is not full repository completion. Executable design/receipt source, remainingports/scripts and supplierpatches still remain in thedenominator. No pending/stale readcredit. Localmain remains{head}; fetchedorigin{upstream} has no furthercommits and the previouslyreviewed8commit/38diff advance is notinstalled because of the sharedreport overlap.

Q392: rawThemeRegistry::add_user_theme can replace theme:light, force provenanceUser, and thereby allow rename/removal of a supposedlyimmutable built-in. Public actualAPI test reproduces it with a fresh-IDfork control. Workshop rejects the collision. CurrentGraphshell intake independently checks built-in/sourcecollision, while Pelt/Workshop load through the stricterThemeLibraryStore. Those are real guards; nohostimport bypass or changedusersettings is claimed. Q393: applicationFontPreferences request SERIF resolves/emits sans-serif fallback with an empty installedcatalogue, while lowercaseserif and authoredThemeFontStyle SERIF retain genericserif. Requested spelling ispreserved; the genericrecognition helpers disagree. Pelt fontCSS/notices call the affectedresolver, but noengine face/layout or headed appearance wasexecuted.

Actualcurrent Tabardroot and localTinct compiled in a standaloneCPU dependencygraph.81tests passed:37existingunit tests,42existing tests from7selectedpureCPUintegrationfiles and2public auditcases. This includes real temporaryfile load/save, busy/stale writerrefusal, failurepreservation, retry, portableversioning, authoredCSS versus derivedpalette andundo/conflict controls. The originaltheme.rs Genet/Livery integrationfile was read but notrun; neitherUnix-only permission/symlink test nor externalLagrange/runtime/font/GPU acceptance isclaimed. All18exact testexe/PDB outputs named by the completedrun were markerchecked andremoved; shareddependency cache remains in theapproved stabletarget. Noisolatedtarget/home/worktree andno productionedits.

A1170-A1177 record both actualconsolidation andremainingmigration. Sharedresolve_theme_choice preserves requestedidentity, explicitfallback diagnostics and authoredCSS authority; strictmode artifacts shareTinct/harmony math and refuse pretending sheets/customcalculators arederivedpalettes. Older noargumentexports intentionally retain rawseed normalcontrast behavior, and olderThemeTokenSet/mode_tokens APIs remain alongside them, withcustommode canonicalfallback. This is documented compatibility, not a hiddennewexport failure. Libraryandartifact exports share oneactual lock/snapshot/atomicpublication writer and theirCPUcontrols pass; activechoice has a separate simplerwriter, so relativepath/temp/stale/concurrentwriter behavior stays a documentedfollowup. Applicationfontrequests andauthoredstyles own differentdata, but genericnormalization should be shared. Customcalculators versus attachedsheets still requirehostadoption tracing. Smolweb shares palettesbut rawauthority hueparsing needs equivalencecases. Neithercontrastmetadata nor a passingselectedCPU suite meansallhostroles or rendering arequalified.

Evidence: [Tabard checkpoint](../../../support/receipts/authority-success-audit/{NAME}) and [actual library CPUtests](../../../support/receipts/authority-success-audit/tabard_owner_probe_receipt.json). Across RegistryandTabard thisturn170tests passed,163original plus7audit, with1existingRegistryinventorytestignored; Q391remains source-only. FirstRegistryharnesscompilerfailure remains preserved. Previousreport bytes, guardedsource, sharedWIP andindex areunchanged. Fullrepository auditcontinues with the remaining sourceinventory.
'''
doc.write_bytes(prefix+addition.encode('utf-8'))
for n,v in [('review_notes.json',notes),('authority_map.json',authority)]: (HERE/n).write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8')
assert subprocess.run(['git','diff','--check'],cwd=ROOT,capture_output=True).returncode==0
evidence=['record_tabard_ownership_review.py','tabard_owner_probe_receipt.json',*probe['evidence_sha256']]
budget=sum(p.stat().st_size for p in HERE.rglob('*') if p.is_file());assert budget<100_000_000
record={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':head,'upstream_commit':upstream,
 'prior_receipt':{'path':'registry_ownership_review_receipt.json','sha256':sha(HERE/'registry_ownership_review_receipt.json')},
 'coverage':stats,'source_sha256':sources,'preserved_production_and_test_hashes':guards,'auxiliary_read_sha256':aux,
 'runtime_input_sha256':runtime,'shared_working_file_sha256':shared,'new_findings':['Q392','Q393'],'authority_entries':ids,
 'all_current_crates_inventory_source_read':True,'full_repository_source_read':False,'next_read':next_read,
 'pending_verification':notes['checkpoint']['pending_verification'],'audit_authored_production_edits_this_pass':False,
 'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()},
 'evidence_sha256':{p:sha(HERE/p) for p in evidence},'git_index_sha256':sha(index),'retained_isolation':[],
 'shared_generated_cache':old['shared_generated_cache'],'receipt_budget':{'cap_bytes':100_000_000,'retained_bytes_before_receipt':budget},
 'qualification':'Progress, not full repository completion.28Tabardsourcefiles read,81actualCPUtests passed(79original2audit). Currenthostguard mitigations source-traced. Allcrates/ inventory source read; remainingports/scripts/vendorpatch/evidence source remain. No fullGenet/Livery/host/browser/font/GPU/external acceptance or production edits.'}
with (HERE/NAME).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix) and sha(index)==old['git_index_sha256']
assert all(sha(ROOT/p)==h for p,h in {**guards,**sources,**aux,**runtime}.items())
assert all(sha(ROOT/p)==h for p,h in shared.items() if p!=old['document']['path'])
print(json.dumps({'receipt':NAME,'coverage':stats,'findings':['Q392','Q393'],'authority_entries':ids,
 'all_crates_source_read':True,'full_repository_complete':False,'budget_bytes':budget,'retained_isolation':[],'next_read':next_read},indent=2))
