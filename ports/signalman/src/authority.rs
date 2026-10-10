// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wallet acts a station port asks of djinn (dramatis repo plan, D17).
//!
//! Signalman holds no wallet. It derives a station's credential from the
//! station-scoped keys djinn releases ([`station_salts`]), and asks djinn to
//! sign the station's grant and to revoke it; the wallet's seed never leaves
//! djinn. [`StationAuthority`] is that seam, and [`DjinnStationAuthority`] is
//! its one production implementation, over graphshell's custody client.

use std::io;
use std::sync::Arc;

use graphshell::native::app_admission::{AppId, configured_app_endpoint};
use graphshell::native::custody::{CustodyRefusal, KeySource, ResidentStatus, StationGrantRequest};
use graphshell::native::custody_client::{BlockingCustodyClient, CustodyClientError};
use pandect::station_grant::{SitedStationGrantError, SitedStationGrantRequest};
use pandect::{DeviceId, RemoteAuthRevocationOutcome};
use personae::RetainedKeys;
use personae::carry::DeviceGrantSet;

/// The application name Signalman connects to djinn as.
pub const SIGNALMAN_APP: &str = "signalman";

/// The salts a station's credential derives from: its Reticulum material and
/// its control signer. These are what djinn releases for one station.
pub fn station_salts(device_id: DeviceId) -> Vec<Vec<u8>> {
    let scope = device_id.as_uuid().as_bytes();
    let mut salts = personae::reticulum::station_salts(scope).to_vec();
    salts.push(personae::reticulum::station_control_salt(scope));
    salts
}

/// The salts a first-owner controller derives from, for one scope.
pub fn controller_salts(controller_scope: &[u8]) -> Vec<Vec<u8>> {
    personae::reticulum::controller_salts(controller_scope).to_vec()
}

/// The wallet acts a station needs, done by the wallet's custodian.
pub trait StationAuthority: Send + Sync {
    /// Sign and record a sited station's narrow grant. The custodian checks
    /// that `issuer_public_key` is its wallet's root.
    fn issue_station_grant(
        &self,
        request: &SitedStationGrantRequest,
        issuer_public_key: [u8; 32],
    ) -> Result<DeviceGrantSet, SitedStationGrantError>;

    /// Revoke one station in the wallet.
    fn revoke_device(&self, device_id: DeviceId) -> io::Result<RemoteAuthRevocationOutcome>;
}

/// djinn, over the custody route. Each call runs on its own thread with its
/// own runtime, so a station's async tasks may call it without nesting
/// runtimes; commissioning and revocation are rare.
#[derive(Clone, Debug)]
pub struct DjinnStationAuthority {
    endpoint: String,
    app: AppId,
}

impl Default for DjinnStationAuthority {
    fn default() -> Self {
        Self::at(configured_app_endpoint())
    }
}

impl DjinnStationAuthority {
    /// djinn at an explicit first-party endpoint.
    pub fn at(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            app: AppId::new(SIGNALMAN_APP),
        }
    }

    /// As a shared trait object, for a lease.
    pub fn shared(self) -> Arc<dyn StationAuthority> {
        Arc::new(self)
    }

    fn with_client<T: Send>(
        &self,
        act: impl FnOnce(&mut BlockingCustodyClient) -> Result<T, CustodyClientError> + Send,
    ) -> Result<T, CustodyClientError> {
        std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let mut client =
                        BlockingCustodyClient::open_at(&self.endpoint, self.app.clone())?;
                    act(&mut client)
                })
                .join()
                .expect("the custody call thread does not panic")
        })
    }

    /// djinn's status: whether the identity is pending, and where the
    /// wallet's public records live.
    pub fn status(&self) -> Result<ResidentStatus, CustodyClientError> {
        self.with_client(|client| client.status())
    }

    /// The station-scoped keys for `device_id`, released by djinn (D11).
    pub fn release_station(&self, device_id: DeviceId) -> Result<RetainedKeys, CustodyClientError> {
        self.with_client(|client| client.release(KeySource::Wallet, station_salts(device_id)))
    }

    /// The key for one station-scoped `salt` (a bench head's storage key).
    pub fn release_storage(&self, salt: Vec<u8>) -> Result<RetainedKeys, CustodyClientError> {
        self.with_client(|client| client.release(KeySource::Wallet, vec![salt]))
    }

    /// The controller-scoped keys for `controller_scope`.
    pub fn release_controller(
        &self,
        controller_scope: &[u8],
    ) -> Result<RetainedKeys, CustodyClientError> {
        let salts = controller_salts(controller_scope);
        self.with_client(|client| client.release(KeySource::Wallet, salts))
    }
}

impl StationAuthority for DjinnStationAuthority {
    fn issue_station_grant(
        &self,
        request: &SitedStationGrantRequest,
        issuer_public_key: [u8; 32],
    ) -> Result<DeviceGrantSet, SitedStationGrantError> {
        let request = StationGrantRequest {
            device_id: *request.device_id().as_uuid(),
            station_ed25519_public_key: request.station_ed25519_public_key(),
            label: request.label().to_string(),
            issued_at_ms: request.issued_at_ms(),
            expires_at_ms: request.expires_at_ms(),
        };
        self.with_client(|client| client.issue_station_grant(request, issuer_public_key))
            .map_err(grant_error)
    }

    fn revoke_device(&self, device_id: DeviceId) -> io::Result<RemoteAuthRevocationOutcome> {
        self.with_client(|client| client.revoke_device(device_id))
            .map_err(io::Error::other)
    }
}

fn grant_error(error: CustodyClientError) -> SitedStationGrantError {
    match error {
        CustodyClientError::Refused(CustodyRefusal::Locked | CustodyRefusal::NoWallet) => {
            SitedStationGrantError::WalletLocked
        },
        error if error.is_pending() => SitedStationGrantError::WalletLocked,
        other => SitedStationGrantError::Storage(io::Error::other(other)),
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    //! A wallet that signs in the test process, standing in for djinn. It
    //! records the grant and the roster entry exactly where djinn's castellan
    //! does, so the lease's checks read what a real wallet writes.

    use std::path::{Path, PathBuf};

    use pandect::station_grant::SitedStationGrant;
    use pandect::wallet_grant::upsert_remote_auth_device_record;
    use pandect::{
        DeviceRoster, load_device_roster, revoke_device_certificates, save_device_grant_set,
        save_device_roster,
    };
    use personae::{IdentityProvider, InMemoryProvider};

    use super::*;

    pub(crate) struct TestWallet {
        root: PathBuf,
        seed: [u8; 32],
    }

    impl TestWallet {
        pub(crate) fn new(root: &Path, seed: u8) -> Arc<Self> {
            Arc::new(Self {
                root: root.to_path_buf(),
                seed: [seed; 32],
            })
        }

        /// What djinn's custody route releases: the wallet root's keys.
        pub(crate) fn provider(&self) -> InMemoryProvider {
            InMemoryProvider::from_seed(self.seed)
        }
    }

    impl StationAuthority for TestWallet {
        fn issue_station_grant(
            &self,
            request: &SitedStationGrantRequest,
            issuer_public_key: [u8; 32],
        ) -> Result<DeviceGrantSet, SitedStationGrantError> {
            let wallet = self.provider().master_public_key().to_bytes();
            let grant = SitedStationGrant::issue_with(
                &self.root,
                issuer_public_key,
                wallet,
                request.clone(),
                |spec| {
                    let actions: Vec<&str> = spec.scopes.iter().map(String::as_str).collect();
                    let set = personae::carry::issue_device_grant_set(
                        self.seed,
                        spec.device_id,
                        spec.delegatee_pubkey,
                        &actions,
                        &spec.personas,
                        spec.expires_at_ms.unwrap_or(spec.issued_at_ms) - spec.issued_at_ms,
                        spec.issued_at_ms,
                    )
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                    let grant_ref = save_device_grant_set(&self.root, spec.device_id, &set)?;
                    let mut roster =
                        load_device_roster(&self.root)?.unwrap_or_else(DeviceRoster::new);
                    upsert_remote_auth_device_record(&mut roster, spec, grant_ref);
                    save_device_roster(&self.root, &roster)?;
                    Ok(set)
                },
            )?;
            Ok(grant.signed().clone())
        }

        fn revoke_device(&self, device_id: DeviceId) -> io::Result<RemoteAuthRevocationOutcome> {
            let statements = revoke_device_certificates(&self.root, self.seed, device_id, 1)?;
            let mut roster = load_device_roster(&self.root)?.unwrap_or_else(DeviceRoster::new);
            let already_revoked = roster.revoked.contains(&device_id);
            if !already_revoked {
                roster.revoked.push(device_id);
                save_device_roster(&self.root, &roster)?;
            }
            Ok(RemoteAuthRevocationOutcome {
                device_id,
                already_revoked,
                rotated_personas: Vec::new(),
                refreshed_devices: Vec::new(),
                statements,
            })
        }
    }
}
