// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Required wallet sealing for a host's mutable slots.

use std::io;

use pandect::{PersonaId, WalletSealedBackend};
use personae::IdentityStorage;

use crate::authority::PersonaeHost;

impl<S: IdentityStorage + 'static> PersonaeHost<S> {
    /// Seal a caller-selected Muniment backend under one persona's wallet.
    ///
    /// Unlike the optional payload seam, this refuses when the carry root or
    /// staged epoch is absent. Contact storage must not silently use cleartext.
    /// The host selects the backend and codec, and supplies historical epochs
    /// separately when reads must survive rotation. Keys remain visible;
    /// authenticated old values at the same key are not rollback-protected.
    pub fn sealed_backend<B>(
        &self,
        persona: PersonaId,
        backend: B,
    ) -> io::Result<WalletSealedBackend<B>> {
        let sealer = self.payload_sealer(persona)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "persona wallet epoch unavailable for sealed storage",
            )
        })?;
        Ok(WalletSealedBackend::new(backend, sealer))
    }
}
