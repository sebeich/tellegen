//! Draws the RMS engine's block diagram at every build, from the `@block`
//! annotations in its code (`tellegen-blocks`), for `/docs/blocks/rms`. An
//! annotation problem is a build warning naming the file and line.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let root = manifest.join("../..");
    let rms = manifest.join("../tellegen/src/rms");
    // A directory: cargo reruns when any file under it changes.
    println!("cargo:rerun-if-changed={}", rms.display());
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("rms-blocks.html");
    let doc = tellegen_blocks::generate(
        &rms,
        "rms",
        &root,
        "tellegen RMS engine",
        &tellegen_blocks::Meta::default(),
        &out,
        None,
    )
    .expect("draw the RMS block diagram");
    for issue in &doc.issues {
        println!("cargo:warning=block docs: {issue}");
    }
}
