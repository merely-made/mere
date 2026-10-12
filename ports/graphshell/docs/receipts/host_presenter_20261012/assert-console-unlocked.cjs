const assert=require('node:assert/strict');
const {execFileSync}=require('node:child_process');
module.exports=()=>{const state=execFileSync('/usr/sbin/ioreg',['-n','Root','-d1','-a'],{encoding:'utf8'});assert.ok(!/<key>CGSSessionScreenIsLocked<\/key>\s*<true\/>/.test(state),'Console is locked; headed browser acceptance requires an unlocked session');};
