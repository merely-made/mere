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
//! which every status then reports.

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
    Signed { team: String, identifier: String },
}

impl Matcher {
    fn matches(&self, peer: &Peer) -> bool {
        match (self, peer) {
            (Self::UnixUid(want), Peer::Unix { uid, .. }) => want == uid,
            (Self::WindowsSid(want), Peer::Windows { sid, .. }) => want == sid,
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
    Release {
        profile: ProfileScope,
        salts: Vec<SaltRule>,
        consent: Consent,
    },
    Sign {
        profile: ProfileScope,
        purpose: PurposeRule,
        consent: Consent,
    },
}

/// Which profile a grant covers.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileScope {
    /// Whichever profile the custodian has open.
    Current,
    Named(String),
}

impl ProfileScope {
    pub fn covers(&self, requested: &str, current: &str) -> bool {
        requested == current
            && match self {
                Self::Current => true,
                Self::Named(name) => name == requested,
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

/// One release namespace.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum SaltRule {
    /// Exactly this UTF-8 salt.
    Exact(String),
    /// Any longer salt with this UTF-8 prefix.
    Prefix(String),
}

impl SaltRule {
    pub fn allows(&self, salt: &[u8]) -> bool {
        match self {
            Self::Exact(exact) => !salt.is_empty() && salt == exact.as_bytes(),
            Self::Prefix(prefix) => salt.len() > prefix.len() && salt.starts_with(prefix.as_bytes()),
        }
    }

    fn describe(&self) -> String {
        match self {
            Self::Exact(exact) => exact.clone(),
            Self::Prefix(prefix) => format!("{prefix}*"),
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
            Matcher::WindowsSid(sid) => self.owner.windows_sid.as_deref() == Some(sid),
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
            if rule.consent_channel && same_user {
                return Err(PolicyError(format!(
                    "{}: the desktop account cannot be the trusted consent channel; any of its processes would match",
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
                    Grant::Release { salts, .. } if salts.is_empty() => {
                        return Err(PolicyError(format!("{}: a release grant names no salts", rule.name)));
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
                        },
                        *consent,
                    ),
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
    }
}
