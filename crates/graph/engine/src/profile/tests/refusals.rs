// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One refusal fixture per resolution rule (declared-lens §3.5), each beside
//! a positive control that differs only in the fault.

use eidetic::pack::{Broken, Origin, Refusal};

use super::support::*;
use crate::model::GraphStratum;
use crate::profile::lens::AdmittedAs;
use crate::profile::load::{Diagnostic, Limits, LoadRefusal, load};
use crate::profile::resolve::{Offered, satisfies};

#[test]
fn a_dependency_cycle_is_refused() {
    let ok = Source::of(vec![
        local(lens("l", &[("a", "1.0.0")], &[])),
        local(pack("a", "1.0.0", &[("b", "1.0.0")], &[])),
        local(pack("b", "1.0.0", &[], &[])),
    ]);
    assert!(load_with(&select("l"), &ok).is_ok(), "control: a chain");
    let cyclic = Source::of(vec![
        local(lens("l", &[("a", "1.0.0")], &[])),
        local(pack("a", "1.0.0", &[("b", "1.0.0")], &[])),
        local(pack("b", "1.0.0", &[("a", "1.0.0")], &[])),
    ]);
    match load_with(&select("l"), &cyclic) {
        Err(LoadRefusal::Cycle { packs }) => assert_eq!(packs.len(), 3, "{packs:?}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn two_versions_of_one_id_are_refused() {
    let offers = |b_needs: &str| {
        Source::of(vec![
            local(lens("l", &[("a", "1.0.0"), ("b", "1.0.0")], &[])),
            local(pack("a", "1.0.0", &[], &[])),
            local(pack("a", "1.4.0", &[], &[])),
            local(pack("a", "2.1.0", &[], &[])),
            local(pack("b", "1.0.0", &[("a", b_needs)], &[])),
        ])
    };
    let ok = load_with(&select("l"), &offers("1.2.0")).unwrap();
    let a = ok
        .selection
        .pins
        .iter()
        .find(|p| p.pin.id.ends_with("/a"))
        .unwrap();
    assert_eq!(
        a.pin.version, "1.4.0",
        "control: the highest compatible version wins"
    );
    assert!(matches!(
        load_with(&select("l"), &offers("2.0.0")),
        Err(LoadRefusal::TwoVersions { .. })
    ));
}

#[test]
fn an_unsatisfied_requirement_is_refused() {
    let offers = |need: &str| {
        Source::of(vec![
            local(lens("l", &[("a", need)], &[])),
            local(pack("a", "1.3.0", &[], &[])),
        ])
    };
    assert!(load_with(&select("l"), &offers("1.1.0")).is_ok(), "control");
    for need in ["1.4.0", "3.0.0", "0.9.0"] {
        assert!(
            matches!(
                load_with(&select("l"), &offers(need)),
                Err(LoadRefusal::Unsatisfied { .. })
            ),
            "{need}"
        );
    }
    assert!(matches!(
        load_with(&select("missing"), &offers("1.1.0")),
        Err(LoadRefusal::Unsatisfied { .. })
    ));
}

#[test]
fn caret_matching_is_one_function() {
    assert!(satisfies("1.4.0", "1.2.0") && satisfies("1.2.0", "1.2.0"));
    assert!(!satisfies("2.0.0", "1.2.0") && !satisfies("1.1.9", "1.2.0"));
    assert!(satisfies("0.2.5", "0.2.1") && !satisfies("0.3.0", "0.2.1"));
    assert!(
        !satisfies("1.3.0-rc.1", "1.2.0"),
        "a pre-release needs a pre-release bound"
    );
}

#[test]
fn an_unsettled_overlap_is_refused() {
    let offers = |settle: &[(&str, &str)]| {
        Source::of(vec![
            local(lens("l", &[("a", "1.0.0"), ("b", "1.0.0")], settle)),
            local(pack(
                "a",
                "1.0.0",
                &[],
                &[("t", "resource"), ("same", "surface")],
            )),
            local(pack(
                "b",
                "1.0.0",
                &[],
                &[("t", "surface"), ("same", "surface")],
            )),
        ])
    };
    assert!(matches!(
        load_with(&select("l"), &offers(&[])),
        Err(LoadRefusal::Overlap { .. })
    ));
    let settled = load_with(&select("l"), &offers(&[("t", "b")])).unwrap();
    let t = settled.terms.entry("https://example.org/ns#t").unwrap();
    assert_eq!(t.row.stratum, GraphStratum::Surface, "b wins t");
    assert_eq!(t.pack, "https://example.org/packs/b");
    let unused = load_with(&select("l"), &offers(&[("t", "a"), ("same", "a")])).unwrap();
    assert!(
        unused.diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::UnusedSettlement { term, .. } if term.ends_with("#same")
        )),
        "equal rows need no settlement"
    );
}

#[test]
fn broken_is_refused_at_any_origin() {
    let a = pack("a", "1.0.0", &[], &[]);
    let other = pack("a", "1.0.1", &[], &[]);
    for origin in [Origin::Local, Origin::CrossesTrust] {
        let mut forged = signed(other.clone(), 1, origin);
        forged.canonical = a.clone();
        let src = Source::of(vec![local(lens("l", &[("a", "1.0.0")], &[])), forged]);
        assert!(
            matches!(
                load_with(&select("l"), &src),
                Err(LoadRefusal::Trust {
                    refusal: Refusal::Broken(Broken::Signature),
                    ..
                })
            ),
            "{origin:?}"
        );
    }
}

#[test]
fn trust_across_the_boundary_needs_an_accepted_signer() {
    let a = pack("a", "1.0.0", &[], &[]);
    let id = "https://example.org/packs/a".to_string();
    let run = |offer: eidetic::pack::Origin, seed: Option<u8>, accepted: Vec<(String, u8)>| {
        let a = match seed {
            Some(s) => signed(a.clone(), s, offer),
            None => Offered {
                origin: offer,
                ..local(a.clone())
            },
        };
        let src = Source::of(vec![local(lens("l", &[("a", "1.0.0")], &[])), a]);
        load(&select("l"), &src, &policy(accepted), &Limits::default()).map(|p| {
            p.selection
                .pins
                .iter()
                .find(|r| r.pin.id == id)
                .unwrap()
                .admitted
                .clone()
        })
    };
    use Origin::{CrossesTrust, Local};
    assert!(matches!(
        run(CrossesTrust, None, vec![]),
        Err(LoadRefusal::Trust {
            refusal: Refusal::Unsigned,
            ..
        })
    ));
    assert!(matches!(
        run(CrossesTrust, Some(9), vec![]),
        Err(LoadRefusal::Trust {
            refusal: Refusal::UnacceptedSigner(_),
            ..
        })
    ));
    assert_eq!(run(CrossesTrust, Some(1), vec![]), Ok(AdmittedAs::Release));
    assert!(matches!(
        run(CrossesTrust, Some(9), vec![(id.clone(), 9)]),
        Ok(AdmittedAs::Reviewed { .. })
    ));
    assert_eq!(run(Local, None, vec![]), Ok(AdmittedAs::Local));
    assert_eq!(run(Local, Some(9), vec![]), Ok(AdmittedAs::Local), "D12");
}

#[test]
fn declare_and_verify_and_limits_refuse() {
    let outside = compiled(&WORKSPACE.replace("provides = [\"rel:arrangement.\",", "provides = ["));
    let src = Source::of(vec![local(outside), local(compiled(WORKSPACE_LENS))]);
    assert!(matches!(
        load_with(&workspace_lens(), &src),
        Err(LoadRefusal::Undeclared { .. })
    ));
    let handled = compiled(&WORKSPACE.replace(
        "labels = { en = \"traversal\" }\n\n[[vocabulary.terms]]\niri = \"rel:",
        "handler = \"ws:trail\"\nlabels = { en = \"traversal\" }\n\n[[vocabulary.terms]]\niri = \"rel:",
    ));
    let mut src = Source::of(vec![local(handled), local(compiled(WORKSPACE_LENS))]);
    assert!(matches!(
        load_with(&workspace_lens(), &src),
        Err(LoadRefusal::UnknownHandler { .. })
    ));
    src.handlers
        .insert("https://mere.computer/ns/test/workspace#trail".to_string());
    assert!(
        load_with(&workspace_lens(), &src).is_ok(),
        "control: a known handler"
    );
    let small = Limits {
        max_terms: 10,
        ..Limits::default()
    };
    assert!(matches!(
        load(
            &workspace_lens(),
            &workspace_source(),
            &policy(vec![]),
            &small
        ),
        Err(LoadRefusal::Limit { limit: "terms", .. })
    ));
    let mut garbled = compiled(WORKSPACE);
    garbled.push(0);
    let src = Source::of(vec![local(garbled), local(compiled(WORKSPACE_LENS))]);
    assert!(matches!(
        load_with(&workspace_lens(), &src),
        Err(LoadRefusal::Decode { .. })
    ));
    let not_a_lens = Source::of(vec![local(pack("a", "1.0.0", &[], &[]))]);
    assert!(matches!(
        load_with(&select("a"), &not_a_lens),
        Err(LoadRefusal::NotALens { .. })
    ));
}

#[test]
fn unobserved_values_fall_back_and_are_kept() {
    let text = WORKSPACE.replace(
        "\"axis:durability\" = \"durability:session\" }\nlabels = { en = \"tile-group\" }",
        "\"axis:durability\" = [\"durability:ephemeral\", \"durability:session\"] }\nlabels = { en = \"tile-group\" }",
    ).replace(
        "\"axis:durability\" = \"durability:session\" }\nlabels = { en = \"split-pair\" }",
        "\"axis:durability\" = \"durability:ephemeral\", \"axis:bounce\" = \"ws:high\" }\nlabels = { en = \"split-pair\" }",
    );
    let src = Source::of(vec![
        local(compiled(&text)),
        local(compiled(WORKSPACE_LENS)),
    ]);
    let p = load_with(&workspace_lens(), &src).unwrap();
    use crate::profile::intern::Durability;
    let tile = p
        .terms
        .entry("https://mere.computer/ns/rel#arrangement.tile-group")
        .unwrap();
    assert_eq!(
        tile.row.durability,
        Durability::Session,
        "the chain's next observed value"
    );
    let split = p
        .terms
        .entry("https://mere.computer/ns/rel#arrangement.split-pair")
        .unwrap();
    assert_eq!(
        split.row.durability,
        Durability::RetainedUntilCompaction,
        "the core default"
    );
    assert!(
        split
            .row
            .declared
            .contains_key("https://mere.computer/ns/axis#bounce"),
        "kept (D31)"
    );
    let notes: Vec<_> = p
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d,
                Diagnostic::UnobservedValue { .. } | Diagnostic::UnobservedAxis { .. }
            )
        })
        .collect();
    assert_eq!(notes.len(), 2, "{notes:?}");
}
