//! Block diagrams that the code draws of itself.
//!
//! The code is divided into blocks where it is written, by one tag in the
//! documentation it already has:
//!
//! ```text
//! //! Time integration of `M ẋ = f(x)` with the implicit trapezoidal rule, …   (module docs)
//! //!
//! //! @block Time integration
//!
//! /// Take one step from `x` …                                                  (item docs)
//! // @block One step
//! pub fn step(…) {
//!     // @block Predict                                                          (a section)
//!     // Start Newton from the extrapolated history …
//!     …
//!     // @block Newton iterations
//!     // …
//! }
//! ```
//!
//! [`scan`] reads a module tree into blocks (module → item → section, nested
//! by where they are), takes each block's id from its Rust path and its edges
//! from the code (a name one block uses and another defines), and
//! [`to_html`] draws them as one self-contained, interactive page: the blocks
//! of one level with what uses what, a summary on hover, the sub-blocks on a
//! click, the full documentation and the code beside it.
//!
//! Nothing is written twice: the text is the rustdoc the code carries anyway,
//! the structure is where the code is, and the arrows are what the code
//! calls. A build script or a test turns annotation problems (a module
//! without a block, an undocumented block, an unknown tag) into warnings or
//! failures, so the diagram cannot drift from the code.
//!
//! @block Block docs generator

pub mod lex;
pub mod render;
pub mod scan;

use std::path::Path;

pub use render::{to_html, to_json, Meta};
pub use scan::{build, scan, Block, Doc, Edge, Issue, Kind};

/// Scan `dir` as module `module` and write the page to `html` (and the data
/// to `json`, if given).
///
/// # Errors
/// When a file cannot be read or written.
pub fn generate(
    dir: &Path,
    module: &str,
    base: &Path,
    title: &str,
    meta: &Meta,
    html: &Path,
    json: Option<&Path>,
) -> std::io::Result<Doc> {
    let doc = scan(dir, module, base, title)?;
    if let Some(parent) = html.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(html, to_html(&doc, meta))?;
    if let Some(json) = json {
        std::fs::write(json, to_json(&doc, meta))?;
    }
    Ok(doc)
}
