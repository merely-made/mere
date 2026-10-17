// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Independent material held by a workspace. Hosts own storage and presentation.
//! No expiry, placement state, live source binding or source-write authority.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub id: Uuid,
    pub title: String,
    pub media_type: String,
    pub text: String,
    /// Navigation/provenance only; reading a Pool never follows this address.
    pub source: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pool {
    pub version: u32,
    pub items: Vec<Material>,
}
impl Default for Pool {
    fn default() -> Self {
        Self {
            version: 1,
            items: Vec::new(),
        }
    }
}
impl Pool {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(
                "This Pool uses an unsupported format; its contents have been left intact".into(),
            );
        }
        let mut ids = std::collections::HashSet::new();
        for item in &self.items {
            if !ids.insert(item.id)
                || item.id.is_nil()
                || item.text.is_empty()
                || item.media_type.is_empty()
            {
                return Err(
                    "Unreadable Pool material; stored contents have been left intact".into(),
                );
            }
        }
        Ok(())
    }
    pub fn add(
        &mut self,
        title: String,
        media_type: String,
        text: String,
        source: Option<String>,
    ) -> Result<Uuid, String> {
        let item = Material {
            id: Uuid::new_v4(),
            title,
            media_type,
            text,
            source,
        };
        let id = item.id;
        let mut candidate = self.clone();
        candidate.items.push(item);
        candidate.validate()?;
        *self = candidate;
        Ok(id)
    }
    pub fn remove(&mut self, id: Uuid) -> Result<(), String> {
        let index = self
            .items
            .iter()
            .position(|item| item.id == id)
            .ok_or("Material is no longer in the Pool")?;
        self.items.remove(index);
        Ok(())
    }
    pub fn get(&self, id: Uuid) -> Result<&Material, String> {
        self.items
            .iter()
            .find(|item| item.id == id)
            .ok_or("Material is no longer in the Pool".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_finished_and_unresolved_material_have_the_same_lifetime() {
        let mut pool = Pool::default();
        let first = pool
            .add(
                "Verse".into(),
                "text/djot".into(),
                "unknown words\n".into(),
                Some("file:verse.djot".into()),
            )
            .unwrap();
        let second = pool
            .add(
                "Progression".into(),
                "text/djot".into(),
                "I IV V I\n".into(),
                None,
            )
            .unwrap();
        let mut copy = pool.get(first).unwrap().clone();
        copy.text.push_str("independent edit");
        assert_eq!(pool.get(first).unwrap().text, "unknown words\n");
        pool.remove(second).unwrap();
        assert_eq!(pool.items.len(), 1);
        let before = pool.clone();
        assert!(
            pool.add("".into(), "text/plain".into(), "".into(), None)
                .is_err()
        );
        assert_eq!(pool, before);
        pool.items.push(pool.items[0].clone());
        assert!(pool.validate().is_err());
        pool.items.pop();
        pool.version = 2;
        assert!(pool.validate().is_err());
    }
}
