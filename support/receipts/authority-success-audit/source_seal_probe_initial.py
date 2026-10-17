"""Exercise actual source-seal CLI on audit-owned fixtures; no build is performed."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,shutil,subprocess,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source='design_docs/mere_docs/testing/receipts/2026-10-11_classic_browser_gates/identity.py'
compiled=['crates/cambium/cambium/src/workspace_chrome.rs','crates/cambium/cambium/src/instrument.css']
guards={p:sha(ROOT/p) for p in [source,*compiled]}
base=HERE/'source-seal-fixture';assert not base.exists();base.mkdir()
runner=base/'identity.py';shutil.copyfile(ROOT/source,runner)
fixture=base/'repo';fixture.mkdir();(fixture/'Cargo.toml').write_text('[package]\nname="audit-inert-fixture"\nversion="0.0.0"\n',encoding='utf-8')
for name in compiled:
    target=fixture/name;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,target)
bootstrap='''import json,runpy,sys
from pathlib import Path
from unittest.mock import patch
target=Path(sys.argv[1]);root=Path(sys.argv[2]);action=sys.argv[3];output=Path(sys.argv[4]);files=json.loads(Path(sys.argv[5]).read_text())
def git_output(command,*args,**kwargs):
    if "ls-files" in command:return ("\\0".join(files)+"\\0").encode()
    if "rev-parse" in command:return b"AUDIT_INERT_SOURCE_FIXTURE_NO_REAL_GIT_COMMIT\\n"
    raise AssertionError(command)
sys.argv=[str(target),action,"--repo",str(root),"--output",str(output),*sys.argv[6:]]
with patch("subprocess.check_output",side_effect=git_output):runpy.run_path(str(target),run_name="__main__")
'''
(base/'bootstrap.py').write_text(bootstrap,encoding='utf-8')
listing=['Cargo.toml',*compiled];(base/'listing.json').write_text(json.dumps(listing),encoding='utf-8')
package=base/'pkg';package.mkdir()
(package/'graphshell_web.js').write_text('// AUDIT inert artifact; no compiled product\n',encoding='utf-8')
(package/'graphshell_web_bg.wasm').write_bytes(b'AUDIT inert non-Wasm bytes')
(base/'metadata.json').write_text(json.dumps({'packages':[{'name':'audit-inert-fixture','version':'0.0.0',
 'source':None,'manifest_path':str(fixture/'Cargo.toml')}]}),encoding='utf-8')
(base/'build.log').write_text('AUDIT supplied build-log fixture: Finished (no actual build)\n',encoding='utf-8')
env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
def run(action,name):
    output=base/(name+'.json');args=[sys.executable,'-B',str(base/'bootstrap.py'),str(runner),str(fixture),action,str(output),str(base/'listing.json')]
    if action=='seal':args+=['--preflight',str(base/'preflight.json'),'--package',str(package),
        '--metadata',str(base/'metadata.json'),'--build-log',str(base/'build.log')]
    result=subprocess.run(args,cwd=ROOT,env=env,capture_output=True,text=True,timeout=20)
    (base/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
    return result,json.loads(output.read_text()) if output.exists() else None
pre,before=run('preflight','preflight');assert pre.returncode==0
assert set(before['sources'][0]['source_sha256'])=={'Cargo.toml',compiled[0]}
css=fixture/compiled[1];css_before=sha(css)
with css.open('ab') as f:f.write(b'\n/* AUDIT changed compiled CSS input after preflight */\n')
assert sha(css)!=css_before
sealed,record=run('seal','css-changed-seal');assert sealed.returncode==0 and record['sealed'] is True
assert record['sources'][0]['fingerprint']==before['sources'][0]['fingerprint']
assert compiled[1] not in record['sources'][0]['source_sha256']
rust=fixture/compiled[0]
with rust.open('ab') as f:f.write(b'\n// AUDIT tracked Rust control change\n')
refused,refused_record=run('seal','rust-changed-control');assert refused.returncode!=0 and refused_record is None
assert 'source changed between preflight and package seal' in refused.stderr
assert all(sha(ROOT/p)==h for p,h in guards.items())
result={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':guards,
 'cases':{'css_input':compiled[1],'css_before_sha256':css_before,'css_after_sha256':sha(css),
 'css_changed_outer_exit':sealed.returncode,'css_changed_sealed':record['sealed'],'fingerprint_unchanged':True,
 'rust_change_refused_exit':refused.returncode},
 'runner_sha256':sha(Path(__file__)),'fixture_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in base.rglob('*') if p.is_file()},
 'qualification':'Whole unchanged identity.py CLI copy executes in real Python processes. Git listing/head are explicitly mocked for an audit-owned inert minimal source fixture. Current real Cambium workspace_chrome.rs and instrument.css copied byte-identically before audit-only edits. CSS bytes actually change between preflight/seal; hash list excludes them and seal accepts. Changed Rust control refuses. Package/log/metadata inputs explicitly synthetic and no genuine build/compiler/Genet/browser/Wasm/runtime/GPU executed. This proves source snapshot omission, not a freshly observed stale real browser package or invalidation of historical success.'}
with (HERE/'source_seal_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps({'cases':result['cases'],'fixture_bytes':sum(p.stat().st_size for p in base.rglob('*') if p.is_file()),'qualification':result['qualification']},indent=2))
