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

const KDIALOG: &str = r#"#!/bin/sh
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

// Exit codes and output follow Zenity 4.2.2 as measured on Fedora 45: an unknown
// option exits 255, and a timeout exits 5 after printing the default tag.
const ZENITY: &str = r#"#!/bin/sh
case " $* " in
  *" --radiolist "*)
    printf '%s\000' "$@" > "$MERE_CONFIRM_ARGS"
    case "$MERE_CONFIRM_MODE" in
      failure) printf 'This option is not available.\n' >&2; exit 255;;
      timeout) printf 'no\n'; exit 5;;
      no) printf 'no\n'; exit 0;;
      yes) printf 'yes\n'; exit 0;;
    esac;;
  *" --version "*) printf '4.2.2\n'; exit 0;;
esac
printf 'unexpected synthetic backend invocation\n' >&2
exit 88
"#;

fn scenario(mode: &str) {
    run_scenario("kdialog", KDIALOG, "KDE", mode);
}

fn gnome_scenario(mode: &str) {
    run_scenario("zenity", ZENITY, "gnome", mode);
}

fn run_scenario(name: &str, script: &str, desktop: &str, mode: &str) {
    let directory = tempfile::tempdir().unwrap();
    let backend = directory.path().join(name);
    std::fs::write(&backend, script).unwrap();
    std::fs::set_permissions(&backend, std::fs::Permissions::from_mode(0o700)).unwrap();
    let args = directory.path().join("args");
    let path = format!("{}:/usr/bin:/bin", directory.path().display());
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "checked_native_child", "--nocapture"])
        .env("PATH", path)
        .env("DISPLAY", ":synthetic-confirmation")
        .env("XDG_SESSION_DESKTOP", desktop)
        .env("XDG_CURRENT_DESKTOP", desktop)
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
    let has = |flag: &[u8]| arguments.iter().any(|a| *a == flag);
    if name == "kdialog" {
        assert!(
            arguments
                .windows(2)
                .any(|a| a == [b"--default".as_slice(), b"no"])
        );
    } else {
        assert!(
            arguments
                .windows(3)
                .any(|a| a == [b"TRUE".as_slice(), b"no", b"No"])
        );
        assert!(has(b"--no-markup") && has(b"--hide-column=2") && has(b"--print-column=2"));
    }
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
fn zenity_parser_failure_leaves_actual_owner_policy_unchanged() {
    gnome_scenario("failure");
}

#[test]
fn zenity_timeout_with_default_tag_is_not_an_answer() {
    gnome_scenario("timeout");
}

#[test]
fn zenity_explicit_no_is_kept() {
    gnome_scenario("no");
}

#[test]
fn zenity_explicit_yes_grants_only_the_declaration() {
    gnome_scenario("yes");
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
        "failure" | "timeout" => {
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
