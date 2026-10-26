# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Counterexamples: unavailable services and missing entitlements are not security passes."""
import copy
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import subprocess
import unittest

spec=importlib.util.spec_from_file_location('h1_runner',Path(__file__).with_name('run.py'))
h1=importlib.util.module_from_spec(spec);spec.loader.exec_module(h1)
spec=importlib.util.spec_from_file_location('h1_installer',Path(__file__).with_name('install_fixture.py'))
installer=importlib.util.module_from_spec(spec);spec.loader.exec_module(installer)


def success():
    def wrapped(value):return {'value':value}
    hello={'ok':True,'pid':12345,'ppid':1,'key':{'keyStatus':0,'keyBytes':32}}
    probe={'vault':{'opened':False,'errno':1},'symlink':{'opened':False,'errno':1},
           'blob':{'opened':False,'errno':1},'ownFileControl':True,'inheritedFixturePaths':[],
           'inheritedVaultCanary':False,'keyObtained':False,'keyStatus':-34018,
           'selfMemoryControl':{'taskPortObtained':True},'memory':{'taskPortObtained':False},
           'network':{'connected':False,'errno':1},
           'protectedWrites':{k:{'writable':False,'errno':13} for k in ['binary','config','agent']},
           'protectedCreate':{'created':False,'errno':13},
           'selectedInputHash':hashlib.sha256(b'Synthetic selected input for this worker only.').hexdigest()}
    unrelated=copy.deepcopy(probe)
    for n in ['vault','blob','symlink']:unrelated[n]={'opened':True}
    unrelated['network']={'connected':True}
    return {'protectedInstallation':True,'fake_observations':'{"lane":"unsafe-control","secretPresent":true}\n',
      'tests':{'authorized_client':wrapped({'hello':hello,'signatureVerified':True,'publicKey':'fixture-public',
                                           'adjacentScope':{'refused':'operation-or-scope'}}),
               'worker':wrapped(probe),'unrelated_same_user':wrapped(unrelated),
               'unrelated_custody_call':wrapped({'secretTransmitted':False}),
               'fake_server':wrapped({'secretTransmitted':False}),
               'unsafe_fake_control':wrapped({'secretTransmitted':True}),
               'adhoc_custody_call':wrapped({'secretTransmitted':False}),
               'adhoc_same_user':wrapped({'keyObtained':False,'keyStatus':-34018}),
               'same_user_signing_oracle':wrapped({'signerUnattended':False,'timedOut':False,'signerExit':1,'output':'no identity found'}),
               'cold_reopen':wrapped({'signatureVerified':True,'publicKey':'fixture-public','hello':{'instance':'new','ppid':1}}),
               'injection_control':{'returncode':0,'markerCreated':True},
               'hostile_launchd_environment':{'ready':True,'markerCreated':False},
               'direct_child_launch':{'returncode':-9},'injected_child_launch':{'returncode':-9}}}


class EvidenceGates(unittest.TestCase):
    def checks(self,r):return h1.evaluate(r)['checks']
    def test_complete_evidence_can_pass(self):self.assertTrue(h1.evaluate(success())['all_required_pass'])
    def test_empty_results_cannot_pass(self):self.assertFalse(any(self.checks({'tests':{}}).values()))
    def test_missing_owner_entitlement_cannot_prove_key_isolation(self):
        r=success();r['tests']['authorized_client']['value']['hello']['key']={'keyStatus':-34018}
        c=self.checks(r)
        self.assertFalse(c['authorized_key_access']);self.assertFalse(c['worker_key_access_denied']);self.assertFalse(c['unrelated_key_access_denied'])
    def test_missing_worker_not_counted_as_denied(self):
        r=success();r['tests']['worker']={'value':{'transportError':4097}}
        self.assertFalse(self.checks(r)['worker_vault_denied'])
    def test_enoent_is_not_a_sandbox_denial(self):
        r=success();r['tests']['worker']['value']['vault']={'opened':False,'errno':2}
        self.assertFalse(self.checks(r)['worker_vault_denied'])
    def test_fake_server_must_be_capable_of_capture(self):
        r=success();r['fake_observations']=''
        self.assertFalse(self.checks(r)['fake_server_capture_positive_control'])
    def test_pinned_secret_leak_fails_even_with_transport_error(self):
        r=success();r['fake_observations']+='{"lane":"pinned","secretPresent":true}\n'
        self.assertFalse(self.checks(r)['fake_server_rejected_before_secret'])
    def test_launch_timeout_not_a_denial(self):
        r=success();r['tests']['direct_child_launch']={'timeout':True,'returncode':None}
        self.assertFalse(self.checks(r)['direct_child_launch_denied'])
    def test_missing_binary_not_a_launch_constraint_denial(self):
        r=success();r['tests']['direct_child_launch']={'spawn_errno':2,'returncode':None}
        self.assertFalse(self.checks(r)['direct_child_launch_denied'])
    def test_inherited_canary_fails_even_if_path_is_hidden(self):
        r=success();r['tests']['worker']['value']['inheritedVaultCanary']=True
        self.assertFalse(self.checks(r)['worker_no_inherited_fixture_handles'])
    def test_memory_denial_requires_alive_authenticated_target(self):
        r=success();r['tests']['authorized_client']['value']['hello']={'transportError':4097}
        self.assertFalse(self.checks(r)['unrelated_custodian_memory_denied'])
    def test_injection_requires_functioning_library_control(self):
        r=success();r['tests']['injection_control']['markerCreated']=False
        self.assertFalse(self.checks(r)['injection_positive_control'])
    def test_alias_start_failure_cannot_prove_environment_filtering(self):
        r=success();r['tests']['hostile_launchd_environment']['ready']=False
        self.assertFalse(self.checks(r)['hardened_launchd_environment_probe'])
    def test_missing_installed_path_not_a_write_denial(self):
        r=success();r['tests']['unrelated_same_user']['value']['protectedWrites']['binary']={'writable':False,'errno':2}
        self.assertFalse(self.checks(r)['unrelated_protected_binary_write_denied'])
    def test_changed_identity_on_reopen_fails(self):
        r=success();r['tests']['cold_reopen']['value']['publicKey']='another'
        self.assertFalse(self.checks(r)['sealed_cold_reopen'])
    def test_unattended_signer_is_a_security_failure(self):
        r=success();r['tests']['same_user_signing_oracle']['value']={'signerUnattended':True,'signerExit':0}
        r['tests']['attacker_minted_identity']={'value':{'keyObtained':True,'syntheticRootPublicKey':'fixture-public'}}
        self.assertFalse(h1.evaluate(r)['all_required_pass'])
        self.assertTrue(h1.evaluate(r)['signing_authority_finding']['synthetic_root_recovered'])
    def test_arbitrary_signer_failure_is_not_key_isolation(self):
        r=success();r['tests']['same_user_signing_oracle']['value']={'signerUnattended':False,'signerExit':1,'output':'invalid bundle'}
        self.assertFalse(self.checks(r)['signer_authority_not_available_to_same_user_app'])
    def test_signer_timeout_cannot_prove_key_isolation(self):
        r=success();r['tests']['same_user_signing_oracle']['value']={'signerUnattended':False,'signerExit':-9,'timedOut':True}
        self.assertFalse(self.checks(r)['signer_authority_not_available_to_same_user_app'])
    def test_user_writable_installation_fails(self):
        r=success();r['protectedInstallation']=False
        self.assertFalse(h1.evaluate(r)['all_required_pass'])
    def test_installer_refuses_symlink_directory_components(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            root=Path(name);(root/'real').mkdir();(root/'link').symlink_to(root/'real')
            with self.assertRaises(OSError):installer.directory_fd(root/'link')
    def test_installer_refuses_symlink_leaf(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            root=Path(name);(root/'a').write_text('fixture');(root/'link').symlink_to(root/'a')
            fd=installer.directory_fd(root)
            try:
                with self.assertRaises(OSError):installer.source_bytes(fd,'link',__import__('os').getuid())
            finally:__import__('os').close(fd)
    def test_trusted_launcher_refuses_changed_installer_before_execution(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            p=Path(name)/'installer.py';marker=Path(name)/'executed'
            p.write_text('from pathlib import Path;Path('+repr(str(marker))+').write_text("ran")')
            r=subprocess.run(['/usr/bin/python3','-I','-c',h1.INSTALL_VERIFY_LAUNCHER,str(p),'0'*64],capture_output=True)
            self.assertNotEqual(r.returncode,0);self.assertFalse(marker.exists())
    def test_trusted_launcher_executes_exact_reviewed_program(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            p=Path(name)/'installer.py';marker=Path(name)/'executed'
            p.write_text('from pathlib import Path;Path('+repr(str(marker))+').write_text("ran")')
            expected=hashlib.sha256(p.read_bytes()).hexdigest()
            r=subprocess.run(['/usr/bin/python3','-I','-c',h1.INSTALL_VERIFY_LAUNCHER,str(p),expected],capture_output=True)
            self.assertEqual(r.returncode,0);self.assertTrue(marker.exists())
    def test_installer_refuses_path_escape(self):
        with tempfile.TemporaryDirectory(dir='/private/tmp') as name:
            fd=installer.directory_fd(Path(name))
            try:
                with self.assertRaises(RuntimeError):installer.source_bytes(fd,'../secret',__import__('os').getuid())
            finally:__import__('os').close(fd)

if __name__=='__main__':unittest.main()
