# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Receiving-account counterexamples: the wrong context cannot qualify H1."""
import copy
import importlib.util
import json
import os
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

def module(name):
    spec=importlib.util.spec_from_file_location('test_h1_'+name,Path(__file__).with_name(name+'.py'))
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result

b=module('bootstrap_receiver');h=module('run')
g=module('test_evidence_gates')


def manifest():
    return {'receiver':{'user':'djinnh1','home':'/Users/djinnh1','builder_uid':501,
        'signer_positive_control':{'signing':{'returncode':0},'verification':{'returncode':0},
                                   'unsigned_binary_sha256':'same'}},
        'bundle_sha256':{'AttackerMintedCustodian.app/Contents/MacOS/Fixture':'same'}}


class ReceiverSetup(unittest.TestCase):
    def context(self,m=None,*,uid=502,user='djinnh1',admin=False,gui=True,home='/Users/djinnh1',sudo=False):
        account=SimpleNamespace(pw_name=user,pw_dir=home)
        member=SimpleNamespace(returncode=0,stdout='user '+user+' is '+('a member' if admin else 'not a member')+' of the group admin')
        session=SimpleNamespace(returncode=0 if gui else 113)
        env={'HOME':home}
        if sudo:env['SUDO_USER']='markik'
        with patch.object(b.os,'getuid',return_value=uid),patch.object(b.pwd,'getpwuid',return_value=account),patch.object(b.subprocess,'run',side_effect=[member,session]),patch.dict(os.environ,env,clear=True):
            return b.receiving_context(m or manifest())
    def test_dedicated_nonadmin_gui_receiver_can_pass_context(self):self.assertTrue(self.context()['valid'])
    def test_builder_account_cannot_qualify(self):self.assertFalse(self.context(uid=501)['valid'])
    def test_admin_receiver_cannot_qualify(self):self.assertFalse(self.context(admin=True)['valid'])
    def test_no_gui_session_cannot_qualify(self):self.assertFalse(self.context(gui=False)['valid'])
    def test_sudo_user_is_not_a_gui_receiving_run(self):self.assertFalse(self.context(sudo=True)['valid'])
    def test_wrong_account_name_cannot_qualify(self):self.assertFalse(self.context(user='someoneelse')['valid'])
    def test_different_home_cannot_qualify(self):self.assertFalse(self.context(home='/tmp/djinnh1')['valid'])
    def test_signer_control_for_another_binary_cannot_qualify(self):
        m=manifest();m['receiver']['signer_positive_control']['unsigned_binary_sha256']='different'
        self.assertFalse(self.context(m)['valid'])
    def test_signer_positive_control_failure_cannot_qualify(self):
        m=manifest();m['receiver']['signer_positive_control']['signing']['returncode']=1
        self.assertFalse(self.context(m)['valid'])
    def test_user_owned_package_is_rejected(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            with self.assertRaises(RuntimeError):b.protected_path(Path(name),directory=True)
    def test_real_root_owned_system_artifact_passes_path_check(self):
        b.protected_path(Path('/System/Library/CoreServices/SystemVersion.plist'))
    def test_artifact_path_cannot_escape(self):
        for relative in ['../private.key','/Users/markik/key','a/../../b']:
            with self.assertRaises(RuntimeError):b.artifact_parts(relative)
    def receiving_result(self):
        r=g.success();r['separateAccountRequired']=True
        r['receivingContext']={'valid':True,'signerPositiveControl':True}
        r['tests']['unrelated_same_user']['value']['signerKeychainFile']={'opened':False,'errno':13}
        return r
    def test_complete_receiving_evidence_can_pass(self):self.assertTrue(h.evaluate(self.receiving_result())['all_required_pass'])
    def test_missing_receiving_context_cannot_pass(self):
        r=self.receiving_result();r['receivingContext']={}
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_signer_keychain_file_missing_is_not_denial(self):
        r=self.receiving_result();r['tests']['unrelated_same_user']['value']['signerKeychainFile']['errno']=2
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_signer_keychain_file_accessible_fails(self):
        r=self.receiving_result();r['tests']['unrelated_same_user']['value']['signerKeychainFile']={'opened':True}
        self.assertFalse(h.evaluate(r)['all_required_pass'])
    def test_wrong_manifest_pin_prevents_runtime_before_any_native_probe(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            p=Path(name);(p/'manifest.json').write_text(json.dumps({'receiver':{'user':'djinnh1'}}))
            with self.assertRaisesRegex(RuntimeError,'manifest digest'):
                h.run(SimpleNamespace(fixture=name,manifest_sha256='0'*64,command='run-protected'))
    def test_isolated_runner_can_load_its_exact_adjacent_helper(self):
        self.assertTrue(callable(h.local_module('bootstrap_receiver').receiving_context))

if __name__=='__main__':unittest.main()
