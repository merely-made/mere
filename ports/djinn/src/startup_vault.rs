// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How the resident opens its vault without the environment (vault lock
//! rulings 42 and 62 to 66).
//!
//! A passphrase vault starts locked: the resident asks before it builds any
//! storage key, profile, door or lane (ruling 66), first in a native box and
//! else at its terminal (ruling 63). A cancel, a mismatch or a refused open
//! asks again after a backoff; with neither a box nor a terminal there is
//! nothing to ask with, and the start fails naming both. With no vault the
//! prompt creates one, asking twice (ruling 64). The OS-rooted vault (DPAPI)
//! still opens unattended, unless its vault carries the persisted lock: then
//! the start waits at the same prompt, Hello first (rulings 5, 76, 80). The
//! environment is never read (ruling 7).

use std::path::Path;
use std::time::Duration;

use castellan::custody::bootstrap::{self, OpenedStorage, PASSPHRASE_VAULT_FILE, Unlock};
use castellan::custody::{
    AUTO_UNLOCK_ROOT_FILE, IdentityStorage, OsPresence, SealedProfileStorage, UnlockMethod,
};
use zeroize::Zeroizing;

/// Which vault a start opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultChoice {
    /// The OS-held root opens it unattended (DPAPI).
    AutoOs,
    /// The passphrase vault, which starts locked.
    Passphrase,
}

/// The passphrase vault when the platform holds no root, when the directory
/// already holds one, or when a passphrase was handed over and no OS-rooted
/// vault is there yet; else the OS's (whose persisted lock a handed-over
/// passphrase then answers).
pub fn choose(vault_dir: &Path, passphrase_given: bool) -> VaultChoice {
    let os_root = cfg!(windows);
    let os_vault = vault_dir.join(AUTO_UNLOCK_ROOT_FILE).exists();
    if !os_root || vault_dir.join(PASSPHRASE_VAULT_FILE).exists() || (passphrase_given && !os_vault)
    {
        VaultChoice::Passphrase
    } else {
        VaultChoice::AutoOs
    }
}

/// What a prompt is for, so each source can word it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// Unlock the existing vault; `retry` carries why the last try failed.
    Unlock { retry: Option<String> },
    /// First run: choose the new vault's passphrase.
    Create { retry: Option<String> },
    /// First run: type it again.
    Confirm,
}

impl Ask {
    /// The words a box or a terminal shows.
    pub fn message(&self) -> String {
        let (base, retry) = match self {
            Ask::Unlock { retry } => ("Enter the vault passphrase to start.", retry),
            Ask::Create { retry } => (
                "No identity vault yet. Choose a passphrase for a new one.",
                retry,
            ),
            Ask::Confirm => return "Type the new vault passphrase again.".into(),
        };
        match retry {
            Some(reason) => format!("{reason} {base}"),
            None => base.into(),
        }
    }
}

/// One answer from a prompt.
pub enum Answer {
    /// What was typed.
    Passphrase(Zeroizing<String>),
    /// Closed without an answer.
    Cancelled,
}

/// A source of passphrases: the native box, the terminal, or a script.
pub trait Prompt {
    /// Ask once. `Err` when this source cannot ask at all.
    fn ask(&mut self, ask: &Ask) -> Result<Answer, String>;

    /// The OS's own check of the user (Windows Hello), where this source
    /// offers one; `None` when unavailable, cancelled or not verified.
    fn presence(&mut self) -> Option<OsPresence> {
        None
    }
}

/// The native box where the desktop has one, else the terminal (ruling 63).
pub struct NativeOrTerminal<U> {
    /// The native identity UI.
    pub native: U,
}

impl<U: crate::identity_ui::NativeIdentityUi> Prompt for NativeOrTerminal<U> {
    fn presence(&mut self) -> Option<OsPresence> {
        self.native.verify_presence().ok().flatten()
    }

    fn ask(&mut self, ask: &Ask) -> Result<Answer, String> {
        let message = ask.message();
        match self.native.ask_vault_passphrase(&message) {
            Ok(Some(passphrase)) => return Ok(Answer::Passphrase(passphrase)),
            Ok(None) => return Ok(Answer::Cancelled),
            Err(_) => {},
        }
        match crate::enrollment::read_from_terminal(&format!("{message} ")) {
            Ok(passphrase) => Ok(Answer::Passphrase(passphrase)),
            Err(error) => Err(format!(
                "no native passphrase box and no terminal to ask at ({error})"
            )),
        }
    }
}

/// The opened vault, and the passphrase that opened it, kept only for the
/// resident's second open of the same directory (Distillery's authority).
pub struct Started {
    /// The opened storage.
    pub opened: OpenedStorage,
    /// Which vault it is.
    pub choice: VaultChoice,
    passphrase: Option<Zeroizing<Vec<u8>>>,
}

impl Started {
    /// The storage, the choice, and the unlock for one more open of the same
    /// vault; whoever drops that unlock drops the last copy of the passphrase.
    pub fn into_parts(self) -> (OpenedStorage, VaultChoice, Unlock) {
        let second = match self.passphrase {
            Some(passphrase) => Unlock::Passphrase(passphrase),
            None => Unlock::AutoOs,
        };
        (self.opened, self.choice, second)
    }
}

/// What a start reports while it waits, for the event file.
pub trait Waiting {
    /// One event and its fields.
    fn emit(&mut self, event: &str, fields: serde_json::Value);
}

impl<F: FnMut(&str, serde_json::Value)> Waiting for F {
    fn emit(&mut self, event: &str, fields: serde_json::Value) {
        self(event, fields)
    }
}

/// The wait before asking again, after `misses` misses: 1 s doubling to 30 s.
pub fn backoff(misses: u32) -> Duration {
    Duration::from_secs((1u64 << misses.min(5)).min(30))
}

/// Open the vault for a start. A passphrase vault waits on `prompt`; `sleep`
/// is the backoff (injected for tests).
pub fn open(
    vault_dir: &Path,
    choice: VaultChoice,
    prompt: &mut dyn Prompt,
    waiting: &mut dyn Waiting,
    sleep: &mut dyn FnMut(Duration),
) -> Result<Started, String> {
    if choice == VaultChoice::AutoOs && castellan::custody::lock_persisted(vault_dir) {
        return open_under_persisted_lock(vault_dir, prompt, waiting, sleep);
    }
    if choice == VaultChoice::AutoOs {
        let opened = bootstrap::open_storage(vault_dir, Unlock::AutoOs)
            .map_err(|error| error.to_string())?;
        return Ok(Started {
            opened,
            choice,
            passphrase: None,
        });
    }
    let exists = vault_dir.join(PASSPHRASE_VAULT_FILE).exists();
    waiting.emit(
        "waiting-for-unlock",
        serde_json::json!({ "creating": !exists }),
    );
    let mut misses = 0u32;
    let mut retry: Option<String> = None;
    loop {
        let ask = match exists {
            true => Ask::Unlock {
                retry: retry.take(),
            },
            false => Ask::Create {
                retry: retry.take(),
            },
        };
        let passphrase = match prompt.ask(&ask)? {
            Answer::Passphrase(passphrase) if passphrase.is_empty() => {
                Some((passphrase, "An empty passphrase is not accepted."))
            },
            Answer::Passphrase(passphrase) => Some((passphrase, "")),
            Answer::Cancelled => None,
        };
        let Some((passphrase, refused)) = passphrase else {
            waiting.emit("unlock-cancelled", serde_json::json!({ "misses": misses }));
            sleep(backoff(misses));
            misses += 1;
            continue;
        };
        if !refused.is_empty() {
            retry = Some(refused.into());
            continue;
        }
        if !exists {
            let Answer::Passphrase(again) = prompt.ask(&Ask::Confirm)? else {
                waiting.emit("unlock-cancelled", serde_json::json!({ "misses": misses }));
                sleep(backoff(misses));
                misses += 1;
                continue;
            };
            if again.as_bytes() != passphrase.as_bytes() {
                retry = Some("The two passphrases did not match.".into());
                continue;
            }
        }
        let bytes = Zeroizing::new(passphrase.as_bytes().to_vec());
        match bootstrap::open_storage(vault_dir, Unlock::Passphrase(bytes.clone())) {
            Ok(opened) => {
                let event = if exists {
                    "unlocked-at-start"
                } else {
                    "vault-created"
                };
                waiting.emit(event, serde_json::json!({}));
                return Ok(Started {
                    opened,
                    choice,
                    passphrase: Some(bytes),
                });
            },
            // Argon2id's cost is the throttle on a wrong passphrase (ruling
            // 19); any refusal is shown and asked again.
            Err(error) => {
                waiting.emit(
                    "unlock-refused",
                    serde_json::json!({ "reason": error.to_string() }),
                );
                retry = Some(format!("That did not open the vault ({error})."));
            },
        }
    }
}

/// An OS-rooted vault whose lock persisted: open it locked and wait for a
/// user act, Hello once and then the passphrase box, before anything else
/// exists (rulings 5, 76). The marker clears with the unlock.
fn open_under_persisted_lock(
    vault_dir: &Path,
    prompt: &mut dyn Prompt,
    waiting: &mut dyn Waiting,
    sleep: &mut dyn FnMut(Duration),
) -> Result<Started, String> {
    waiting.emit(
        "waiting-for-unlock",
        serde_json::json!({ "persisted_lock": true }),
    );
    let storage = SealedProfileStorage::open_locked(vault_dir);
    let mut unlocked = match prompt.presence() {
        Some(verified) => storage.unlock(UnlockMethod::OsPresence(verified)).is_ok(),
        None => false,
    };
    let mut misses = 0u32;
    let mut retry: Option<String> = None;
    while !unlocked {
        let passphrase = match prompt.ask(&Ask::Unlock {
            retry: retry.take(),
        })? {
            Answer::Passphrase(passphrase) if passphrase.is_empty() => {
                retry = Some("An empty passphrase is not accepted.".into());
                continue;
            },
            Answer::Passphrase(passphrase) => passphrase,
            Answer::Cancelled => {
                waiting.emit("unlock-cancelled", serde_json::json!({ "misses": misses }));
                sleep(backoff(misses));
                misses += 1;
                continue;
            },
        };
        match storage.unlock(UnlockMethod::Passphrase(passphrase.as_bytes())) {
            Ok(()) => unlocked = true,
            Err(error) => {
                waiting.emit(
                    "unlock-refused",
                    serde_json::json!({ "reason": error.to_string() }),
                );
                retry = Some(format!("That did not open the vault ({error})."));
            },
        }
    }
    castellan::custody::clear_persisted_lock(vault_dir).map_err(|error| error.to_string())?;
    waiting.emit(
        "unlocked-at-start",
        serde_json::json!({ "persisted_lock": true }),
    );
    Ok(Started {
        opened: OpenedStorage {
            storage: Box::new(storage),
            description: format!(
                "OS auto-unlock sealed records at {} (DPAPI-wrapped root), opened by a user act \
                 under the persisted lock",
                vault_dir.display()
            ),
        },
        choice: VaultChoice::AutoOs,
        passphrase: None,
    })
}

/// One passphrase from standard input, for the harness (ruling 65): one
/// line, read once, never the environment. Only fd 0 exists everywhere.
#[cfg(feature = "passphrase-fd")]
pub fn read_passphrase_fd(fd: u32) -> Result<Zeroizing<String>, String> {
    use std::io::BufRead;
    if fd != 0 {
        return Err(format!(
            "--passphrase-fd {fd}: only 0 (standard input) is read"
        ));
    }
    let mut line = Zeroizing::new(String::new());
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|error| format!("--passphrase-fd 0: {error}"))?;
    let trimmed = line.trim_end_matches(['\r', '\n']).len();
    line.truncate(trimmed);
    Ok(line)
}

/// A handed-over passphrase: one try at unlocking or creating (it answers
/// the confirmation too), so a refused passphrase ends the start.
pub struct Given {
    passphrase: Zeroizing<String>,
    tried: bool,
}

impl Given {
    /// A prompt that answers with `passphrase`.
    pub fn new(passphrase: Zeroizing<String>) -> Self {
        Self {
            passphrase,
            tried: false,
        }
    }
}

impl Prompt for Given {
    fn ask(&mut self, ask: &Ask) -> Result<Answer, String> {
        if *ask != Ask::Confirm {
            if self.tried {
                return Err("the handed-over passphrase did not open the vault".into());
            }
            self.tried = true;
        }
        Ok(Answer::Passphrase(self.passphrase.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct Script(VecDeque<Option<&'static str>>, Vec<Ask>);

    impl Prompt for Script {
        fn ask(&mut self, ask: &Ask) -> Result<Answer, String> {
            self.1.push(ask.clone());
            match self.0.pop_front() {
                Some(Some(text)) => Ok(Answer::Passphrase(Zeroizing::new(text.into()))),
                Some(None) => Ok(Answer::Cancelled),
                None => Err("script ended".into()),
            }
        }
    }

    fn script(answers: &[Option<&'static str>]) -> Script {
        Script(answers.iter().copied().collect(), Vec::new())
    }

    struct Log(Vec<String>);

    impl Waiting for Log {
        fn emit(&mut self, event: &str, _fields: serde_json::Value) {
            self.0.push(event.into());
        }
    }

    fn start(
        dir: &Path,
        prompt: &mut Script,
    ) -> (Result<Started, String>, Vec<String>, Vec<Duration>) {
        start_as(dir, VaultChoice::Passphrase, prompt)
    }

    fn start_as(
        dir: &Path,
        choice: VaultChoice,
        prompt: &mut Script,
    ) -> (Result<Started, String>, Vec<String>, Vec<Duration>) {
        let mut log = Log(Vec::new());
        let mut slept = Vec::new();
        let started = open(dir, choice, prompt, &mut log, &mut |wait| slept.push(wait));
        (started, log.0, slept)
    }

    fn create(dir: &Path) {
        let (started, ..) = start(dir, &mut script(&[Some("right"), Some("right")]));
        let started = started.unwrap();
        let id = personae::vault::ProfileId("p".into());
        bootstrap::load_or_create_profile(&*started.opened.storage, &id).unwrap();
    }

    #[test]
    fn first_run_asks_twice_and_a_mismatch_asks_again() {
        let dir = tempfile::tempdir().unwrap();
        let mut prompt = script(&[Some("one"), Some("two"), Some("right"), Some("right")]);
        let (started, events, _) = start(dir.path(), &mut prompt);
        assert!(started.is_ok());
        assert_eq!(events, ["waiting-for-unlock", "vault-created"]);
        assert_eq!(prompt.1[0], Ask::Create { retry: None });
        assert_eq!(prompt.1[1], Ask::Confirm);
        assert_eq!(
            prompt.1[2],
            Ask::Create {
                retry: Some("The two passphrases did not match.".into())
            }
        );
    }

    #[test]
    fn a_wrong_passphrase_and_a_cancel_ask_again_and_the_right_one_opens() {
        let dir = tempfile::tempdir().unwrap();
        create(dir.path());
        let mut prompt = script(&[Some("wrong"), None, Some("right")]);
        let (started, events, slept) = start(dir.path(), &mut prompt);
        let started = started.unwrap();
        assert_eq!(
            events,
            [
                "waiting-for-unlock",
                "unlock-refused",
                "unlock-cancelled",
                "unlocked-at-start"
            ]
        );
        assert_eq!(
            slept,
            [backoff(0)],
            "only the cancel waits; Argon2 throttles a miss"
        );
        assert!(matches!(prompt.1[1], Ask::Unlock { retry: Some(_) }));
        let (_, choice, second) = started.into_parts();
        assert_eq!(choice, VaultChoice::Passphrase);
        assert!(
            matches!(second, Unlock::Passphrase(_)),
            "kept for the second open"
        );
    }

    #[test]
    fn an_empty_passphrase_is_refused_without_opening() {
        let dir = tempfile::tempdir().unwrap();
        create(dir.path());
        let mut prompt = script(&[Some(""), Some("right")]);
        let (started, events, _) = start(dir.path(), &mut prompt);
        assert!(started.is_ok());
        assert_eq!(events, ["waiting-for-unlock", "unlocked-at-start"]);
    }

    #[test]
    fn with_nothing_to_ask_with_the_start_fails() {
        let dir = tempfile::tempdir().unwrap();
        create(dir.path());
        let (started, ..) = start(dir.path(), &mut script(&[]));
        assert!(started.is_err());
    }

    #[test]
    fn a_handed_over_passphrase_gets_one_try() {
        let dir = tempfile::tempdir().unwrap();
        create(dir.path());
        let mut log = Log(Vec::new());
        let wrong = open(
            dir.path(),
            VaultChoice::Passphrase,
            &mut Given::new(Zeroizing::new("wrong".into())),
            &mut log,
            &mut |_| panic!("a handed-over passphrase never waits"),
        );
        assert!(wrong.is_err());
        let right = open(
            dir.path(),
            VaultChoice::Passphrase,
            &mut Given::new(Zeroizing::new("right".into())),
            &mut log,
            &mut |_| panic!("a handed-over passphrase never waits"),
        );
        assert!(right.is_ok());
    }

    #[test]
    fn a_handed_over_passphrase_creates_a_missing_vault() {
        let dir = tempfile::tempdir().unwrap();
        let created = open(
            dir.path(),
            VaultChoice::Passphrase,
            &mut Given::new(Zeroizing::new("new".into())),
            &mut Log(Vec::new()),
            &mut |_| panic!("a handed-over passphrase never waits"),
        );
        assert!(created.is_ok());
    }

    #[test]
    fn the_choice_follows_the_platform_the_flag_and_the_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(choose(dir.path(), true), VaultChoice::Passphrase);
        let bare = choose(dir.path(), false);
        assert_eq!(bare == VaultChoice::AutoOs, cfg!(windows));
        std::fs::write(dir.path().join(PASSPHRASE_VAULT_FILE), b"{}").unwrap();
        assert_eq!(choose(dir.path(), false), VaultChoice::Passphrase);
    }

    /// Rulings 5, 76 and 80 on Windows: an OS-rooted vault opens unattended
    /// until its lock persists, then waits for the passphrase, and opens
    /// unattended again once that unlock cleared the marker.
    #[cfg(windows)]
    #[test]
    fn a_persisted_lock_makes_the_os_vault_wait_for_its_passphrase() {
        let dir = tempfile::tempdir().unwrap();
        {
            let storage = SealedProfileStorage::open_auto_os(dir.path())
                .unwrap()
                .unwrap();
            let id = personae::vault::ProfileId("p".into());
            bootstrap::load_or_create_profile(&storage, &id).unwrap();
            storage.enroll_passphrase(b"right").unwrap();
        }
        assert_eq!(
            choose(dir.path(), true),
            VaultChoice::AutoOs,
            "the OS vault is there"
        );

        let (unattended, events, _) = start_as(dir.path(), VaultChoice::AutoOs, &mut script(&[]));
        assert!(unattended.is_ok());
        assert!(events.is_empty(), "no lock persisted: no prompt");

        castellan::custody::persist_lock(dir.path()).unwrap();
        let mut prompt = script(&[Some("wrong"), None, Some("right")]);
        let (started, events, slept) = start_as(dir.path(), VaultChoice::AutoOs, &mut prompt);
        let (opened, choice, second) = started.unwrap().into_parts();
        assert_eq!(choice, VaultChoice::AutoOs);
        assert!(matches!(second, Unlock::AutoOs));
        assert!(!opened.storage.is_locked());
        assert_eq!(
            events,
            [
                "waiting-for-unlock",
                "unlock-refused",
                "unlock-cancelled",
                "unlocked-at-start"
            ]
        );
        assert_eq!(slept, [backoff(0)]);
        assert!(
            !castellan::custody::lock_persisted(dir.path()),
            "the unlock cleared the marker"
        );
        drop(opened);

        let (again, events, _) = start_as(dir.path(), VaultChoice::AutoOs, &mut script(&[]));
        assert!(again.is_ok());
        assert!(events.is_empty());
    }

    #[test]
    fn backoff_doubles_to_thirty_seconds() {
        let waits: Vec<u64> = (0..8).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(waits, [1, 2, 4, 8, 16, 30, 30, 30]);
    }
}
