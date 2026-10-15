// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident inference lane (resident inference and recall plan, I1).
//!
//! One esp inference actor, owned by Distillery's [`InferenceLane`], served
//! on the `inference.ask` route to Turnstone (ruling 8). The lane borrows the
//! Distillery lane's model library (ruling 3), so settings that name it
//! without a Distillery lane are refused before anything opens, as are
//! settings that name it in a build without the `inference` feature.
//!
//! [`InferenceLane`]: distillery::inference::InferenceLane

use crate::settings::OwnerSettings;

/// Refuse inference settings this build or this resident cannot honour.
/// Called before any lane opens, so a refusal leaves nothing to unwind.
pub fn validate(owner: &OwnerSettings) -> Result<(), String> {
    if owner.inference.is_none() {
        return Ok(());
    }
    if !cfg!(feature = "inference") {
        return Err(
            "inference names the resident inference lane, but this build carries no \
             inference lane; rebuild with the `inference` feature or remove the inference \
             block. A resident must not compose settings it cannot honour."
                .into(),
        );
    }
    if owner.distillery.is_none() {
        return Err(
            "inference needs the distillery lane: it borrows that lane's model library \
             (resident inference and recall plan, ruling 3); add a distillery block or \
             remove the inference block"
                .into(),
        );
    }
    Ok(())
}

#[cfg(feature = "inference")]
pub use lane::ResidentInference;

/// Without the feature the lane cannot exist: [`validate`] refuses its
/// settings, so no value of this type is ever made.
#[cfg(not(feature = "inference"))]
pub enum ResidentInference {}

#[cfg(not(feature = "inference"))]
impl ResidentInference {
    /// Uninhabited without the feature.
    pub fn register(
        &self,
        _: &mut graphshell::native::endpoint_catalog::ResidentEndpointCatalog,
    ) -> Result<
        graphshell::native::endpoint_catalog::ResidentEndpointRoute,
        graphshell::native::endpoint_catalog::ResidentEndpointCatalogError,
    > {
        match *self {}
    }

    /// Uninhabited without the feature.
    pub async fn close(self) -> Result<(), String> {
        match self {}
    }
}

#[cfg(feature = "inference")]
mod lane {
    use distillery::inference::{
        INFERENCE_ASK_NOTICE_POLL, INFERENCE_ASK_ROUTE, InferenceLane, StubInferenceProvider,
    };
    use graphshell::native::endpoint_catalog::{
        ResidentEndpointCatalog, ResidentEndpointCatalogError, ResidentEndpointRoute,
    };

    use crate::settings::{InferenceLaneSettings, InferenceModelSettings};

    /// The open lane.
    pub struct ResidentInference {
        lane: InferenceLane,
    }

    impl ResidentInference {
        /// Start the actor on the model the settings name.
        pub fn open(settings: &InferenceLaneSettings) -> Self {
            let lane = match settings.model {
                InferenceModelSettings::Stub => {
                    InferenceLane::open(|| Box::new(StubInferenceProvider::new()))
                },
            };
            Self { lane }
        }

        /// The lane itself, for in-process callers.
        pub fn lane(&self) -> &InferenceLane {
            &self.lane
        }

        /// Register `inference.ask`; each admitted session gets its own
        /// endpoint over the one lane.
        pub fn register(
            &self,
            catalog: &mut ResidentEndpointCatalog,
        ) -> Result<ResidentEndpointRoute, ResidentEndpointCatalogError> {
            let lane = self.lane.clone();
            catalog.register_notifying(
                INFERENCE_ASK_ROUTE,
                "Resident inference",
                move |context| Ok(lane.endpoint(context.session().clone())),
            )?;
            Ok(Self::route())
        }

        /// The route descriptor granted to a first-party client.
        pub fn route() -> ResidentEndpointRoute {
            ResidentEndpointRoute::new(INFERENCE_ASK_ROUTE, INFERENCE_ASK_NOTICE_POLL)
                .expect("the inference route is valid")
        }

        /// Cancel unfinished runs and join the lane's threads.
        pub async fn close(self) -> Result<(), String> {
            tokio::task::spawn_blocking(move || self.lane.close())
                .await
                .map_err(|error| format!("join the inference lane's close: {error}"))?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{InferenceLaneSettings, InferenceModelSettings};

    fn stub() -> Option<InferenceLaneSettings> {
        Some(InferenceLaneSettings {
            model: InferenceModelSettings::Stub,
        })
    }

    #[test]
    fn absent_settings_pass() {
        assert_eq!(validate(&OwnerSettings::default()), Ok(()));
    }

    #[cfg(feature = "inference")]
    #[test]
    fn the_lane_without_a_distillery_lane_is_refused() {
        let owner = OwnerSettings {
            inference: stub(),
            ..OwnerSettings::default()
        };
        let refusal = validate(&owner).unwrap_err();
        assert!(refusal.contains("needs the distillery lane"), "{refusal}");
    }

    #[cfg(not(feature = "inference"))]
    #[test]
    fn the_lane_in_a_build_without_it_is_refused() {
        let owner = OwnerSettings {
            inference: stub(),
            ..OwnerSettings::default()
        };
        let refusal = validate(&owner).unwrap_err();
        assert!(refusal.contains("`inference` feature"), "{refusal}");
    }

    #[test]
    fn settings_name_the_stub_in_kebab_case() {
        let owner: OwnerSettings =
            serde_json::from_str(r#"{"inference": {"model": "stub"}}"#).unwrap();
        assert_eq!(owner.inference, stub());
        assert!(
            serde_json::from_str::<OwnerSettings>(r#"{"inference": {"model": "llama-70b"}}"#)
                .is_err()
        );
    }
}
