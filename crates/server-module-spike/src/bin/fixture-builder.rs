//! Deterministically builds inert proof fixtures into Component Model artifacts.

use std::env;
use std::fs;
use std::path::PathBuf;

use omarchygs_server_module_spike::{ProofError, build_fixture_component};

const FIXTURES: [(&str, &[u8]); 8] = [
    (
        "valid",
        include_bytes!("../../fixtures/components/valid.wat"),
    ),
    ("noop", include_bytes!("../../fixtures/components/noop.wat")),
    (
        "unauthorized",
        include_bytes!("../../fixtures/components/unauthorized.wat"),
    ),
    ("trap", include_bytes!("../../fixtures/components/trap.wat")),
    ("loop", include_bytes!("../../fixtures/components/loop.wat")),
    (
        "memory-hog",
        include_bytes!("../../fixtures/components/memory-hog.wat"),
    ),
    (
        "forbidden-import",
        include_bytes!("../../fixtures/components/forbidden-import.wat"),
    ),
    (
        "wrong-interface",
        include_bytes!("../../fixtures/components/wrong-interface.wat"),
    ),
];

fn main() {
    if let Err(error) = run() {
        eprintln!("fixture build failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ProofError> {
    let mut args = env::args().skip(1);
    let output = match (args.next(), args.next()) {
        (Some(output), None) => PathBuf::from(output),
        _ => {
            return Err(ProofError::Contract(
                "usage: fixture-builder <output-directory>".into(),
            ));
        }
    };
    fs::create_dir_all(&output)?;
    for (name, source) in FIXTURES {
        let component = build_fixture_component(source)?;
        fs::write(output.join(format!("{name}.wasm")), component)?;
    }
    Ok(())
}
