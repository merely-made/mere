// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
const assert=require('node:assert/strict');
const {execFileSync}=require('node:child_process');
module.exports=()=>{const state=execFileSync('/usr/sbin/ioreg',['-n','Root','-d1','-a'],{encoding:'utf8'});assert.ok(!/<key>CGSSessionScreenIsLocked<\/key>\s*<true\/>/.test(state),'Console is locked; headed browser acceptance requires an unlocked session');};
