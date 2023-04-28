use bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let bindings: bindgen::Bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Failed to generate bindings");

    let out_path: PathBuf = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("v4l2_bindings.rs"))
        .expect("Failed to write bindings");
}
