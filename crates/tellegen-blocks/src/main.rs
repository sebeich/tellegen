//! `tellegen-blocks <dir>`: write the block diagram of a module tree.
//!
//! ```text
//! tellegen-blocks crates/tellegen/src/rms --title "tellegen RMS engine" --out target/blocks/rms.html
//! tellegen-blocks crates/tellegen/src/rms --check     # fail on annotation issues
//! ```
//!
//! Options: `--module <name>` (the module path of `<dir>`, default its
//! directory name), `--title <text>`, `--base <dir>` (paths are shown relative
//! to it, default the current directory), `--out <file.html>` (default
//! `blocks.html`), `--json <file.json>`, `--source-url <prefix>` (links each
//! file to `<prefix><path>`), `--from <note>` (a revision or date shown in the
//! footer), `--check` (exit 1 when there are issues).
//!
//! @block Command line

use std::path::PathBuf;
use std::process::ExitCode;

use tellegen_blocks::{generate, Meta};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut dir: Option<PathBuf> = None;
    let mut module = None;
    let mut title = None;
    let mut base = std::env::current_dir().unwrap_or_default();
    let mut out = PathBuf::from("blocks.html");
    let mut json = None;
    let mut meta = Meta::default();
    let mut check = false;
    while let Some(a) = args.next() {
        let mut value = |name: &str| {
            args.next().unwrap_or_else(|| {
                eprintln!("{name} needs a value");
                std::process::exit(2)
            })
        };
        match a.as_str() {
            "--module" => module = Some(value("--module")),
            "--title" => title = Some(value("--title")),
            "--base" => base = PathBuf::from(value("--base")),
            "--out" => out = PathBuf::from(value("--out")),
            "--json" => json = Some(PathBuf::from(value("--json"))),
            "--source-url" => meta.source_url = value("--source-url"),
            "--from" => meta.generated_from = value("--from"),
            "--check" => check = true,
            "-h" | "--help" => {
                println!("usage: tellegen-blocks <dir> [--module NAME] [--title TEXT] [--base DIR] [--out FILE.html] [--json FILE.json] [--source-url PREFIX] [--from NOTE] [--check]");
                return ExitCode::SUCCESS;
            }
            _ if dir.is_none() && !a.starts_with('-') => dir = Some(PathBuf::from(a)),
            _ => {
                eprintln!("unknown argument {a}; see --help");
                return ExitCode::from(2);
            }
        }
    }
    let Some(dir) = dir else {
        eprintln!("usage: tellegen-blocks <dir> [options]; see --help");
        return ExitCode::from(2);
    };
    let module = module.unwrap_or_else(|| {
        dir.file_name()
            .map_or_else(|| "crate".to_owned(), |n| n.to_string_lossy().into_owned())
    });
    let title = title.unwrap_or_else(|| module.clone());
    let base = base.canonicalize().unwrap_or(base);
    let dir = dir.canonicalize().unwrap_or(dir);
    match generate(&dir, &module, &base, &title, &meta, &out, json.as_deref()) {
        Ok(doc) => {
            for issue in &doc.issues {
                eprintln!("warning: {issue}");
            }
            println!(
                "{}: {} blocks, {} edges, {} files, {} issues",
                out.display(),
                doc.blocks.len(),
                doc.edges.len(),
                doc.files.len(),
                doc.issues.len()
            );
            if check && !doc.issues.is_empty() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
