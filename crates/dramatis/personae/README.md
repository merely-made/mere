# personae

The identity and carry layer for the Merely ecosystem (mere, isometry, hocket,
woodshed). A person has *personae*, plural: a work face, a research face, a
burner. This crate is the register of them and the root of trust they derive
from, a master Ed25519 keypair with deterministic per-protocol key derivation,
sealed-record storage under keys the caller holds, and the issuing side of the
signed capability-delegation grammar that
[insigne](https://crates.io/crates/insigne) defines.

It holds no secret custody. The vault, its passphrase and OS-sealed storages,
the unlock ladder and the SSH agent moved to castellan, the crate only the
resident (djinn) links (mere's dramatis repo plan, DR-B, rulings D3, D9, D22).

Promoted from mere's `persona/identity`. Edition 2024, pure-Rust crypto
(`ed25519-dalek`, `blake3`, `chacha20poly1305`).

```rust
use personae::{IdentityProvider, InMemoryProvider};

let provider = InMemoryProvider::random();
let cabal = provider.derive_keypair(b"a-32-byte-salt-for-this-cabal...").unwrap();
let sig = cabal.sign(b"hello");
assert!(cabal.public_key().verify(b"hello", &sig));
```

Derivation is `BLAKE3-keyed(master_seed, salt)` to an Ed25519 seed. The master
secret never leaves the `IdentityProvider`; callers get only the derived
keypair. `IdentityProvider::attest_derived_key` returns a
`DerivedKeyAttestation` proving a derived key was authorized by the master
identity, so application traffic is never signed with the master key directly.

## Modules

| Module | Contents |
| --- | --- |
| root | `PersonaId`, `IdentityError`, `Ed25519Keypair`, `Ed25519PublicKey`, `Ed25519Signature`, `DerivedKeypair`, `RetainedKeys`, `VERSION`, `STAGE`; and, from the private `provider` module, `IdentityProvider`, `InMemoryProvider`, `AttestationKeys` (attestation keys as `Ed25519PublicKey`), `attest_derived_key` |
| `vault` | The plain types: `ProfileId`, `ProtocolKey`, `CredentialLineage`, `UnlockTier`, `ProfileSummary`, `PublicProfile`, `SlotSummary` |
| `startup_unlock` | `StartupUnlockMode`, the device setting; the loaders that act on it are castellan's |
| `seal` | `seal_bytes` / `unseal_bytes`, XChaCha20-Poly1305 with a prepended random nonce |
| `sealed_record_storage` | `SealedRecordStorage`, one sealed typed serde value per path, under a key the caller holds |
| `zeroizing_json` | serde_json for secret plaintext without freed copies (castellan's storages use it) |
| `delegation` | Issuing: `Issue` (`issue` for both signed types) and `DelegationError`. Import the grammar directly from `insigne::delegation`: `DelegationCertificate` / `SignedDelegationCertificate`, `DelegationRevocation` / `SignedDelegationRevocation`, `DelegationId`, `DelegationParent`, `CapabilityScope`, `delegation_signing_salt`, `path_covers` |
| `carry` | The wallet carry model: devices, grants, epochs, refs |
| `signing` | The plain signing records: `SigningRequest`, `SigningPolicy`, `SigningDecision`, `SigningAuthorization`, `SigningRecord`. The approval broker is castellan's |
| `ssh_ca` | `SshCertAuthority`, `UserCertRequest`, `HostCertRequest`, `CertMintError`, `self_grant`, `key_id_for`, `ssh_ca_salt`, `MAX_CERT_TTL_MS`, `SSH_CA_MOD_ID`. The delegation grammar projected into OpenSSH certificates. Feature `ssh` |
| `ssh_face` | `FacePolicy` (`work`/`research`/`burner`), `policy_key`, `SSH_FACE_MOD_ID`. What one face may do over SSH; reading and writing it in a profile is castellan's. Feature `ssh` |
| `ssh_krl` | `RevocationLedger`, `RevokedDevice`, `ledger_key`. Feature `ssh` |
| `enroll` | `user_trust_line`, `user_install_script`, `system_sshd_snippet`, `known_hosts_line`, `device_id_for_host`, `local_device_id`, `local_host_name`, `split_target`, `ENROLLMENT_MARKER`. Feature `ssh` |

## Features

| Feature | Pulls | Enables |
| --- | --- | --- |
| `ssh` | `ssh-key`, `gethostname` | `ssh_ca`, `ssh_face`, `ssh_krl`, `enroll` |

The `personae-agent` and `personae-vault` bins, and their install scripts,
moved to castellan with the agent (castellan's `agent` feature).

## Scope

The carry layer (device roster, capability grants, private-epoch history, the
portable-persona spine that moves a persona and its data between devices) folds
in as it lifts out of mere's `session-runtime`. This crate subsumes what was
going to be named `signet`: one name for the faces and how they carry.

Design notes live in `design_docs/` beside this file.

## License

MPL-2.0 (see LICENSE). The name is the plural of *persona*,
unrelated to Mozilla's discontinued Persona / BrowserID.
