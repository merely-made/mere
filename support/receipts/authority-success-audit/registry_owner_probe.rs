//! Audit composition: whole unchanged Registry modules, actual command-menu,
//! and an exact data-only ViewerSubsystemCapabilities source projection.
//! No actual viewer, native mod, WASM runtime or product host is executed.
#![allow(unexpected_cfgs, unused_imports, dead_code)]
#[path = "../../../crates/system/registry/src/diagnostics.rs"]
pub mod diagnostics;
#[path = "../../../crates/system/registry/src/layout.rs"]
pub mod layout;
#[path = "../../../crates/system/registry/src/input.rs"]
pub mod input;
#[path = "../../../crates/system/registry/src/mod_loader.rs"]
pub mod mod_loader;
pub mod viewer {
    use crate::layout::CapabilityDeclaration;
    include!("registry_viewer_capabilities_projection.rs");
}

#[cfg(test)]
mod audit {
    use super::*;
    use diagnostics::*;
    use mod_loader::*;

    #[test]
    fn descriptor_sampling_is_not_the_effective_default() {
        let id = diagnostics::channels::CHANNEL_COMPOSITOR_TILE_ACTIVITY;
        let mut registry = DiagnosticsRegistry::default();
        let (descriptor, config) = registry.list_channel_configs().into_iter()
            .find(|(descriptor, _)| descriptor.channel_id == id).unwrap();
        assert_eq!(descriptor.sampling, SamplingPolicy::SampleRate(0.25));
        assert_eq!(descriptor.retention, RetentionPolicy::KeepRecent(256));
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.retention_count, 100);
        assert_eq!((0..8).filter(|_| registry.should_emit_channel(id)).count(), 8);
        registry.set_config(id, ChannelConfig { sample_rate: 0.25, ..config });
        assert_eq!((0..8).filter(|_| registry.should_emit_channel(id)).count(), 2);
        println!("descriptor declares 25 percent; generic config admits 8/8; explicit 25 percent admits 2/8");
    }

    struct RefuseNative;
    impl NativeModRuntime for RefuseNative {
        fn activate(&self, _id: &str) -> Result<(), String> { Err("audit refusal".into()) }
    }
    #[test]
    fn no_native_runtime_still_publishes_active_capabilities() {
        assert!(std::env::var_os("GRAPHSHELL_DISABLE_MODS").is_none());
        assert!(std::env::var_os("GRAPHSHELL_DISABLE_VERSO").is_none());
        let mut registry = ModRegistry::new();
        registry.resolve_dependencies().unwrap();
        let loaded = registry.load_all();
        assert!(loaded.iter().any(|id| id == "mod:web-runtime"));
        assert_eq!(registry.get_status("mod:web-runtime"), Some(ModStatus::Active));
        assert!(registry.is_capability_available("viewer:webview"));
        let mut refused = ModRegistry::new().with_native_runtime(std::sync::Arc::new(RefuseNative));
        refused.resolve_dependencies().unwrap();
        assert!(refused.load_all().is_empty());
        assert!(!refused.is_capability_available("viewer:webview"));
        assert!(runtime_has_capability("viewer:webview"));
        println!("absent native runtime: Active + webview; injected refusal: none; independent global query: true");
    }

    fn manifest(id: &str, provides: &[&str], requires: &[&str]) -> ModManifest {
        ModManifest::new(id, id, ModType::Native,
            provides.iter().map(|v| (*v).into()).collect(),
            requires.iter().map(|v| (*v).into()).collect(), vec![])
    }
    #[test]
    fn failed_dependency_does_not_block_dependent_activation() {
        let manifests = vec![manifest("mod:provider", &["cap:provider"], &[]),
            manifest("mod:dependent", &["cap:dependent"], &["cap:provider"])];
        let mut registry = ModRegistry::from_manifests_for_tests(manifests.clone());
        registry.resolve_dependencies().unwrap();
        let (tx, rx) = diagnostic_channel(IngressLimits::default());
        let _scope = scoped_thread_sender(tx);
        let mut calls = Vec::new();
        let loaded = registry.load_all_with_extensions(|manifest, _| {
            calls.push(manifest.mod_id.clone());
            if manifest.mod_id == "mod:provider" { Err(ModActivationError::failed("audit provider failure")) }
            else { Ok(vec![]) }
        }, |_| Ok(()));
        assert_eq!(calls, vec!["mod:provider", "mod:dependent"]);
        assert_eq!(loaded, vec!["mod:dependent"]);
        assert_eq!(registry.get_status("mod:provider"), Some(ModStatus::Failed));
        assert_eq!(registry.get_status("mod:dependent"), Some(ModStatus::Active));
        assert!(!registry.is_capability_available("cap:provider"));
        assert!(registry.is_capability_available("cap:dependent"));
        assert!(rx.try_drain(100).unwrap().packets.iter().any(|packet| matches!(packet.event,
            DiagnosticEvent::MessageSent { channel_id: diagnostics::channels::CHANNEL_MOD_LOAD_SUCCEEDED, .. })));
        let mut control = ModRegistry::from_manifests_for_tests(manifests);
        control.resolve_dependencies().unwrap();
        assert_eq!(control.load_all_with_extensions(|_, _| Ok(vec![]), |_| Ok(())).len(), 2);
        let mut missing = ModRegistry::from_manifests_for_tests(vec![manifest("mod:dependent", &[], &["missing"])]);
        assert!(matches!(missing.resolve_dependencies(), Err(ModDependencyError::MissingRequirement { .. })));
        println!("provider Failed; dependent callback called, Active and capability published; all-success/missing controls pass");
    }

    #[test]
    fn repeated_load_bypasses_quarantine_and_forgets_unremoved_record() {
        let mut registry = ModRegistry::from_manifests_for_tests(vec![manifest("mod:target", &["cap:target"], &[])]);
        registry.resolve_dependencies().unwrap();
        let old = ModExtensionRecord::Action { action_id: "old:effect".into() };
        let mut installed = vec![];
        registry.load_all_with_extensions(|_, _| {
            installed.push(old.clone());
            Err(ModActivationError::rollback("audit failure", vec![old.clone()]))
        }, |_| Err("audit rollback refusal".into()));
        assert_eq!(registry.get_status("mod:target"), Some(ModStatus::Quarantined));
        assert_eq!(registry.extension_records_for("mod:target"), Some(&[old.clone()][..]));
        let new = ModExtensionRecord::Action { action_id: "new:effect".into() };
        let mut rollback_calls = 0;
        let loaded = registry.load_all_with_extensions(|_, _| {
            installed.push(new.clone());
            Ok(vec![new.clone()])
        }, |_| { rollback_calls += 1; Ok(()) });
        assert_eq!(loaded, vec!["mod:target"]);
        assert_eq!(registry.get_status("mod:target"), Some(ModStatus::Active));
        assert_eq!(registry.extension_records_for("mod:target"), Some(&[new.clone()][..]));
        assert_eq!(installed, vec![old.clone(), new.clone()]);
        assert_eq!(rollback_calls, 0);
        registry.unload_mod_with("mod:target", |record| {
            let index = installed.iter().position(|r| *r == record).unwrap();
            installed.remove(index); Ok(())
        }).unwrap();
        assert_eq!(installed, vec![old]);
        assert_eq!(registry.get_status("mod:target"), Some(ModStatus::Unloaded));
        println!("quarantine -> repeated load Active -> unload Unloaded while old callback-owned effect remains");
    }

    #[test]
    fn remap_codec_loses_legal_delimiter_characters() {
        use input::*;
        let key = |ch| InputBinding::Key { modifiers: ModifierMask::NONE, keycode: Keycode::Char(ch) };
        for ch in ['a', '|', '>'] {
            let remap = InputBindingRemap { old: key(ch), new: key('b'), context: InputContext::GraphView };
            let decoded = InputBindingRemap::decode(&remap.encode());
            if ch == '|' { assert!(decoded.is_err()); }
            else { assert_eq!(decoded.unwrap(), remap); }
        }
        let binding = InputBinding::Chord(vec![key('>'), key('a')]);
        assert!(binding.label().parse::<InputBinding>().is_err());
        let control = InputBinding::Chord(vec![key('x'), key('a')]);
        assert_eq!(control.label().parse::<InputBinding>().unwrap(), control);
        println!("plain remap and ordinary chord round-trip; pipe remap and greater-than chord fail");
    }
}
