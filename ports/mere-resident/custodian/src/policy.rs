// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The installed custody policy (§5.3): which OS principals exist, what
//! each may ask for, under which profile and namespace, and whether the
//! custodian's own consent channel must approve. It is read once from the
//! service's root-owned configuration, never from a client. Grants are
//! deny-by-default and per principal; there is no global releasable-salt
//! list. A principal matched only by the desktop user's uid or SID is an
//! ordinary same-user requester: loading refuses an unattended signing or
//! release grant for it unless the configuration declares development mode,
//! which every status then reports. The one same-user consent channel is
//! Windows' pinned helper image ([`Matcher::WindowsImage`]), which may hold a
//! status grant and nothing else.

use custody_wire::Peer;
use serde::Deserialize;

/// The whole installed configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub vault: VaultConfig,
    pub principals: Vec<PrincipalRule>,
    /// How petitions reach the owner.
    pub consent: ConsentMode,
    /// The desktop account whose vault this is. Its uid or SID identifies
    /// only "some process of that user".
    pub owner: Owner,
    /// Same-user principals may hold unattended grants. Never for a
    /// protected deployment; reported in every status.
    #[serde(default)]
    pub development: bool,
}

/// Where the vault lives, in the custodian's own state directory.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultConfig {
    pub directory: std::path::PathBuf,
    /// The profile to open; absent, the remembered choice or the sole one.
    #[serde(default)]
    pub profile: Option<String>,
    /// The wallet (`identity/`) and credential catalog (`castellan/`) in
    /// Djinn's layout. Absent, wallet and credential operations refuse.
    #[serde(default)]
    pub data_root: Option<std::path::PathBuf>,
    /// Where the custodian serves the SSH agent, when it is served.
    #[serde(default)]
    pub agent_endpoint: Option<String>,
}

/// The vault owner's desktop account.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    #[serde(default)]
    pub unix_uid: Option<u32>,
    #[serde(default)]
    pub windows_sid: Option<String>,
}

/// Which channel decides petitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConsentMode {
    /// Nobody: every petition is refused.
    Deny,
    /// A session admitted as `consent_channel`.
    Channel,
    /// Linux polkit with an interactive authentication of the requester's
    /// session (`auth_self`), run by the custodian itself.
    Polkit,
}

/// One principal, matched on an OS fact.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalRule {
    pub name: String,
    #[serde(rename = "match")]
    pub matcher: Matcher,
    #[serde(default)]
    pub grants: Vec<Grant>,
    /// This principal is the trusted consent channel.
    #[serde(default)]
    pub consent_channel: bool,
}

/// The OS fact a principal is matched on.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Matcher {
    UnixUid(u32),
    WindowsSid(String),
    /// A process of `sid` whose main executable is `image`: the installed
    /// consent helper on Windows (`djinn-custodian consent`, run in the
    /// owner's session), where the installer leaves the executable writable
    /// only by administrators. Another process of the same user can still
    /// attach to it or start it under its own environment, so this is the
    /// same-user posture of a polkit authentication agent, not an OS-held
    /// identity; the plan records it as such. As the consent channel it may
    /// hold a status grant only, and it must be listed before the account's
    /// own rule, since the first matching rule wins.
    WindowsImage { sid: String, image: String },
    Signed { team: String, identifier: String },
}

impl Matcher {
    fn matches(&self, peer: &Peer) -> bool {
        match (self, peer) {
            (Self::UnixUid(want), Peer::Unix { uid, .. }) => want == uid,
            (Self::WindowsSid(want), Peer::Windows { sid, .. }) => want == sid,
            (Self::WindowsImage { sid: want, image: pinned }, Peer::Windows { sid, image: Some(image), .. }) => want == sid && same_image(pinned, image),
            (
                Self::Signed { team, identifier },
                Peer::Signed {
                    team: t,
                    identifier: i,
                },
            ) => team == t && identifier == i,
            _ => false,
        }
    }
}

/// Whether two Win32 paths name the same file, as the filesystem would:
/// case-insensitively, either separator, with or without the `\\?\` prefix.
fn same_image(pinned: &str, image: &str) -> bool {
    let normal = |path: &str| path.trim_start_matches(r"\\?\").replace('/', "\\").to_lowercase();
    !pinned.is_empty() && normal(pinned) == normal(image)
}

/// One granted operation.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Grant {
    Status,
    Lock,
    Unlock,
    Revoke {
        consent: Consent,
    },
    /// Release derived keys of `source` for salts the rules admit.
    Release {
        profile: ProfileScope,
        salts: Vec<SaltRule>,
        consent: Consent,
        #[serde(default)]
        source: SourceRule,
    },
    /// Attest derived keys of `source` for salts the rules admit.
    Attest {
        profile: ProfileScope,
        salts: Vec<SaltRule>,
        consent: Consent,
        #[serde(default)]
        source: SourceRule,
    },
    Sign {
        profile: ProfileScope,
        purpose: PurposeRule,
        consent: Consent,
    },
    /// The public read model.
    Snapshot,
    /// Profile intents, vault commands, passphrase change and SSH import.
    Manage {
        consent: Consent,
    },
    /// Wallet epochs, station grants and device revocation.
    Wallet {
        consent: Consent,
    },
    /// Retained-payload sealing under the open persona's epoch.
    Payload {
        consent: Consent,
    },
    /// The credential catalog: quarantine, older imports, review and CXF.
    Credentials {
        consent: Consent,
    },
}

/// Which root a release or attestation grant draws from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceRule {
    #[default]
    Persona,
    Wallet,
}

impl SourceRule {
    pub fn matches(self, source: custody_wire::Source) -> bool {
        matches!(
            (self, source),
            (Self::Persona, custody_wire::Source::Persona) | (Self::Wallet, custody_wire::Source::Wallet)
        )
    }
}

/// Which profile a grant covers.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileScope {
    /// Whichever profile the custodian has open.
    Current,
    Named(String),
    /// Any profile the vault holds, open or not (an application's own
    /// adopted persona, D13).
    Any,
}

impl ProfileScope {
    pub fn covers(&self, requested: &str, current: &str) -> bool {
        match self {
            Self::Current => requested == current,
            Self::Named(name) => name == requested && requested == current,
            Self::Any => !requested.is_empty(),
        }
    }
}

/// When the owner must approve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Consent {
    None,
    /// Once per lock generation for this principal and grant.
    PerGeneration,
    PerOperation,
}

impl Consent {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PerGeneration => "per-generation",
            Self::PerOperation => "per-operation",
        }
    }
}

/// One release namespace. An empty salt is the master's place and no rule
/// admits it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum SaltRule {
    /// Exactly this UTF-8 salt.
    Exact(String),
    /// Any longer salt with this UTF-8 prefix.
    Prefix(String),
    /// Exactly these bytes, as hex.
    ExactHex(String),
    /// Any longer salt beginning with these bytes, as hex.
    PrefixHex(String),
    /// Every delegation scope key in this insigne domain.
    Delegation(String),
    /// A family matcher: `reticulum`, `tucket-station` or `sennet-station`;
    /// or `djinn-door`, exactly the open profile's two door keys.
    Named(String),
}

/// Djinn's door keys for the profile whose master is `master`: the local
/// grant's delegation signer and the session signer bound to its local
/// network (`djinn::door::local_session::door_salts`, pinned against this by
/// Djinn's tests). Naming them exactly keeps every other `mere.graphshell`
/// delegation key sign-only.
pub fn door_salts(master: &[u8; 32]) -> [Vec<u8>; 2] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mere.graphshell/local-browser-network/v1");
    hasher.update(master);
    let network = notochord::NetworkId(*hasher.finalize().as_bytes());
    let scope = insigne::delegation::CapabilityScope {
        domain: "mere.graphshell".into(),
        resource: network.0.to_vec(),
        path_prefix: String::new(),
        actions: Default::default(),
    };
    [insigne::delegation::delegation_signing_salt(&scope), notochord::network_session_signing_salt(&network)]
}

fn hex(text: &str) -> Option<Vec<u8>> {
    (text.len() % 2 == 0)
        .then(|| (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok()).collect())
        .flatten()
}

fn prefixed(salt: &[u8], prefix: &[u8]) -> bool {
    salt.len() > prefix.len() && salt.starts_with(prefix)
}

impl SaltRule {
    /// [`Self::allows`], with the open profile's master for a rule that
    /// names that profile's own keys; `None` for any other profile.
    pub fn allows_under(&self, salt: &[u8], master: Option<&[u8; 32]>) -> bool {
        match (self, master) {
            (Self::Named(name), Some(master)) if name == "djinn-door" => !salt.is_empty() && door_salts(master).iter().any(|door| door == salt),
            _ => self.allows(salt),
        }
    }

    pub fn allows(&self, salt: &[u8]) -> bool {
        if salt.is_empty() {
            return false;
        }
        match self {
            Self::Exact(exact) => salt == exact.as_bytes(),
            Self::Prefix(prefix) => prefixed(salt, prefix.as_bytes()),
            Self::ExactHex(exact) => hex(exact).is_some_and(|exact| salt == exact.as_slice()),
            Self::PrefixHex(prefix) => hex(prefix).is_some_and(|prefix| prefixed(salt, &prefix)),
            Self::Delegation(domain) => prefixed(salt, &insigne::delegation::delegation_signing_prefix(domain)),
            Self::Named(name) => match name.as_str() {
                "reticulum" => personae::reticulum::is_reticulum_salt(salt),
                "tucket-station" => personae::tucket::is_station_salt(salt),
                "sennet-station" => personae::sennet::is_station_salt(salt),
                _ => false,
            },
        }
    }

    fn valid(&self) -> bool {
        match self {
            Self::Exact(text) | Self::Prefix(text) | Self::Delegation(text) => !text.is_empty(),
            Self::ExactHex(text) | Self::PrefixHex(text) => hex(text).is_some_and(|b| !b.is_empty()),
            Self::Named(name) => matches!(name.as_str(), "reticulum" | "tucket-station" | "sennet-station" | "djinn-door"),
        }
    }

    fn describe(&self) -> String {
        match self {
            Self::Exact(exact) => exact.clone(),
            Self::Prefix(prefix) => format!("{prefix}*"),
            Self::ExactHex(exact) => format!("hex:{exact}"),
            Self::PrefixHex(prefix) => format!("hex:{prefix}*"),
            Self::Delegation(domain) => format!("delegation:{domain}*"),
            Self::Named(name) => format!("named:{name}"),
        }
    }
}

/// What a signing grant is for.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "purpose", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PurposeRule {
    /// Delegation statements in exactly this domain.
    Delegation { audience: String },
    /// The profile master itself.
    Root,
    /// Arbitrary bytes under derived keys the rules admit: an application's
    /// own scoped act signed inside custody (D11), never the master.
    Derived {
        #[serde(default)]
        source: SourceRule,
        salts: Vec<SaltRule>,
    },
}

/// A principal selected for a connection.
#[derive(Clone, Debug)]
pub struct Principal {
    pub name: String,
    pub grants: Vec<Grant>,
    pub consent_channel: bool,
    /// Matched only by the owner's desktop account.
    pub same_user: bool,
}

/// Why a configuration was refused.
#[derive(Debug, PartialEq, Eq)]
pub struct PolicyError(pub String);

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Config {
    /// Parse and validate an installed configuration.
    pub fn parse(bytes: &[u8]) -> Result<Self, PolicyError> {
        let config: Self =
            serde_json::from_slice(bytes).map_err(|e| PolicyError(format!("configuration: {e}")))?;
        config.validate()?;
        Ok(config)
    }

    fn same_user(&self, matcher: &Matcher) -> bool {
        match matcher {
            Matcher::UnixUid(uid) => self.owner.unix_uid == Some(*uid),
            Matcher::WindowsSid(sid) | Matcher::WindowsImage { sid, .. } => self.owner.windows_sid.as_deref() == Some(sid),
            Matcher::Signed { .. } => false,
        }
    }

    fn validate(&self) -> Result<(), PolicyError> {
        if !self.vault.directory.is_absolute() {
            return Err(PolicyError("the vault directory must be absolute".into()));
        }
        let mut names = std::collections::BTreeSet::new();
        for rule in &self.principals {
            if rule.name.is_empty() || rule.name.len() > 64 || !names.insert(&rule.name) {
                return Err(PolicyError(format!("principal name {:?} is empty, too long or repeated", rule.name)));
            }
            let same_user = self.same_user(&rule.matcher);
            let pinned = match &rule.matcher {
                Matcher::WindowsImage { image, .. } => {
                    let absolute = image.get(1..2) == Some(":") || image.starts_with('\\');
                    if !absolute {
                        return Err(PolicyError(format!("{}: the pinned image must be an absolute Win32 path", rule.name)));
                    }
                    true
                },
                _ => false,
            };
            if rule.consent_channel && same_user && !pinned {
                return Err(PolicyError(format!(
                    "{}: the desktop account cannot be the trusted consent channel; any of its processes would match",
                    rule.name
                )));
            }
            if rule.consent_channel && same_user && rule.grants.iter().any(|grant| !matches!(grant, Grant::Status)) {
                return Err(PolicyError(format!(
                    "{}: a consent channel pinned to the desktop account's executable may hold only a status grant",
                    rule.name
                )));
            }
            for grant in &rule.grants {
                match grant {
                    Grant::Sign {
                        purpose: PurposeRule::Root,
                        consent,
                        ..
                    } if *consent != Consent::PerOperation => {
                        return Err(PolicyError(format!("{}: a root signature needs per-operation consent", rule.name)));
                    },
                    Grant::Sign { consent: Consent::None, .. } | Grant::Release { consent: Consent::None, .. }
                        if same_user && !self.development =>
                    {
                        return Err(PolicyError(format!(
                            "{}: a same-user principal cannot sign or release unattended outside development mode",
                            rule.name
                        )));
                    },
                    Grant::Revoke { consent: Consent::None } if same_user => {
                        return Err(PolicyError(format!("{}: revocation by a same-user principal needs consent", rule.name)));
                    },
                    Grant::Release { salts, .. } | Grant::Attest { salts, .. } | Grant::Sign { purpose: PurposeRule::Derived { salts, .. }, .. }
                        if salts.is_empty() || !salts.iter().all(SaltRule::valid) =>
                    {
                        return Err(PolicyError(format!("{}: a grant names no salts or an invalid salt rule", rule.name)));
                    },
                    Grant::Manage { consent: Consent::None } | Grant::Wallet { consent: Consent::None } | Grant::Payload { consent: Consent::None } | Grant::Credentials { consent: Consent::None } | Grant::Attest { consent: Consent::None, .. }
                        if same_user && !self.development =>
                    {
                        return Err(PolicyError(format!(
                            "{}: a same-user principal cannot hold an unattended custody grant outside development mode",
                            rule.name
                        )));
                    },
                    Grant::Sign {
                        purpose: PurposeRule::Delegation { audience },
                        ..
                    } if audience.is_empty() => {
                        return Err(PolicyError(format!("{}: a delegation grant names no audience", rule.name)));
                    },
                    _ => {},
                }
            }
        }
        Ok(())
    }

    /// Every signed identity a principal is matched on.
    pub fn signed_identities(&self) -> Vec<custody_wire::SignedIdentity> {
        self.principals
            .iter()
            .filter_map(|rule| match &rule.matcher {
                Matcher::Signed { team, identifier } => Some(custody_wire::SignedIdentity {
                    team: team.clone(),
                    identifier: identifier.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    /// The principal a peer is, if any. The first matching rule wins.
    pub fn principal(&self, peer: &Peer) -> Option<Principal> {
        self.principals
            .iter()
            .find(|rule| rule.matcher.matches(peer))
            .map(|rule| Principal {
                name: rule.name.clone(),
                grants: rule.grants.clone(),
                consent_channel: rule.consent_channel,
                same_user: self.same_user(&rule.matcher),
            })
    }
}

impl Principal {
    /// The scopes a session reports.
    pub fn scopes(&self) -> Vec<custody_wire::contract::Scope> {
        self.grants
            .iter()
            .map(|grant| {
                let (profile, operation, namespace, consent) = match grant {
                    Grant::Status => (String::new(), "status", String::new(), Consent::None),
                    Grant::Lock => (String::new(), "lock", String::new(), Consent::None),
                    Grant::Unlock => (String::new(), "unlock", String::new(), Consent::None),
                    Grant::Revoke { consent } => (String::new(), "revoke", String::new(), *consent),
                    Grant::Release {
                        profile,
                        salts,
                        consent,
                        ..
                    } => (
                        profile_name(profile),
                        "release",
                        salts.iter().map(SaltRule::describe).collect::<Vec<_>>().join(","),
                        *consent,
                    ),
                    Grant::Sign {
                        profile,
                        purpose,
                        consent,
                    } => (
                        profile_name(profile),
                        "sign",
                        match purpose {
                            PurposeRule::Delegation { audience } => format!("delegation:{audience}"),
                            PurposeRule::Root => "root".into(),
                            PurposeRule::Derived { salts, .. } => salts.iter().map(SaltRule::describe).collect::<Vec<_>>().join(","),
                        },
                        *consent,
                    ),
                    Grant::Attest { profile, salts, consent, .. } => (
                        profile_name(profile),
                        "attest",
                        salts.iter().map(SaltRule::describe).collect::<Vec<_>>().join(","),
                        *consent,
                    ),
                    Grant::Snapshot => (String::new(), "snapshot", String::new(), Consent::None),
                    Grant::Manage { consent } => (String::new(), "manage", String::new(), *consent),
                    Grant::Wallet { consent } => (String::new(), "wallet", String::new(), *consent),
                    Grant::Payload { consent } => (String::new(), "payload", String::new(), *consent),
                    Grant::Credentials { consent } => (String::new(), "credentials", String::new(), *consent),
                };
                custody_wire::contract::Scope {
                    profile,
                    operation: operation.into(),
                    namespace,
                    consent: consent.name().into(),
                }
            })
            .collect()
    }
}

fn profile_name(scope: &ProfileScope) -> String {
    match scope {
        ProfileScope::Current => "current".into(),
        ProfileScope::Named(name) => name.clone(),
        ProfileScope::Any => "any".into(),
    }
}
