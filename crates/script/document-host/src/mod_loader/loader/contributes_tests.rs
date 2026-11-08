// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use inker::PresentationDeclaration;

use super::contributes::{undeclared_record, verify_records};
use super::tests::write_wasm_fixture;
use super::*;

const WEB_RUNTIME: &str = "mod:web-runtime";

fn knot_presentation() -> Contribution {
    Contribution::Presentation(
        PresentationDeclaration::surface("knot.editor")
            .schemes(["knot"])
            .content_types(["text/x-knot"])
            .extensions(["kn"]),
    )
}

fn manifest_contributing(contributes: Vec<Contribution>) -> ModManifest {
    ModManifest::new("mod:knot", "Knot", ModType::Native, vec![], vec![], vec![])
        .with_contributes(contributes)
}

fn built_in(mod_id: &str) -> ModManifest {
    discover_native_mods()
        .into_iter()
        .find(|manifest| manifest.mod_id == mod_id)
        .unwrap_or_else(|| panic!("{mod_id} is built in"))
}

fn precedes(order: &[ModManifest], first: &str, second: &str) -> bool {
    let position = |id: &str| order.iter().position(|manifest| manifest.mod_id == id);
    position(first) < position(second)
}

#[test]
fn the_web_runtime_contributes_a_presentation_and_provides_nothing() {
    let verso = built_in(WEB_RUNTIME);

    assert!(verso.provides.is_empty());
    let [Contribution::Presentation(declaration)] = &verso.contributes[..] else {
        panic!("one presentation, found {:?}", verso.contributes);
    };
    assert_eq!(declaration.id, "viewer:webview");
    assert_eq!(declaration.presents.schemes, ["http", "https", "data"]);
}

#[test]
fn built_in_provides_and_requires_hold_dependency_names_only() {
    let manifests = discover_native_mods();

    assert_eq!(manifests.len(), 7);
    for manifest in &manifests {
        let names = manifest.provides.iter().chain(&manifest.requires);
        assert!(
            names.clone().all(|name| !name.contains(':')),
            "{}",
            manifest.mod_id
        );
        let contributes = !manifest.contributes.is_empty();
        assert_eq!(
            contributes,
            manifest.mod_id == WEB_RUNTIME,
            "{}",
            manifest.mod_id
        );
    }
}

#[test]
fn moving_ids_into_contributes_leaves_the_load_order_unchanged() {
    let current = discover_native_mods();
    let mut before = current.clone();
    let verso = before.iter_mut().find(|m| m.mod_id == WEB_RUNTIME).unwrap();
    verso.contributes.clear();
    verso.provides = [
        "protocol:http",
        "protocol:https",
        "protocol:data",
        "viewer:webview",
    ]
    .map(String::from)
    .to_vec();

    assert_eq!(
        built_in(WEB_RUNTIME).requires,
        ["ProtocolRegistry", "ViewerRegistry"]
    );
    for manifests in [&before, &current] {
        let order = resolve_mod_load_order(manifests).expect("built-ins resolve");
        assert_eq!(order.len(), 7);
        assert!(precedes(&order, "mod:core-protocol", WEB_RUNTIME));
        assert!(precedes(&order, "mod:core-viewer", WEB_RUNTIME));
    }
}

#[test]
fn a_contributed_id_is_not_a_dependency_name() {
    let needy = ModManifest::new(
        "mod:needy",
        "Needy",
        ModType::Native,
        vec![],
        vec!["knot.editor".into()],
        vec![],
    );
    let manifests = [manifest_contributing(vec![knot_presentation()]), needy];

    assert_eq!(
        resolve_mod_load_order(&manifests),
        Err(ModDependencyError::MissingRequirement {
            mod_id: "mod:needy".into(),
            requirement: "knot.editor".into(),
        })
    );
}

#[test]
fn the_web_presentation_is_active_only_while_its_mod_is() {
    let mut registry = ModRegistry::new();
    registry.resolve_dependencies().expect("built-ins resolve");
    assert!(registry.active_contributions().is_empty());
    registry.load_all();

    let active = registry.active_contributions();
    assert!(active.iter().any(|c| c.id() == "viewer:webview"));
    assert!(
        active
            .iter()
            .all(|c| c.kind() == ContributionKind::Presentation)
    );
}

#[test]
fn contributions_round_trip_through_toml() {
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Document {
        contributes: Vec<Contribution>,
    }
    let document = Document {
        contributes: vec![
            knot_presentation(),
            Contribution::PaneAdapter {
                id: "signalman".into(),
            },
            Contribution::Pane {
                id: "steward".into(),
            },
            Contribution::Servitor {
                id: "helper".into(),
            },
        ],
    };

    let text = toml::to_string(&document).expect("contributions serialize");
    assert!(text.contains(r#"kind = "pane-adapter""#), "{text}");
    assert_eq!(
        toml::from_str::<Document>(&text).expect("contributions parse"),
        document
    );
}

#[test]
fn a_wasm_manifest_declares_contributions_in_toml() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let module = write_wasm_fixture(
        &temp_dir,
        "playback",
        r#"
        mod_id = "mod:playback"

        [[contributes]]
        kind = "presentation"
        id = "playback.audio"
        operations = ["play", "annotate"]
        [contributes.presents]
        extensions = ["ogg", "flac"]
        [contributes.reaches]
        mode = "Headless"
        a11y = "Opaque"

        [[contributes]]
        kind = "pane-adapter"
        id = "signalman"
        "#,
    );

    let (manifest, _) = read_wasm_mod_from_path(&module).expect("manifest reads");

    let [Contribution::Presentation(audio), adapter] = &manifest.contributes[..] else {
        panic!("two contributions, found {:?}", manifest.contributes);
    };
    assert_eq!(audio.id, "playback.audio");
    assert_eq!(audio.presents.extensions, ["ogg", "flac"]);
    assert_eq!(
        adapter,
        &Contribution::PaneAdapter {
            id: "signalman".into()
        }
    );
}

#[test]
fn a_wasm_manifest_with_an_unknown_contribution_kind_is_invalid() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let module = write_wasm_fixture(
        &temp_dir,
        "odd",
        "mod_id = \"mod:odd\"\n[[contributes]]\nkind = \"engine\"\nid = \"x\"\n",
    );

    assert!(matches!(
        read_wasm_mod_from_path(&module),
        Err(ModLoadPathError::InvalidManifest { .. })
    ));
}

fn scheme(scheme: &str) -> ModExtensionRecord {
    ModExtensionRecord::ProtocolScheme {
        scheme: scheme.into(),
        previously_present: false,
    }
}

#[test]
fn declared_records_pass_verification_whatever_their_case() {
    let records = vec![
        scheme("KNOT"),
        ModExtensionRecord::PresentationContentType {
            content_type: "text/x-knot".into(),
            previous_presentation_id: None,
        },
        ModExtensionRecord::PresentationExtension {
            extension: ".KN".into(),
            previous_presentation_id: None,
        },
        ModExtensionRecord::PresentationReaches {
            presentation_id: "knot.editor".into(),
            previous: None,
        },
        ModExtensionRecord::Action {
            action_id: "action:fold".into(),
        },
    ];

    let contributes = [knot_presentation()];
    assert_eq!(undeclared_record(&contributes, &records), None);
    assert_eq!(
        verify_records("mod:knot", &contributes, records.clone()),
        Ok(records)
    );
}

#[test]
fn an_undeclared_record_is_a_named_refusal_carrying_everything_applied() {
    let applied = vec![scheme("knot"), scheme("ipfs")];

    let refusal = verify_records("mod:knot", &[knot_presentation()], applied.clone())
        .expect_err("ipfs is not declared");

    assert_eq!(refusal.undeclared_record(), Some(&scheme("ipfs")));
    assert!(
        refusal.reason().contains("mod:knot"),
        "{}",
        refusal.reason()
    );
    assert_eq!(
        refusal,
        ModActivationError::undeclared("mod:knot", scheme("ipfs"), applied)
    );
}

#[test]
fn a_mod_that_declares_nothing_may_register_no_scheme_or_reach() {
    let reaches = ModExtensionRecord::PresentationReaches {
        presentation_id: "knot.editor".into(),
        previous: None,
    };

    for record in [scheme("knot"), reaches] {
        let refusal =
            verify_records("mod:knot", &[], vec![record.clone()]).expect_err("nothing is declared");
        assert_eq!(refusal.undeclared_record(), Some(&record));
    }
}

#[test]
fn activation_refuses_an_undeclared_record_and_rolls_back_all_of_them() {
    let mut registry = ModRegistry::from_manifests_for_tests(vec![manifest_contributing(vec![
        knot_presentation(),
    ])]);
    registry.resolve_dependencies().expect("one mod resolves");
    let mut rolled_back = Vec::new();

    let loaded = registry.load_all_with_extensions(
        |_, _| Ok(vec![scheme("knot"), scheme("ipfs")]),
        |record| {
            rolled_back.push(record);
            Ok(())
        },
    );

    assert!(loaded.is_empty());
    assert_eq!(registry.get_status("mod:knot"), Some(ModStatus::Failed));
    assert_eq!(registry.extension_records_for("mod:knot"), None);
    assert_eq!(rolled_back, [scheme("ipfs"), scheme("knot")]);
}

#[test]
fn activation_keeps_the_records_a_manifest_declares() {
    let mut registry = ModRegistry::from_manifests_for_tests(vec![manifest_contributing(vec![
        knot_presentation(),
    ])]);
    registry.resolve_dependencies().expect("one mod resolves");

    let loaded = registry.load_all_with_extensions(|_, _| Ok(vec![scheme("knot")]), |_| Ok(()));

    assert_eq!(loaded, ["mod:knot"]);
    assert_eq!(registry.get_status("mod:knot"), Some(ModStatus::Active));
    assert_eq!(
        registry.extension_records_for("mod:knot"),
        Some(&[scheme("knot")][..])
    );
}
