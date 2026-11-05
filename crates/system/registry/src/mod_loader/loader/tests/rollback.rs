// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn load_all_rolls_back_applied_records_on_activation_failure() {
    let (diag_tx, diag_rx) = diagnostic_channel(IngressLimits::default());
    let _diagnostic_scope = scoped_thread_sender(diag_tx);

    let protocol = test_manifest("mod:protocol", &["ProtocolRegistry"], &[]);
    let failing = test_manifest("mod:failing", &["protocol:test"], &["ProtocolRegistry"]);
    let mut registry = ModRegistry::from_manifests_for_tests(vec![protocol, failing]);

    registry
        .resolve_dependencies()
        .expect("dependencies should resolve");

    let loaded = registry.load_all_with_extensions(
        |manifest, _wasm_source| {
            if manifest.mod_id == "mod:failing" {
                Err(ModActivationError::rollback(
                    "activation failed",
                    vec![ModExtensionRecord::Action {
                        action_id: "action:mod:failing".to_string(),
                    }],
                ))
            } else {
                Ok(vec![ModExtensionRecord::Action {
                    action_id: format!("action:{}", manifest.mod_id),
                }])
            }
        },
        |_record| Ok(()),
    );

    assert_eq!(loaded, vec!["mod:protocol".to_string()]);
    assert_eq!(registry.get_status("mod:failing"), Some(ModStatus::Failed));
    assert_eq!(registry.extension_records_for("mod:failing"), None);
    assert!(
        diag_rx
            .try_drain(usize::MAX)
            .unwrap()
            .packets
            .into_iter()
            .map(|packet| packet.event)
            .any(|event| matches!(
                event,
                DiagnosticEvent::MessageSent { channel_id, .. }
                    if channel_id == CHANNEL_MOD_ROLLBACK_SUCCEEDED
            ))
    );
}

#[test]
fn load_all_quarantines_when_rollback_fails() {
    let (diag_tx, diag_rx) = diagnostic_channel(IngressLimits::default());
    let _diagnostic_scope = scoped_thread_sender(diag_tx);

    let protocol = test_manifest("mod:protocol", &["ProtocolRegistry"], &[]);
    let failing = test_manifest("mod:failing", &["protocol:test"], &["ProtocolRegistry"]);
    let mut registry = ModRegistry::from_manifests_for_tests(vec![protocol, failing]);

    registry
        .resolve_dependencies()
        .expect("dependencies should resolve");

    let loaded = registry.load_all_with_extensions(
        |manifest, _wasm_source| {
            if manifest.mod_id == "mod:failing" {
                Err(ModActivationError::rollback(
                    "activation failed",
                    vec![ModExtensionRecord::Action {
                        action_id: "action:mod:failing".to_string(),
                    }],
                ))
            } else {
                Ok(vec![ModExtensionRecord::Action {
                    action_id: format!("action:{}", manifest.mod_id),
                }])
            }
        },
        |record| match record {
            ModExtensionRecord::Action { action_id } if action_id == "action:mod:failing" => {
                Err("simulated rollback failure".to_string())
            },
            _ => Ok(()),
        },
    );

    assert_eq!(loaded, vec!["mod:protocol".to_string()]);
    assert_eq!(
        registry.get_status("mod:failing"),
        Some(ModStatus::Quarantined)
    );
    assert_eq!(
        registry.extension_records_for("mod:failing"),
        Some(
            &[ModExtensionRecord::Action {
                action_id: "action:mod:failing".to_string(),
            }][..]
        )
    );
    let emitted = diag_rx
        .try_drain(usize::MAX)
        .unwrap()
        .packets
        .into_iter()
        .map(|packet| packet.event)
        .collect::<Vec<_>>();
    assert!(emitted.iter().any(|event| matches!(
        event,
        DiagnosticEvent::MessageSent { channel_id, .. }
            if *channel_id == CHANNEL_MOD_ROLLBACK_FAILED
    )));
    assert!(emitted.iter().any(|event| matches!(
        event,
        DiagnosticEvent::MessageSent { channel_id, .. }
            if *channel_id == CHANNEL_MOD_QUARANTINED
    )));
}

#[test]
fn unload_mod_quarantines_and_preserves_records_on_removal_failure() {
    let (diag_tx, diag_rx) = diagnostic_channel(IngressLimits::default());
    let _diagnostic_scope = scoped_thread_sender(diag_tx);

    let protocol = test_manifest("mod:protocol", &["ProtocolRegistry"], &[]);
    let target = test_manifest("mod:target", &["protocol:test"], &["ProtocolRegistry"]);
    let mut registry = ModRegistry::from_manifests_for_tests(vec![protocol, target]);

    registry
        .resolve_dependencies()
        .expect("dependencies should resolve");
    registry.load_all_with_extensions(
        |manifest, _wasm_source| {
            Ok(vec![ModExtensionRecord::Action {
                action_id: format!("action:{}", manifest.mod_id),
            }])
        },
        |_record| Ok(()),
    );
    let _ = diag_rx
        .try_drain(usize::MAX)
        .unwrap()
        .packets
        .into_iter()
        .map(|packet| packet.event)
        .collect::<Vec<_>>();

    let error = registry
        .unload_mod_with("mod:target", |record| match record {
            ModExtensionRecord::Action { action_id } if action_id == "action:mod:target" => {
                Err("simulated removal failure".to_string())
            },
            _ => Ok(()),
        })
        .expect_err("unload should fail when removal fails");

    assert!(matches!(
        error,
        ModUnloadError::ExtensionRemovalFailed { mod_id, reason }
            if mod_id == "mod:target" && reason == "simulated removal failure"
    ));
    assert_eq!(
        registry.get_status("mod:target"),
        Some(ModStatus::Quarantined)
    );
    assert_eq!(
        registry.extension_records_for("mod:target"),
        Some(
            &[ModExtensionRecord::Action {
                action_id: "action:mod:target".to_string(),
            }][..]
        )
    );
    let emitted = diag_rx
        .try_drain(usize::MAX)
        .unwrap()
        .packets
        .into_iter()
        .map(|packet| packet.event)
        .collect::<Vec<_>>();
    assert!(emitted.iter().any(|event| matches!(
        event,
        DiagnosticEvent::MessageSent { channel_id, .. }
            if *channel_id == CHANNEL_MOD_UNLOAD_FAILED
    )));
    assert!(emitted.iter().any(|event| matches!(
        event,
        DiagnosticEvent::MessageSent { channel_id, .. }
            if *channel_id == CHANNEL_MOD_QUARANTINED
    )));
}

#[test]
fn unload_mod_refuses_while_any_dependent_is_active() {
    let provider = test_manifest("mod:provider", &["capability:test"], &[]);
    let zeta = test_manifest("mod:zeta", &[], &["capability:test"]);
    let alpha = test_manifest("mod:alpha", &[], &["capability:test"]);
    let mut registry = ModRegistry::from_manifests_for_tests(vec![provider, zeta, alpha]);

    registry
        .resolve_dependencies()
        .expect("dependencies should resolve");
    registry.load_all();

    let error = registry
        .unload_mod_with("mod:provider", |_| {
            panic!("a blocked unload removes nothing")
        })
        .expect_err("an active dependent must block its provider's unload");

    assert_eq!(
        error,
        ModUnloadError::DependencyActive {
            mod_id: "mod:provider".to_string(),
            dependent_id: "mod:alpha".to_string(),
        },
        "the singular error names one deterministic blocker",
    );
    assert_eq!(
        registry.get_status("mod:provider"),
        Some(ModStatus::Active),
        "the refused unload leaves the provider active",
    );
}
