import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../..');
const dest=path.join(here,'resource_chunk_integrity_receipt.json');assert(!fs.existsSync(dest));
const source='ports/graphshell/web/extension/resource-chunk.js';const bytes=fs.readFileSync(path.join(root,source));
const context=vm.createContext({atob,Uint8Array});vm.runInContext(bytes.toString('utf8'),context,{filename:source});
const fixture=JSON.parse(fs.readFileSync(path.join(here,'resource_chunk_fixture.json'),'utf8'));
const Assembly=context.GraphshellResourceChunk.ResourceAssembly;
const output=[];
for(const [name,expected]of[['valid',fixture.original_utf8],['bad_chunk',fixture.changed_utf8],['bad_resource',fixture.changed_utf8]]){
 const frame=fixture.frames[name];const assembly=new Assembly(frame.session,frame.resource);
 const result=assembly.accept(frame);assert(result instanceof Uint8Array);
 assert.equal(new TextDecoder().decode(result),expected);output.push({frame:name,accepted:true,result_utf8:expected});
}
const frame=fixture.frames.valid;
assert.throws(()=>new Assembly(frame.session,frame.resource).accept({...frame,session:'native:other'}),/another session/);
const sha=raw=>crypto.createHash('sha256').update(raw).digest('hex');
fs.writeFileSync(dest,JSON.stringify({date_utc:new Date().toISOString(),node:process.version,exit:0,checks:4,results:output,
 source,source_sha256:sha(bytes),fixture_sha256:sha(fs.readFileSync(path.join(here,'resource_chunk_fixture.json'))),
 scope:'Complete actual JavaScript ResourceAssembly executed in native Node VM with exact synthetic BLAKE3 fixture frames. A chunk with changed bytes and unchanged chunk address is accepted; a validly chunk-addressed changed payload under the wrong whole-resource address is also accepted. Valid frame and wrong-session rejection controls. No real native host, browser, remote peer, user resource, renderer or network was exercised. Shared Rust assembler is qualified separately.'},null,2)+'\n');
console.log(JSON.stringify({receipt:path.basename(dest),checks:4,exit:0}));
