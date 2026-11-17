//! The RMS engine's block annotations are complete: every module is a block,
//! every block is documented, every tag is known. A failure here names the
//! file and line to fix.

use std::path::Path;

#[test]
fn rms_blocks_are_complete() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = crates.join("tellegen/src/rms");
    let doc = tellegen_blocks::scan(&dir, "rms", &crates, "RMS").expect("scan the RMS engine");
    let issues: Vec<String> = doc.issues.iter().map(ToString::to_string).collect();
    assert!(
        issues.is_empty(),
        "block annotation issues:\n{}",
        issues.join("\n")
    );
    assert!(doc.files.len() >= 15, "found {} files", doc.files.len());
    // Every module hangs under the RMS root.
    let root = doc.roots();
    assert_eq!(root.len(), 1, "one root block");
    assert_eq!(doc.blocks[root[0]].id, "rms");
    // The diagram has something to draw.
    assert!(!doc.edges.is_empty());
}
