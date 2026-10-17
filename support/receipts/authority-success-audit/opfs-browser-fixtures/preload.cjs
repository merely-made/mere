const cp=require('node:child_process'),fs=require('node:fs');
const input=JSON.parse(fs.readFileSync(process.env.AUDIT_CASE_FILE,'utf8'));
cp.spawnSync=(...args)=>{if(!input.native)throw Error('unexpected native invocation');return {status:input.native.exit,stdout:JSON.stringify({ok:input.native.ok}),stderr:'AUDIT injected native verifier result'}};
require('node:module').syncBuiltinESMExports();
