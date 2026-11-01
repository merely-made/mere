use wit_bindgen_core::{Files, wit_parser::Resolve};

fn main() {
    let wit =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../wit/projection-session.wit");
    println!("cargo:rerun-if-changed={}", wit.display());
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for (world, file) in [
        ("projection-provider", "provider.rs"),
        ("projection-consumer", "consumer.rs"),
    ] {
        let mut resolve = Resolve::default();
        let package = resolve.push_file(&wit).unwrap();
        let id = resolve.select_world(&[package], Some(world)).unwrap();
        let mut files = Files::default();
        let opts = wit_bindgen_wrpc_rust::Opts {
            format: true,
            ..Default::default()
        };
        opts.build().generate(&mut resolve, id, &mut files).unwrap();
        let (_, src) = files.iter().next().unwrap();
        std::fs::write(out.join(file), src).unwrap();
    }
}
