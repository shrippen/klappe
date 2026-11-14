//! Generates `demo/beispiel-de.klappe` and `demo/beispiel-en.klappe` from the demo world
//! (`demo/world.json`, synced from shrippen.github.io/demo). A development tool: nothing
//! of it ships.
//!
//! ```text
//! cargo run -p klappe-demo [-- --world FILE --out DIR]
//! ```
//!
//! The projects are written through `klappe-store`, so they are in exactly the format the
//! app saves.

mod error;
mod example;
mod lang;

use std::path::PathBuf;
use std::process::ExitCode;

use error::DemoError;
use lang::Lang;

const USAGE: &str = "klappe-demo [--world FILE] [--out DIR]\n\
\n  --world FILE  demo world (default: demo/world.json)\n\
  --out DIR     where the projects go (default: demo)\n";

/// Exit status for a wrong command line (sysexits EX_USAGE).
const EXIT_USAGE: u8 = 64;
const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

struct Args {
    world: PathBuf,
    out: PathBuf,
}

fn parse_args(args: impl Iterator<Item = String>) -> Option<Args> {
    let demo = PathBuf::from(REPO_ROOT).join("demo");
    let mut parsed = Args {
        world: demo.join("world.json"),
        out: demo,
    };
    let mut args = args;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--world" => parsed.world = args.next()?.into(),
            "--out" => parsed.out = args.next()?.into(),
            _ => return None,
        }
    }

    Some(parsed)
}

fn run(args: &Args) -> Result<(), DemoError> {
    let text = std::fs::read_to_string(&args.world).map_err(|source| DemoError::Io {
        path: args.world.clone(),
        source,
    })?;
    let world = serde_json::from_str(&text)?;

    for lang in Lang::ALL {
        let dir = args.out.join(format!("beispiel-{}.klappe", lang.code()));
        example::write_example(&world, lang, &dir)?;
        println!("wrote {}", dir.display());
    }

    Ok(())
}

fn main() -> ExitCode {
    let Some(args) = parse_args(std::env::args().skip(1)) else {
        eprint!("{USAGE}");
        return ExitCode::from(EXIT_USAGE);
    };

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("klappe-demo: {e}");
            ExitCode::FAILURE
        }
    }
}
