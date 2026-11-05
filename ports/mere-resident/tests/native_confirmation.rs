// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exercise the actual Linux UI-to-owner-policy boundary with a fake GUI backend.
//! Each child owns its environment; no real dialog, vault or owner state is used.
#![cfg(target_os = "linux")]

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use mere_resident::applications::OwnerConsent;
use mere_resident::identity_ui::SystemNativeIdentityUi;
use mere_resident::settings::OwnerSettings;
use ostiary::app_admission::AppId;
use ostiary::declaration::{AppDeclaration, CustodyOperation, CustodyUse, NamePattern};

const BACKEND: &str = r#"#!/bin/sh
case " $* " in
  *" --menu "*)
    printf '%s\000' "$@" > "$MERE_CONFIRM_ARGS"
    case "$MERE_CONFIRM_MODE" in
      failure) printf 'synthetic parser failure\n' >&2; exit 1;;
      no) printf 'no\n'; exit 0;;
      yes) printf 'yes\n'; exit 0;;
    esac;;
  *" --help "*) printf 'synthetic kdialog --attach --menu --default\n'; exit 0;;
  *" --version "*) printf 'synthetic kdialog 1\n'; exit 0;;
esac
printf 'unexpected synthetic backend invocation\n' >&2
exit 88
"#;

fn scenario(mode: &str) {
    let directory = tempfile::tempdir().unwrap();
    let backend = directory.path().join("kdialog");
    std::fs::write(&backend, BACKEND).unwrap();
    std::fs::set_permissions(&backend, std::fs::Permissions::from_mode(0o700)).unwrap();
    let args = directory.path().join("args");
    let path = format!("{}:/usr/bin:/bin", directory.path().display());
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "checked_native_child", "--nocapture"])
        .env("PATH", path)
        .env("DISPLAY", ":synthetic-confirmation")
        .env("XDG_SESSION_DESKTOP", "KDE")
        .env("XDG_CURRENT_DESKTOP", "KDE")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("SSH_CLIENT")
        .env_remove("SSH_CONNECTION")
        .env_remove("SSH_TTY")
        .env("MERE_CONFIRM_MODE", mode)
        .env("MERE_CONFIRM_ARGS", &args)
        .env(
            "MERE_CONFIRM_SETTINGS",
            directory.path().join("settings.json"),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arguments = std::fs::read(&args).expect("the checked native choice was reached");
    let arguments: Vec<_> = arguments.split(|b| *b == 0).collect();
    assert!(
        arguments
            .windows(2)
            .any(|a| a == [b"--default".as_slice(), b"no"])
    );
    assert!(
        arguments
            .iter()
            .any(|a| String::from_utf8_lossy(a).contains("\"fixture-app\""))
    );
    assert!(
        !arguments
            .iter()
            .any(|a| a.windows(15).any(|s| s == b"INVALID MESSAGE"))
    );
}

#[test]
fn backend_failure_leaves_actual_owner_policy_unchanged() {
    scenario("failure");
}

#[test]
fn explicit_native_no_is_kept() {
    scenario("no");
}

#[test]
fn explicit_native_yes_grants_only_the_declaration() {
    scenario("yes");
}

#[test]
fn checked_native_child() {
    let Ok(mode) = std::env::var("MERE_CONFIRM_MODE") else {
        return;
    };
    let settings = PathBuf::from(std::env::var_os("MERE_CONFIRM_SETTINGS").unwrap());
    let declaration = AppDeclaration {
        routes: vec![NamePattern::exact("custody")],
        custody: CustodyUse {
            operations: [CustodyOperation::Status, CustodyOperation::Roster].into(),
            ..CustodyUse::default()
        },
        ..AppDeclaration::default()
    };
    let app = AppId::new("fixture-app");
    let consent = OwnerConsent::new(
        settings.clone(),
        AppRouteGrants::new(AllowedAppRoutes::none()),
        Arc::new(SystemNativeIdentityUi::default()),
    );
    let answer = consent.decide(&app, &declaration, None);
    let policy = OwnerSettings::load(&settings).unwrap();
    match mode.as_str() {
        "failure" => {
            assert!(answer.is_err());
            assert!(!settings.exists());
            assert!(policy.refused_applications.is_empty());
            assert!(policy.applications.is_none());
        },
        "no" => {
            assert!(!answer.unwrap());
            assert_eq!(policy.refused_applications, vec![app]);
            assert!(policy.applications.is_none());
        },
        "yes" => {
            assert!(answer.unwrap());
            assert!(policy.refused_applications.is_empty());
            let granted = policy.applications.unwrap();
            assert_eq!(granted.len(), 1);
            assert_eq!(granted[0].app, app);
            assert_eq!(granted[0].routes, declaration.routes);
            assert_eq!(
                granted[0].custody.operations,
                declaration.custody.operations
            );
        },
        _ => panic!("unexpected fixture mode"),
    }
}
