// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl ActionFormV1 {
    /// An empty form for `schema`, which must match the action it belongs to.
    pub fn new(schema: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            fields: Vec::new(),
        }
    }

    /// Append one input.
    pub fn with_field(mut self, field: ActionFormFieldV1) -> Self {
        self.fields.push(field);
        self
    }

    /// Check static form facts before a host exposes the action.
    pub fn validate(&self) -> Result<(), ActionFormError> {
        if self.schema.trim().is_empty() {
            return Err(ActionFormError::EmptyFormSchema);
        }
        let mut names = BTreeSet::new();
        for field in &self.fields {
            if field.name.trim().is_empty() {
                return Err(ActionFormError::EmptyFieldName);
            }
            if field.name == "schema" {
                return Err(ActionFormError::ReservedFieldName);
            }
            if !names.insert(field.name.clone()) {
                return Err(ActionFormError::DuplicateField(field.name.clone()));
            }
            if field.label.trim().is_empty() {
                return Err(ActionFormError::EmptyFieldLabel(field.name.clone()));
            }
            if field.choices.is_empty() {
                return Err(ActionFormError::EmptyChoices(field.name.clone()));
            }
            let mut values = BTreeSet::new();
            for choice in &field.choices {
                if choice.value.trim().is_empty() {
                    return Err(ActionFormError::EmptyChoiceValue(field.name.clone()));
                }
                if choice.label.trim().is_empty() {
                    return Err(ActionFormError::EmptyChoiceLabel(field.name.clone()));
                }
                if !values.insert(choice.value.clone()) {
                    return Err(ActionFormError::DuplicateChoiceValue {
                        field: field.name.clone(),
                        value: choice.value.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Build the version-one JSON payload after checking the endpoint's form.
    pub fn compose_payload(
        &self,
        expected_schema: &str,
        values: &BTreeMap<String, String>,
    ) -> Result<Vec<u8>, ActionFormError> {
        self.validate()?;
        if self.schema != expected_schema {
            return Err(ActionFormError::SchemaMismatch {
                expected: expected_schema.to_string(),
                form: self.schema.clone(),
            });
        }

        for name in values.keys() {
            if !self.fields.iter().any(|field| field.name == *name) {
                return Err(ActionFormError::UnknownField(name.clone()));
            }
        }
        for field in &self.fields {
            let value = values.get(&field.name);
            if field.required && value.is_none_or(|value| value.is_empty()) {
                return Err(ActionFormError::MissingField(field.name.clone()));
            }
            let Some(value) = value else {
                continue;
            };
            if !field.choices.iter().any(|choice| choice.value == *value) {
                return Err(ActionFormError::InvalidChoice {
                    field: field.name.clone(),
                    value: value.clone(),
                });
            }
        }

        #[derive(Serialize)]
        struct Payload<'a> {
            schema: &'a str,
            #[serde(flatten)]
            values: &'a BTreeMap<String, String>,
        }

        serde_json::to_vec(&Payload {
            schema: &self.schema,
            values,
        })
        .map_err(|error| ActionFormError::Encode(error.to_string()))
    }
}

impl ActionFormFieldV1 {
    /// A required field offering exactly these choices.
    pub fn choice(
        name: impl Into<String>,
        label: impl Into<String>,
        choices: impl IntoIterator<Item = ActionFormChoiceV1>,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            description: String::new(),
            required: true,
            choices: choices.into_iter().collect(),
        }
    }

    /// Add the longer guidance.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Let this field be omitted when the payload is composed.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

impl ActionFormChoiceV1 {
    /// A choice carrying `value` behind `label`.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
        }
    }

    /// Add the longer guidance.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
