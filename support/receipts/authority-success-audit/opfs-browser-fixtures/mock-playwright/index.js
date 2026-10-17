const fs=require('node:fs');const input=JSON.parse(fs.readFileSync(process.env.AUDIT_CASE_FILE,'utf8'));
const receipt=input.receipt;
const page={on(){},async goto(){},async waitForFunction(){},async exposeFunction(){},async evaluate(fn,arg){
 const code=String(fn);
 if(code.includes('const channel = new BroadcastChannel'))return;
 if(code.includes('await window.munimentOpfsProbe.runLane')){if(input.laneThrows)throw Error('AUDIT injected lane exception');return}
 if(code.includes('return { ok: l?.ok')){const l=receipt.lanes[arg];return {ok:l?.ok??null,failed:l?.failed??false,error:l?.error??null}}
 if(code.includes('window.munimentOpfsProbe.receipt()'))return receipt;
 if(code.includes('window.munimentOpfsProbe.exportedBase64()'))return Buffer.from('AUDIT inert non-database bytes').toString('base64');
 throw Error('unexpected fake-page boundary: '+code);
}};
module.exports={firefox:{async launch(){return {async newContext(){return {async newPage(){return page}}},async close(){}}}}};
