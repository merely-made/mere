fn main() {
    println!("cargo:rerun-if-env-changed=C0_CORPUS");
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("corpus.rs");
    match std::env::var("C0_CORPUS") {
        Ok(path) => {
            println!("cargo:rerun-if-changed={path}");
            std::fs::copy(path, out).unwrap();
        },
        Err(_) => std::fs::write(out, "").unwrap(),
    }
}
