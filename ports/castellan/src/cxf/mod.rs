// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Two-step CXF import inside credential custody. Reading is memory-only;
//! committing requires an explicit persona for every exported account.

use crate::items::Payload;
use chatelaine::{CredentialKind, Item};
use personae::{PersonaId, ProfileId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use zeroize::Zeroizing;

mod commit;
mod erase;
mod parse;
pub use commit::CommitFailure;
mod ssh;
#[cfg(test)]
mod tests;

/// A deliberately secret-free summary, safe to show before import.
#[derive(Clone, Debug, Serialize)]
pub struct Preview {
    /// Identifies the exact plaintext source confirmed by a later commit.
    pub source_sha256: String,
    /// The exported accounts, each needing its own persona choice.
    pub accounts: Vec<AccountPreview>,
    /// Substitutions, refusals and format corrections, without field values.
    pub notices: Vec<Notice>,
}
/// One account and its proposed credential dispositions.
#[derive(Clone, Debug, Serialize)]
pub struct AccountPreview {
    /// Opaque CXF account id, used as the commit mapping key.
    pub source_id: String,
    /// Account label supplied by the exporter.
    pub username: String,
    /// Account email supplied by the exporter.
    pub email: String,
    /// Number of source items.
    pub items: usize,
    /// Counts by disposition: stored, quarantined, ssh, link or ignored.
    pub dispositions: BTreeMap<String, usize>,
}
/// A problem or substitution, located without exposing credential values.
#[derive(Clone, Debug, Serialize)]
pub struct Notice {
    /// The exported account.
    pub account: String,
    /// The source item, if applicable.
    pub item: Option<String>,
    /// The credential's zero-based position, if applicable.
    pub credential: Option<usize>,
    /// A fixed explanation; never a parser error containing input.
    pub reason: String,
}
/// Explicit custody destination for one account.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    /// The scoped item store's persona.
    pub persona: PersonaId,
    /// The identity profile that owns native SSH slots.
    pub profile: ProfileId,
    /// Unlock policy for newly imported SSH slots. A held slot keeps its
    /// existing policy. Omission follows the native import's Session default.
    #[serde(default = "session_policy")]
    pub ssh_unlock_policy: dramatis::intents::SshUnlockPolicyIntentV1,
}
fn session_policy() -> dramatis::intents::SshUnlockPolicyIntentV1 {
    dramatis::intents::SshUnlockPolicyIntentV1::Session
}
/// Completed writes and omissions. The source file remains the user's to delete.
#[derive(Debug, Serialize)]
pub struct Receipt {
    /// Number of indexed vault or quarantine items.
    pub items: usize,
    /// Number of native SSH imports, including held-key no-ops.
    pub ssh_keys: usize,
    /// Link omissions and other import notices.
    pub notices: Vec<Notice>,
    /// CXF source files contain plaintext credentials and should be deleted.
    pub delete_plaintext_source: bool,
}
/// An import failure whose text never contains a credential value.
#[derive(Debug)]
pub struct ImportError(pub(crate) &'static str);
impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ImportError {}

/// Parsed credentials held only inside the keeper. No `Debug` implementation.
pub struct Import {
    preview: Preview,
    accounts: Vec<PreparedAccount>,
}
struct PreparedAccount {
    id: String,
    items: Vec<PreparedItem>,
    collections: erase::Json,
}
struct PreparedItem {
    metadata: Item,
    credentials: Vec<PreparedCredential>,
}
struct PreparedCredential {
    kind: Option<CredentialKind>,
    payload: Option<Payload>,
    quarantine: bool,
    reference: Option<(Option<String>, String)>,
    ssh: Option<ssh_key::PrivateKey>,
}
impl Import {
    /// Parse a complete UTF-8 CXF document. Owns and zeroizes the input buffer.
    /// This operation never opens a vault, store, or filesystem path.
    pub fn read(bytes: Vec<u8>) -> Result<Self, ImportError> {
        parse::read(Zeroizing::new(bytes))
    }
    /// The secret-free account choices and proposed dispositions.
    pub fn preview(&self) -> &Preview {
        &self.preview
    }
}
