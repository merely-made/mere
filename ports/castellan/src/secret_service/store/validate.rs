// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resource limits and text checks every value passes before it reaches
//! the item store.

use std::collections::BTreeMap;

use super::{SecretServiceError, SecretServiceStore};

impl SecretServiceStore {
    pub(super) fn validate_name(
        &self,
        text: &str,
        kind: &'static str,
    ) -> Result<(), SecretServiceError> {
        if text.is_empty() || text.chars().any(char::is_control) {
            return Err(SecretServiceError::InvalidText(kind));
        }
        if text.len() > self.limits.max_name_bytes {
            return Err(SecretServiceError::Limit(kind));
        }
        Ok(())
    }

    pub(super) fn validate_attributes(
        &self,
        attributes: &BTreeMap<String, String>,
    ) -> Result<(), SecretServiceError> {
        if attributes.len() > self.limits.max_attributes {
            return Err(SecretServiceError::Limit("attributes per item"));
        }
        for (key, value) in attributes {
            self.validate_name(key, "attribute key")?;
            if value.len() > self.limits.max_attribute_value_bytes
                || value.chars().any(char::is_control)
            {
                return Err(SecretServiceError::Limit("attribute value"));
            }
        }
        Ok(())
    }

    pub(super) fn validate_secret(&self, secret: &[u8]) -> Result<(), SecretServiceError> {
        if secret.len() > self.limits.max_secret_bytes {
            Err(SecretServiceError::Limit("secret bytes"))
        } else {
            Ok(())
        }
    }

    pub(super) fn validate_alias(&self, alias: &str) -> Result<(), SecretServiceError> {
        self.validate_name(alias, "collection alias")?;
        if alias
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            Ok(())
        } else {
            Err(SecretServiceError::InvalidText("collection alias"))
        }
    }
}
