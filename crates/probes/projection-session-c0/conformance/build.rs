//! Bind the conformance interface through both generators.

use wit_bindgen_core::{Files, wit_parser::Resolve};

fn main() {
    let wit = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wit/conformance.wit");
    println!("cargo:rerun-if-changed={}", wit.display());
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let ours = c0_witgen::generate(&wit, "values").expect("our generator");
    std::fs::write(out.join("ours.rs"), ours).unwrap();

    let mut resolve = Resolve::default();
    let package = resolve.push_file(&wit).unwrap();
    let world = resolve
        .select_world(&[package], Some("conformance"))
        .unwrap();
    let mut files = Files::default();
    wit_bindgen_wrpc_rust::Opts {
        format: true,
        ..Default::default()
    }
    .build()
    .generate(&mut resolve, world, &mut files)
    .unwrap();
    let (_, src) = files.iter().next().unwrap();
    std::fs::write(out.join("wrpc.rs"), src).unwrap();
}
