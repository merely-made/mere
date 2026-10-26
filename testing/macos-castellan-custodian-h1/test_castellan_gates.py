# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Missing native/real-vault evidence is never a production integration pass."""
import importlib.util
from pathlib import Path
import unittest

def module(name):
    s=importlib.util.spec_from_file_location('castellan_'+name,Path(__file__).with_name(name+'.py'))
    m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
h=module('run');g=module('test_receiver_setup')

def complete():
    r=g.ReceiverSetup().receiving_result();r['castellanRequired']=True
    r['tests']['authorized_client']['value']['hello']['castellan']={'backend':'castellan-sealed-profile-v1','realConsumersAllowed':False}
    r['tests']['castellan_e2e']={'value':{
        'forgedApproval':{'refused':'invalid-request'},'clientDecision':{'refused':'invalid-request'},
        'firstSignatureVerified':True,'derivedKeyDiffersFromRoot':True,'wrongNamespace':{'refused':'operation-or-scope'},
        'wrongProfile':{'refused':'operation-or-scope'},'initialHello':{'instance':'old'},'restartExit':0,'lockMarkerRemoved':True,
        'lockFaultInjected':True,'lockFaultRepaired':True,'faultLock':{'refused':'vault-storage'},
        'faultRetry':{'refused':'vault-storage'},'faultLockedHello':{'castellan':{'locked':True}},'lock':{'ok':True,'locked':True},
        'lockedHello':{'instance':'new','castellan':{'locked':True}},'lockedSign':{'refused':'locked'},
        'nativeUnlock':{'ok':True,'locked':False},'sameRootAfterNativeUnlock':True,'unlockedHello':{'castellan':{'locked':False}},
        'oldCapabilityAfterUnlock':{'refused':'capability-denied'},'secondSignatureVerified':True,'revoke':{'ok':True},
        'afterRevoke':{'refused':'capability-denied'},'lastSignatureVerified':True,'afterExhaustion':{'refused':'capability-denied'}}}
    r['tests']['copied_custodian']={'signature':{'returncode':0},'events':[{'domain':'CustodianInstallation','fatal':1}]}
    return r

class CastellanGates(unittest.TestCase):
    def test_complete_evidence_has_49_passing_checks(self):
        value=h.evaluate(complete());self.assertTrue(value['all_required_pass']);self.assertEqual(value['total'],49)
    def test_old_cryptokit_backend_cannot_qualify_real_castellan(self):
        r=complete();r['tests']['authorized_client']['value']['hello']['castellan']={}
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_missing_e2e_cannot_qualify(self):
        r=complete();r['tests']['castellan_e2e']={'value':None};self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_echoed_approval_does_not_count_as_refusal(self):
        r=complete();r['tests']['castellan_e2e']['value']['forgedApproval']={'ok':True}
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_locked_restart_must_really_be_locked(self):
        r=complete();r['tests']['castellan_e2e']['value']['lockedHello']['castellan']['locked']=False
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_absent_lock_marker_attack_evidence_cannot_qualify(self):
        r=complete();r['tests']['castellan_e2e']['value']['lockMarkerRemoved']=False
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_native_unlock_failure_cannot_qualify(self):
        r=complete();r['tests']['castellan_e2e']['value']['nativeUnlock']={'refused':'native-presence-unavailable'}
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_false_success_on_lock_io_failure_cannot_qualify(self):
        r=complete();r['tests']['castellan_e2e']['value']['faultLock']={'ok':True}
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_scope_consumption_requires_a_verified_signature(self):
        r=complete();r['tests']['castellan_e2e']['value']['lastSignatureVerified']=False
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_copy_must_have_a_valid_signature_before_startup_denial_counts(self):
        r=complete();r['tests']['copied_custodian']['signature']['returncode']=1
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_missing_copied_binary_is_not_an_installation_guard_pass(self):
        r=complete();r['tests']['copied_custodian']['events']=[{'fatal':2,'domain':'NSPOSIXErrorDomain'}]
        self.assertFalse(h.evaluate(r)['all_required_pass'])

if __name__=='__main__':unittest.main()
