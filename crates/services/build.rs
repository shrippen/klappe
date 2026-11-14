//! Embeds the example projects (`demo/beispiel-<lang>.klappe`, made by `klappe-demo`) into
//! the library, so an installed app needs no files beside its binary.
//!
//! Writes `$OUT_DIR/examples.rs`: for each language a list of `(path in the project, bytes)`.

use std::error::Error;
use std::fmt::Write;
use std::path::{Path, PathBuf};

const LANGS: [&str; 2] = ["de", "en"];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let demo = manifest.join("../../demo");
    let mut code = String::new();

    for lang in LANGS {
        let dir = demo.join(format!("beispiel-{lang}.klappe"));
        println!("cargo:rerun-if-changed={}", dir.display());

        writeln!(
            code,
            "const EXAMPLE_{}: &[(&str, &[u8])] = &[",
            lang.to_uppercase()
        )?;
        for (name, path) in files(&dir, &dir)? {
            writeln!(
                code,
                "    ({name:?}, include_bytes!({:?})),",
                path.canonicalize()?
            )?;
        }
        writeln!(code, "];")?;
    }

    std::fs::write(
        PathBuf::from(std::env::var("OUT_DIR")?).join("examples.rs"),
        code,
    )?;

    Ok(())
}

/// Every file under `dir` as `(path relative to root with /, absolute path)`, sorted.
fn files(root: &Path, dir: &Path) -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
    let mut found = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            found.extend(files(root, &path)?);
            continue;
        }
        let name = path
            .strip_prefix(root)?
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        found.push((name, path));
    }
    found.sort();

    Ok(found)
}
