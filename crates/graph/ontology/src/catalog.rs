// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use eidetic::pack::{Origin, PackHeader, PackKind, compile_authored};
use eidetic::schema::TrustEnvelope;
use mere::profile::core::CORE_PACK_ID;
use mere::profile::load::{Limits, Profile, load_local_lens};
use mere::profile::pack::PIN_HASH;
use mere::profile::resolve::{Offered, PackRef, PackSource};

#[derive(Default)]
struct Catalog(BTreeMap<String, Vec<u8>>);

impl PackSource for Catalog {
    fn offer(&self, id: &str) -> Vec<Offered> {
        self.0.get(id).map_or_else(Vec::new, |bytes| {
            vec![Offered {
                canonical: bytes.clone(),
                envelope: TrustEnvelope::self_asserted(),
                origin: Origin::Local,
            }]
        })
    }

    fn handler_known(&self, _: &str) -> bool {
        false
    }
}

impl Catalog {
    fn insert(&mut self, text: &str, limits: &Limits, kind: PackKind) -> Result<String, String> {
        if text.len() > limits.max_pack_bytes || self.0.len() >= limits.max_packs {
            return Err("Local ontology pack exceeds the pack size or count limit".into());
        }
        let (header, canonical) = compile_authored(text).map_err(|e| e.to_string())?;
        if header.id == CORE_PACK_ID || header.body.len() != 1 || !header.body.contains_key(&kind) {
            return Err(format!(
                "Expected a local {kind:?} pack, found {}",
                header.id
            ));
        }
        if self.0.insert(header.id.clone(), canonical).is_some() {
            return Err(format!("Duplicate local ontology pack {}", header.id));
        }
        Ok(header.id)
    }
}

fn read(path: &Path, limits: &Limits) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut text = String::new();
    file.take(limits.max_pack_bytes as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text)
}

/// Load a product's default lens and its bundled subjects, or select the
/// operator's lens.toml and local subject packs. A local subject supersedes
/// the bundled copy with the same ID. Only regular TOML files in the
/// selected directory are read. Peer documents cannot select this directory.
///
/// # Errors
///
/// Returns the filename or admission failure for an invalid selection.
pub fn load(
    default_lens: &str,
    bundled: &[&str],
    directory: Option<&Path>,
) -> Result<Profile, String> {
    let limits = Limits::default();
    let mut catalog = Catalog::default();
    for text in bundled {
        catalog.insert(text, &limits, PackKind::Vocabulary)?;
    }
    let local_lens;
    let lens = if let Some(dir) = directory {
        let mut local = Catalog::default();
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let lens_path = dir.join("lens.toml");
        let mut found_lens = false;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension() != Some(OsStr::new("toml")) {
                continue;
            }
            if !fs::metadata(&path)
                .map_err(|e| format!("{}: {e}", path.display()))?
                .is_file()
            {
                return Err(format!(
                    "Expected a regular ontology file: {}",
                    path.display()
                ));
            }
            if path == lens_path {
                found_lens = true;
                continue;
            }
            local
                .insert(&read(&path, &limits)?, &limits, PackKind::Vocabulary)
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        if !found_lens {
            return Err(format!("Missing {}", lens_path.display()));
        }
        catalog.0.extend(local.0);
        local_lens = read(&lens_path, &limits)?;
        &local_lens
    } else {
        default_lens
    };
    let id = catalog.insert(lens, &limits, PackKind::Lens)?;
    let pin = PackHeader::pin_of(&catalog.0[&id], PIN_HASH).map_err(|e| e.to_string())?;
    load_local_lens(&PackRef::Pinned(pin), &catalog, &limits)
        .map_err(|e| format!("Could not admit ontology lens {id}: {e:?}"))
}
