fn main() {
    let wit =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../wit/projection-session.wit");
    println!("cargo:rerun-if-changed={}", wit.display());
    let src = c0_witgen::generate(&wit, "projection-session").expect("generate bindings");
    let out =
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("projection_session.rs");
    std::fs::write(out, src).unwrap();
}
