// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Narrow, expiring RemoteAuth grants for sited Reticulum stations.
//!
//! Moved here from castellan in DR-C (dramatis repo plan, D17): the station
//! policy is public grant vocabulary, so it lives beside the wallet's public
//! records. Signing the grant needs the wallet seed, which only djinn holds,
//! so [`SitedStationGrant::issue_with`] takes the custodian's signing step as
//! a function and keeps every policy check here.
//!
//! A station grant is a `DeviceGrantSet` holding exactly one device-scoped
//! `SignedDelegationCertificate`. This adapter fixes the station policy over
//! it: one derived station signing key, `transport.egress` only, no persona
//! authority at all, no subdelegation, and a mandatory hard expiry.
//!
//! The reconciliation this module once deferred is settled: the device grant
//! *is* a delegation certificate now, so `no-subdelegation` is read as
//! `remaining_delegation_depth == 0` and enforced by the grammar rather than
//! asserted as a string atom nobody checked.

use std::fmt;
use std::path::Path;

use identity::carry::DeviceGrantSet;
use insigne::CheckFault;

use crate::{
    DeviceExposure, DeviceGrantError, DeviceId, DeviceMode, DevicePublicKey, RemoteAuthGrantSpec,
    certificate_device_id, device_grant_set_ref, device_is_fully_revoked, load_device_grant_set,
    load_device_roster,
};

/// The sole capability a sited station can receive.
pub const TRANSPORT_EGRESS_SCOPE: &str = "transport.egress";
/// The compulsory attenuation for a sited station grant.
pub const NO_SUBDELEGATION_ATTENUATION: &str = "no-subdelegation";

/// Inputs for issuing one sited-station RemoteAuth grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SitedStationGrantRequest {
    device_id: DeviceId,
    station_ed25519_public_key: [u8; 32],
    label: String,
    issued_at_ms: u64,
    expires_at_ms: u64,
}

impl SitedStationGrantRequest {
    /// Create a request whose expiry is strictly after its issuance time.
    pub fn new(
        device_id: DeviceId,
        station_ed25519_public_key: [u8; 32],
        label: impl Into<String>,
        issued_at_ms: u64,
        expires_at_ms: u64,
    ) -> Result<Self, SitedStationGrantError> {
        let label = label.into();
        if label.trim().is_empty() {
            return Err(SitedStationGrantError::EmptyLabel);
        }
        if expires_at_ms <= issued_at_ms {
            return Err(SitedStationGrantError::InvalidExpiryWindow {
                issued_at_ms,
                expires_at_ms,
            });
        }
        Ok(Self {
            device_id,
            station_ed25519_public_key,
            label,
            issued_at_ms,
            expires_at_ms,
        })
    }

    /// The station this request commissions.
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    /// The station's Ed25519 key the grant will authorize.
    pub fn station_ed25519_public_key(&self) -> [u8; 32] {
        self.station_ed25519_public_key
    }

    /// The label the roster shows.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// When the grant is issued.
    pub fn issued_at_ms(&self) -> u64 {
        self.issued_at_ms
    }

    /// The grant's mandatory expiry.
    pub fn expires_at_ms(&self) -> u64 {
        self.expires_at_ms
    }

    /// The RemoteAuth spec the custodian signs: one device certificate,
    /// `transport.egress` only, no persona, no subdelegation.
    pub fn remote_auth_spec(&self) -> RemoteAuthGrantSpec {
        RemoteAuthGrantSpec {
            device_id: self.device_id,
            delegatee_pubkey: DevicePublicKey(self.station_ed25519_public_key),
            label: self.label.clone(),
            exposure: DeviceExposure::ExposedEgress,
            issued_at_ms: self.issued_at_ms,
            expires_at_ms: Some(self.expires_at_ms),
            personas: Vec::new(),
            scopes: vec![TRANSPORT_EGRESS_SCOPE.into()],
            attenuations: vec![NO_SUBDELEGATION_ATTENUATION.into()],
            wrapped_private_epochs: Vec::new(),
        }
    }
}

/// A signed RemoteAuth envelope that passed the sited-station policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SitedStationGrant {
    grant: DeviceGrantSet,
}

impl SitedStationGrant {
    /// Issue and persist a narrow grant through the custodian's wallet.
    ///
    /// `issuer_public_key` must be the Persona public key that derived the
    /// station credential; `wallet_public_key` is the custodian's unlocked
    /// wallet root, so a different Persona cannot commission the station.
    /// `sign` is the custodian's issue step (castellan's
    /// `issue_remote_auth_device_grant`, inside djinn), which persists the
    /// grant and its roster record.
    pub fn issue_with(
        data_root: &Path,
        issuer_public_key: [u8; 32],
        wallet_public_key: [u8; 32],
        request: SitedStationGrantRequest,
        sign: impl FnOnce(&RemoteAuthGrantSpec) -> std::io::Result<DeviceGrantSet>,
    ) -> Result<Self, SitedStationGrantError> {
        if wallet_public_key != issuer_public_key {
            return Err(SitedStationGrantError::IssuerMismatch);
        }

        if let Some(existing) =
            Some(load_device_grant_set(data_root, request.device_id)?).filter(|set| !set.is_empty())
        {
            let existing = Self::from_signed(existing)?;
            if request.expires_at_ms <= existing.expires_at_ms() {
                return Err(SitedStationGrantError::NonExtendingRenewal {
                    device_id: request.device_id,
                    existing_expires_at_ms: existing.expires_at_ms(),
                    requested_expires_at_ms: request.expires_at_ms,
                });
            }
        }

        let grant = sign(&request.remote_auth_spec())?;
        Self::from_signed(grant)
    }

    /// Check a signed envelope's static station policy and signature.
    pub fn from_signed(grant: DeviceGrantSet) -> Result<Self, SitedStationGrantError> {
        let station_grant = Self { grant };
        station_grant.validate_policy()?;
        Ok(station_grant)
    }

    /// Load the active grant for a station, including host-side revocation.
    ///
    /// Expiry is fail-closed: `now_ms == expires_at_ms` is already expired.
    pub fn load_active(
        data_root: &Path,
        device_id: DeviceId,
        station_ed25519_public_key: [u8; 32],
        now_ms: u64,
    ) -> Result<Self, SitedStationGrantError> {
        let grant = Some(load_device_grant_set(data_root, device_id)?)
            .filter(|set| !set.is_empty())
            .ok_or(SitedStationGrantError::MissingGrant { device_id })?;
        let station_grant = Self::from_signed(grant)?;

        // Both records are consulted, because they answer different questions.
        // The roster list is this host's own index of what it revoked. The
        // ledger holds signed statements, which is what a revocation arriving
        // from a peer looks like: honouring only the list would mean a station
        // withdrawn elsewhere kept its authority here.
        if device_is_fully_revoked(data_root, device_id)? {
            return Err(SitedStationGrantError::Revoked { device_id });
        }
        let roster = load_device_roster(data_root)?.ok_or(SitedStationGrantError::MissingRoster)?;
        if roster.revoked.contains(&device_id) {
            return Err(SitedStationGrantError::Revoked { device_id });
        }
        let device = roster
            .devices
            .iter()
            .find(|device| device.device_id == device_id)
            .ok_or(SitedStationGrantError::UnknownDevice { device_id })?;
        if device.mode != DeviceMode::RemoteAuth {
            return Err(SitedStationGrantError::NotRemoteAuth { device_id });
        }
        if device.exposure != DeviceExposure::ExposedEgress {
            return Err(SitedStationGrantError::NotExposedEgress { device_id });
        }
        if device.device_pubkey != DevicePublicKey(station_ed25519_public_key) {
            return Err(SitedStationGrantError::StationKeyMismatch { device_id });
        }
        let expected_ref = device_grant_set_ref(&station_grant.grant);
        if device.grant_ref != Some(expected_ref) {
            return Err(SitedStationGrantError::RosterGrantMismatch { device_id });
        }

        station_grant.authorize_at(device_id, station_ed25519_public_key, now_ms)?;
        Ok(station_grant)
    }

    /// Check that this grant is for the expected station and still live.
    ///
    /// This does not consult host storage. Use [`Self::load_active`] when a
    /// host must also reject revoked or stale roster entries.
    pub fn authorize_at(
        &self,
        device_id: DeviceId,
        station_ed25519_public_key: [u8; 32],
        now_ms: u64,
    ) -> Result<(), SitedStationGrantError> {
        self.validate_policy()?;
        if self.device_id() != device_id {
            return Err(SitedStationGrantError::DeviceMismatch {
                expected: device_id,
                actual: self.device_id(),
            });
        }
        if self.station_ed25519_public_key() != station_ed25519_public_key {
            return Err(SitedStationGrantError::StationKeyMismatch { device_id });
        }
        let expires_at_ms = self.expires_at_ms();
        if now_ms >= expires_at_ms {
            return Err(SitedStationGrantError::Expired {
                device_id,
                expires_at_ms,
                now_ms,
            });
        }
        Ok(())
    }

    /// The single device-scoped certificate a station grant is made of.
    ///
    /// `validate_policy` guarantees it is present, so the accessors below may
    /// rely on it. A station carries device authority and nothing else: no
    /// persona ever delegates to it.
    fn certificate(&self) -> &insigne::delegation::SignedDelegationCertificate {
        self.grant
            .device
            .as_ref()
            .expect("validate_policy requires a device certificate")
    }

    /// Borrow the signed wallet envelope for transport or durable storage.
    pub fn signed(&self) -> &DeviceGrantSet {
        &self.grant
    }

    /// When this grant was issued.
    pub fn issued_at_ms(&self) -> u64 {
        self.certificate().certificate.issued_at_ms
    }

    /// The mandatory fail-closed expiry of this grant.
    pub fn expires_at_ms(&self) -> u64 {
        self.certificate()
            .certificate
            .expires_at_ms
            .expect("validate_policy requires a station-grant expiry")
    }

    /// The RemoteAuth device this grant is bound to.
    pub fn device_id(&self) -> DeviceId {
        certificate_device_id(self.certificate())
            .expect("validate_policy requires a device-scoped certificate")
    }

    /// The Reticulum Ed25519 station key this grant authorizes.
    pub fn station_ed25519_public_key(&self) -> [u8; 32] {
        self.certificate().certificate.subject
    }

    /// The Persona root that signed this grant.
    pub fn issuer_public_key(&self) -> [u8; 32] {
        self.certificate().certificate.issuer
    }

    fn validate_policy(&self) -> Result<(), SitedStationGrantError> {
        let Some(certificate) = self.grant.device.as_ref() else {
            return Err(SitedStationGrantError::ScopeViolation);
        };
        certificate
            .check()
            .map_err(SitedStationGrantError::InvalidSignature)?;
        // A station holds device authority and nothing else. A persona
        // certificate in the set would mean some persona had delegated to an
        // unattended radio, which is exactly what this port refuses.
        if !self.grant.personas.is_empty() {
            return Err(SitedStationGrantError::PersonaAuthority);
        }
        if certificate_device_id(certificate).is_none() {
            return Err(SitedStationGrantError::ScopeViolation);
        }
        let scope = &certificate.certificate.scope;
        if scope.actions.len() != 1 || !scope.actions.contains(TRANSPORT_EGRESS_SCOPE) {
            return Err(SitedStationGrantError::ScopeViolation);
        }
        // `no-subdelegation` used to be a string atom nobody enforced. The
        // grammar enforces it now, so this reads the depth instead.
        if certificate.certificate.remaining_delegation_depth != 0 {
            return Err(SitedStationGrantError::AttenuationViolation);
        }
        let expires_at_ms = certificate
            .certificate
            .expires_at_ms
            .ok_or(SitedStationGrantError::MissingExpiry)?;
        if expires_at_ms <= certificate.certificate.issued_at_ms {
            return Err(SitedStationGrantError::InvalidExpiryWindow {
                issued_at_ms: certificate.certificate.issued_at_ms,
                expires_at_ms,
            });
        }
        Ok(())
    }
}

/// Why a sited-station grant could not be issued or used.
#[derive(Debug)]
pub enum SitedStationGrantError {
    /// The host wallet store could not be read or written.
    Storage(std::io::Error),
    /// The signed envelope could not be decoded, signed, or content-addressed.
    Envelope(DeviceGrantError),
    /// The commissioning label was blank.
    EmptyLabel,
    /// The expiry did not fall strictly after the issuance time.
    InvalidExpiryWindow {
        /// The grant's issuance time.
        issued_at_ms: u64,
        /// The grant's requested expiry time.
        expires_at_ms: u64,
    },
    /// The host wallet has no unlocked root to issue a grant with.
    WalletLocked,
    /// The credential's Persona did not match the host wallet root.
    IssuerMismatch,
    /// A replacement grant did not extend the existing live window.
    NonExtendingRenewal {
        /// The station whose grant was being replaced.
        device_id: DeviceId,
        /// The existing grant's expiry time.
        existing_expires_at_ms: u64,
        /// The requested replacement expiry time.
        requested_expires_at_ms: u64,
    },
    /// The host store has no signed grant for this device.
    MissingGrant {
        /// The requested device.
        device_id: DeviceId,
    },
    /// The host store has no device roster.
    MissingRoster,
    /// The requested device is absent from the host roster.
    UnknownDevice {
        /// The requested device.
        device_id: DeviceId,
    },
    /// The host has revoked this device.
    Revoked {
        /// The revoked device.
        device_id: DeviceId,
    },
    /// The roster records a different device mode.
    NotRemoteAuth {
        /// The incorrectly configured device.
        device_id: DeviceId,
    },
    /// The roster does not allow the device to act as egress.
    NotExposedEgress {
        /// The incorrectly configured device.
        device_id: DeviceId,
    },
    /// The signed envelope names a different device than expected.
    DeviceMismatch {
        /// The device the caller expected.
        expected: DeviceId,
        /// The device named by the envelope.
        actual: DeviceId,
    },
    /// The envelope or roster names a different station signing key.
    StationKeyMismatch {
        /// The affected device.
        device_id: DeviceId,
    },
    /// The roster does not point at the currently stored signed envelope.
    RosterGrantMismatch {
        /// The affected device.
        device_id: DeviceId,
    },
    /// The signed envelope did not check; the fault says which step failed.
    InvalidSignature(CheckFault),
    /// The envelope grants any Persona authority.
    PersonaAuthority,
    /// The envelope carries private-lane epoch material.
    PrivateEpochMaterial,
    /// The envelope grants anything other than `transport.egress`.
    ScopeViolation,
    /// The envelope permits any attenuation other than `no-subdelegation`.
    AttenuationViolation,
    /// The envelope has no expiry.
    MissingExpiry,
    /// The envelope's expiry has passed or arrived.
    Expired {
        /// The expired device.
        device_id: DeviceId,
        /// The strict expiry instant.
        expires_at_ms: u64,
        /// The instant used for evaluation.
        now_ms: u64,
    },
}

impl From<std::io::Error> for SitedStationGrantError {
    fn from(error: std::io::Error) -> Self {
        Self::Storage(error)
    }
}

impl From<DeviceGrantError> for SitedStationGrantError {
    fn from(error: DeviceGrantError) -> Self {
        Self::Envelope(error)
    }
}

impl fmt::Display for SitedStationGrantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "sited station wallet storage failed: {error}"),
            Self::Envelope(error) => write!(f, "sited station grant envelope failed: {error}"),
            Self::EmptyLabel => f.write_str("a sited station label must not be blank"),
            Self::InvalidExpiryWindow {
                issued_at_ms,
                expires_at_ms,
            } => write!(
                f,
                "sited station expiry {expires_at_ms} must be after issuance {issued_at_ms}"
            ),
            Self::WalletLocked => f.write_str("the host wallet root is locked or absent"),
            Self::IssuerMismatch => f.write_str(
                "the station credential was derived by a different Persona than the host wallet",
            ),
            Self::NonExtendingRenewal {
                device_id,
                existing_expires_at_ms,
                requested_expires_at_ms,
            } => write!(
                f,
                "replacement grant for sited station {} expires at {requested_expires_at_ms}, which does not extend existing expiry {existing_expires_at_ms}",
                device_id.as_uuid()
            ),
            Self::MissingGrant { device_id } => {
                write!(f, "sited station grant missing for {}", device_id.as_uuid())
            },
            Self::MissingRoster => f.write_str("the host device roster is missing"),
            Self::UnknownDevice { device_id } => {
                write!(
                    f,
                    "sited station device {} is absent from the host roster",
                    device_id.as_uuid()
                )
            },
            Self::Revoked { device_id } => {
                write!(f, "sited station device {} is revoked", device_id.as_uuid())
            },
            Self::NotRemoteAuth { device_id } => write!(
                f,
                "sited station device {} is not a RemoteAuth device",
                device_id.as_uuid()
            ),
            Self::NotExposedEgress { device_id } => write!(
                f,
                "sited station device {} is not an exposed egress",
                device_id.as_uuid()
            ),
            Self::DeviceMismatch { expected, actual } => write!(
                f,
                "sited station grant targets {}, not expected device {}",
                actual.as_uuid(),
                expected.as_uuid()
            ),
            Self::StationKeyMismatch { device_id } => write!(
                f,
                "sited station signing key does not match device {}",
                device_id.as_uuid()
            ),
            Self::RosterGrantMismatch { device_id } => write!(
                f,
                "sited station roster reference does not match device {} grant",
                device_id.as_uuid()
            ),
            Self::InvalidSignature(fault) => {
                write!(f, "sited station grant signature is invalid: {fault}")
            },
            Self::PersonaAuthority => {
                f.write_str("sited station grants must not authorize any persona")
            },
            Self::PrivateEpochMaterial => {
                f.write_str("sited station grants must not carry private epoch material")
            },
            Self::ScopeViolation => {
                f.write_str("sited station grants must contain only the transport.egress scope")
            },
            Self::AttenuationViolation => f.write_str(
                "sited station grants must contain only the no-subdelegation attenuation",
            ),
            Self::MissingExpiry => f.write_str("sited station grants require an expiry"),
            Self::Expired {
                device_id,
                expires_at_ms,
                now_ms,
            } => write!(
                f,
                "sited station device {} expired at {expires_at_ms}; evaluated at {now_ms}",
                device_id.as_uuid()
            ),
        }
    }
}

impl std::error::Error for SitedStationGrantError {}

#[cfg(test)]
mod tests {
    use identity::{IdentityProvider, InMemoryProvider};

    use super::*;

    #[test]
    fn a_signed_grant_with_extra_authority_is_not_a_sited_station_grant() {
        let seed = [0x51; 32];
        let issuer = InMemoryProvider::from_seed(seed);
        let station = issuer.derive_keypair(b"sited-station-grant-test").unwrap();
        let device_id = DeviceId::new();
        // A grant carrying identity.act as well: the extra action lands on a
        // persona certificate, so the set has persona authority in it and the
        // station policy must refuse it.
        let signed = identity::carry::issue_device_grant_set(
            seed,
            device_id,
            DevicePublicKey::from(station.public_key()),
            &[TRANSPORT_EGRESS_SCOPE, "identity.act"],
            &[identity::PersonaId::new()],
            100,
            100,
        )
        .unwrap();
        let error = SitedStationGrant::from_signed(signed).unwrap_err();

        assert!(matches!(error, SitedStationGrantError::PersonaAuthority));
    }

    /// The custodian's signing step, as djinn performs it, over a fixed seed.
    fn sign_with(
        root: &Path,
        seed: [u8; 32],
    ) -> impl FnOnce(&RemoteAuthGrantSpec) -> std::io::Result<DeviceGrantSet> + '_ {
        move |spec| {
            let actions: Vec<&str> = spec.scopes.iter().map(String::as_str).collect();
            let set = identity::carry::issue_device_grant_set(
                seed,
                spec.device_id,
                spec.delegatee_pubkey,
                &actions,
                &spec.personas,
                spec.expires_at_ms.unwrap() - spec.issued_at_ms,
                spec.issued_at_ms,
            )
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            crate::save_device_grant_set(root, spec.device_id, &set)?;
            Ok(set)
        }
    }

    #[test]
    fn a_station_grant_can_only_be_renewed_with_a_later_expiry() {
        let root = tempfile::tempdir().unwrap();
        let seed = [0x52; 32];
        let issuer = InMemoryProvider::from_seed(seed);
        let master = issuer.master_public_key().to_bytes();
        let device_id = DeviceId::new();
        let key = issuer
            .derive_keypair(b"sited-station-renewal-test")
            .unwrap();
        let first = SitedStationGrantRequest::new(
            device_id,
            key.public_key().to_bytes(),
            "Ridge north",
            100,
            200,
        )
        .unwrap();
        SitedStationGrant::issue_with(
            root.path(),
            master,
            master,
            first,
            sign_with(root.path(), seed),
        )
        .unwrap();

        let replacement = SitedStationGrantRequest::new(
            device_id,
            key.public_key().to_bytes(),
            "Ridge north",
            150,
            200,
        )
        .unwrap();
        let error = SitedStationGrant::issue_with(
            root.path(),
            master,
            master,
            replacement,
            sign_with(root.path(), seed),
        )
        .unwrap_err();

        assert!(matches!(
            error,
            SitedStationGrantError::NonExtendingRenewal {
                existing_expires_at_ms: 200,
                requested_expires_at_ms: 200,
                ..
            }
        ));
    }

    #[test]
    fn another_personas_wallet_cannot_commission_the_station() {
        let root = tempfile::tempdir().unwrap();
        let request =
            SitedStationGrantRequest::new(DeviceId::new(), [7; 32], "Ridge north", 100, 200)
                .unwrap();
        let error = SitedStationGrant::issue_with(
            root.path(),
            [1; 32],
            [2; 32],
            request,
            sign_with(root.path(), [0x53; 32]),
        )
        .unwrap_err();
        assert!(matches!(error, SitedStationGrantError::IssuerMismatch));
    }
}
