use std::{env, fs, path::PathBuf};

use wit_component::{ComponentEncoder, StringEncoding, embed_component_metadata};
use wit_parser::Resolve;

const WIT: &str = include_str!("wit/omarchygs-module.wit");
const WORLD: &str = "module-production";

fn main() {
    println!("cargo:rerun-if-changed=wit/omarchygs-module.wit");
    println!("cargo:rerun-if-changed=fixtures");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    for name in [
        "valid",
        "noop",
        "unauthorized",
        "trap",
        "loop",
        "memory-hog",
    ] {
        let source = fs::read(format!("fixtures/{name}.wat"))
            .unwrap_or_else(|error| panic!("failed to read {name} fixture: {error}"));
        let component = componentize(&source)
            .unwrap_or_else(|error| panic!("failed to build {name} fixture: {error}"));
        fs::write(output.join(format!("{name}.component.wasm")), component)
            .unwrap_or_else(|error| panic!("failed to write {name} fixture: {error}"));
    }
    for name in ["forbidden-import", "wrong-interface"] {
        let source = fs::read(format!("fixtures/{name}.wat"))
            .unwrap_or_else(|error| panic!("failed to read {name} fixture: {error}"));
        let component = wat::parse_bytes(&source)
            .unwrap_or_else(|error| panic!("failed to parse {name} fixture: {error}"));
        fs::write(
            output.join(format!("{name}.component.wasm")),
            component.as_ref(),
        )
        .unwrap_or_else(|error| panic!("failed to write {name} fixture: {error}"));
    }
}

fn componentize(source: &[u8]) -> Result<Vec<u8>, String> {
    let mut module = wat::parse_bytes(source)
        .map(|bytes| bytes.into_owned())
        .map_err(|error| format!("WAT parsing failed: {error:#}"))?;
    let mut resolve = Resolve::default();
    let package = resolve
        .push_str("omarchygs-module.wit", WIT)
        .map_err(|error| format!("WIT parsing failed: {error:#}"))?;
    let world = resolve
        .select_world(&[package], Some(WORLD))
        .map_err(|error| format!("WIT world selection failed: {error:#}"))?;
    embed_component_metadata(&mut module, &resolve, world, StringEncoding::UTF8)
        .map_err(|error| format!("WIT metadata failed: {error:#}"))?;
    ComponentEncoder::default()
        .module(&module)
        .and_then(|encoder| encoder.validate(true).encode())
        .map_err(|error| format!("component encoding failed: {error:#}"))
}
