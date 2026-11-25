// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Read portable saved views against current RDF exports explicitly supplied by the operator.
use mere::{
    ContextCache,
    formats::{RdfFormat, parse::parse},
};
use mere_ontology::{CITATION, Profile, RecordIndex, SavedView, ViewQuery, WEB_ANNOTATION};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
    process::ExitCode,
};

const USAGE: &str = "mere-view define LENS_DIR SCOPE NAME type|incoming|outgoing IRI SOURCE_ID...\nmere-view run LENS_DIR SCOPE VIEW.json [SOURCE_ID BASE_IRI RDF_FILE]...";
const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;

fn read(path: &str, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|f| f.take(limit as u64 + 1).read_to_end(&mut bytes))
        .map_err(|e| format!("{path}: {e}"))?;
    if bytes.len() > limit {
        return Err(format!("{path} exceeds {limit} bytes"));
    }
    Ok(bytes)
}

fn run(
    profile: &Profile,
    scope: &str,
    recipe: &str,
    bindings: &[String],
) -> Result<String, String> {
    let bytes = read(recipe, SavedView::MAX_BYTES)?;
    let view = SavedView::from_json(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)?;
    view.check_scope(scope, profile)?;
    let selected: BTreeSet<_> = view.sources.iter().map(String::as_str).collect();
    let mut sources = BTreeMap::new();
    for binding in bindings.chunks(3) {
        let [id, base, file] = binding else {
            return Err(USAGE.into());
        };
        if !selected.contains(id.as_str()) || sources.insert(id.as_str(), (base, file)).is_some() {
            return Err(format!("Unexpected or duplicate source binding: {id}"));
        }
    }
    let contexts = mere_ontology::with_web_annotation_context(ContextCache::full());
    let context_revision = std::sync::OnceLock::new();
    let results = view.evaluate_loaded(scope, profile, |id| {
        let Some(&(base, file)) = sources.get(id) else {
            return Ok(None);
        };
        oxiri::Iri::parse(base.as_str()).map_err(|e| format!("Invalid base for {id}: {e}"))?;
        let format = match Path::new(file).extension().and_then(|e| e.to_str()) {
            Some("json") => Some(RdfFormat::JsonLd {
                profile: Default::default(),
            }),
            Some(ext) => RdfFormat::from_extension(ext),
            None => None,
        }
        .ok_or_else(|| format!("Unknown RDF file extension: {file}"))?;
        let bytes = read(file, MAX_SOURCE_BYTES)?;
        let dataset = parse(format, &bytes, Some(base), contexts.clone())
            .map_err(|e| format!("{file}: {e}"))?;
        let context = matches!(format, RdfFormat::JsonLd { .. })
            .then(|| context_revision.get_or_init(|| contexts.revision()));
        let identity =
            serde_json::to_vec(&(base, format.iri(), eidetic::Hash::of(&bytes), context))
                .map_err(|e| e.to_string())?;
        let revision = eidetic::Hash::of(&identity).to_string();
        Ok(Some((
            revision,
            RecordIndex::from_dataset(profile, &dataset),
        )))
    })?;
    serde_json::to_string_pretty(&results).map_err(|e| e.to_string())
}

fn execute(args: &[String]) -> Result<String, String> {
    let [command, lens, scope, rest @ ..] = args else {
        return Err(USAGE.into());
    };
    let profile = mere_ontology::load("", &[CITATION, WEB_ANNOTATION], Some(Path::new(lens)))?;
    match (command.as_str(), rest) {
        ("define", [name, kind, iri, sources @ ..]) => {
            let query = match kind.as_str() {
                "type" => ViewQuery::Type { class: iri.clone() },
                "incoming" => ViewQuery::Incoming {
                    record: iri.clone(),
                },
                "outgoing" => ViewQuery::Outgoing {
                    record: iri.clone(),
                },
                _ => return Err(USAGE.into()),
            };
            SavedView::new(
                name.clone(),
                scope.clone(),
                sources.to_vec(),
                query,
                &profile,
            )?
            .to_json()
        },
        ("run", [recipe, bindings @ ..]) => run(&profile, scope, recipe, bindings),
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let args: Result<Vec<_>, _> = std::env::args_os()
        .skip(1)
        .map(|s| s.into_string())
        .collect();
    match args
        .map_err(|_| "Arguments must be UTF-8".into())
        .and_then(|args| execute(&args))
    {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        },
    }
}
