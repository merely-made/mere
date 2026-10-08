// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Secret Service's Prompt object: `Unlock` on a locked vault returns
//! one, and its `Prompt()` runs the resident's native unlock (ruling 67).
//! `Completed` reports the unlocked objects, or `dismissed` on a cancel.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use zbus::Connection;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedObjectPath, Value};

use super::SERVICE_PATH;
use super::state::ServiceState;

pub(super) struct PromptInterface {
    state: Arc<ServiceState>,
    path: OwnedObjectPath,
    objects: Vec<OwnedObjectPath>,
    started: Arc<AtomicBool>,
}

impl PromptInterface {
    pub(super) fn new(
        state: Arc<ServiceState>,
        path: OwnedObjectPath,
        objects: Vec<OwnedObjectPath>,
    ) -> Self {
        Self {
            state,
            path,
            objects,
            started: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// A fresh prompt path.
pub(super) fn prompt_path() -> OwnedObjectPath {
    OwnedObjectPath::try_from(format!(
        "{SERVICE_PATH}/prompt/{}",
        uuid::Uuid::new_v4().simple()
    ))
    .expect("UUID prompt path is valid")
}

/// Emit `Completed` once and take the prompt off the bus.
async fn complete(
    connection: &Connection,
    path: &OwnedObjectPath,
    dismissed: bool,
    objects: Vec<OwnedObjectPath>,
) -> zbus::Result<()> {
    let emitter = SignalEmitter::new(connection, path.clone())?;
    PromptInterface::completed(&emitter, dismissed, Value::new(objects)).await?;
    connection
        .object_server()
        .remove::<PromptInterface, _>(path)
        .await?;
    Ok(())
}

#[zbus::interface(name = "org.freedesktop.Secret.Prompt")]
impl PromptInterface {
    /// Run the unlock; the answer arrives as `Completed`.
    async fn prompt(&self, _window_id: &str, #[zbus(connection)] connection: &Connection) {
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }
        let (state, path, objects) = (
            Arc::clone(&self.state),
            self.path.clone(),
            self.objects.clone(),
        );
        let connection = connection.clone();
        tokio::spawn(async move {
            let vault = Arc::clone(&state.vault);
            let unlocked = tokio::task::spawn_blocking(move || vault.prompt_unlock())
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or(false);
            let (dismissed, result) = match unlocked {
                true => (false, objects),
                false => (true, Vec::new()),
            };
            if let Err(error) = complete(&connection, &path, dismissed, result).await {
                tracing::warn!(%error, "Secret Service prompt could not complete");
            }
        });
    }

    /// Withdraw the prompt before it ran.
    async fn dismiss(&self, #[zbus(connection)] connection: &Connection) {
        if self.started.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Err(error) = complete(connection, &self.path, true, Vec::new()).await {
            tracing::warn!(%error, "Secret Service prompt could not be dismissed");
        }
    }

    #[zbus(signal)]
    async fn completed(
        emitter: &SignalEmitter<'_>,
        dismissed: bool,
        result: Value<'_>,
    ) -> zbus::Result<()>;
}
