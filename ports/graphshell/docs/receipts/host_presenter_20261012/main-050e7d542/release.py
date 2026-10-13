from pathlib import Path
import os,subprocess,shutil,json,gzip,hashlib
repo=Path("/Users/markik/Code/worktrees/mere-nested-graph-view");web=repo/"ports/graphshell/web";out=Path(__file__).parent
compiler=Path("/Users/markik/.rustup/toolchains/1.98.1-x86_64-apple-darwin/bin")
env={**os.environ,"RUSTC":str(compiler/"rustc"),"RUSTDOC":str(compiler/"rustdoc"),"DYLD_LIBRARY_PATH":str(compiler.parent/"lib"),"CARGO_PROFILE_RELEASE_OPT_LEVEL":"s","CARGO_PROFILE_RELEASE_LTO":"thin","CARGO_PROFILE_RELEASE_STRIP":"symbols","CARGO_PROFILE_RELEASE_CODEGEN_UNITS":"1"}
cmd=[str(compiler/"cargo"),"build","--locked","--offline","--release","--target","wasm32-unknown-unknown","--no-default-features","-j1"]
with (out/"viewer-release.log").open("w") as stream:r=subprocess.run(cmd,cwd=web,env=env,stdout=stream,stderr=subprocess.STDOUT)
if r.returncode:raise SystemExit(r.returncode)
raw=out/"viewer-release.wasm";shutil.copy2(web/"target/wasm32-unknown-unknown/release/graphshell_web.wasm",raw)
pkg=out/"release-pkg";subprocess.run(["/Users/markik/.cargo/bin/wasm-bindgen","--target","web","--remove-name-section","--out-name","graphshell_web","--out-dir",str(pkg),str(raw)],check=True)
rows={}
for name,file,cap in [("wasm",pkg/"graphshell_web_bg.wasm",3407872),("glue",pkg/"graphshell_web.js",20480),("mount",web/"viewer.js",8192),("history",web/"fixtures/site-repository-host-history.json",131072)]:
 b=file.read_bytes();gz=gzip.compress(b,compresslevel=9,mtime=0);rows[name]={"raw_bytes":len(b),"gzip9_bytes":len(gz),"cap_bytes":cap,"cap_pass":len(gz)<=cap,"sha256":hashlib.sha256(b).hexdigest()}
report={"source":subprocess.check_output(["git","rev-parse","HEAD"],cwd=repo,text=True).strip(),"final_mere_basis":"631355a2fcbc6d5e88b1b1040d401a0e0b08bfab","upstream_basis":"050e7d54284f22943fd7ffe8297a3b736691bf17","profile":{"opt_level":"s","lto":"thin","strip":"symbols","codegen_units":1},"sizes":rows,"live_site_deployed":False}
(out/"release-size.json").write_text(json.dumps(report,indent=2)+"\n");print(json.dumps(report),flush=True)
