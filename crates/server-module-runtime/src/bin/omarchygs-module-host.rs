//! Single-request host for one exact parent-provisioned server-module release.

use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read as _},
    path::PathBuf,
};

use omarchygs_server_module_runtime::{
    ExecutionTrust, HostReady, HostRequest, MAX_ARTIFACT_BYTES, ModuleRuntime, ModuleRuntimeError,
    decode_verifying_key, read_frame, verify_release_material, write_frame,
};
use uuid::Uuid;

fn main() {
    if run().is_err() {
        eprintln!("module_host_rejected");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ModuleRuntimeError> {
    let arguments = parse_arguments()?;
    let component_bytes = read_component(&arguments.component)?;
    let runtime = ModuleRuntime::compile_bytes(&component_bytes)?;
    runtime.readiness()?;
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_frame(&mut writer, &HostReady::measured()?)?;

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let request: HostRequest = read_frame(&mut reader)?;
    match arguments.failure.as_deref() {
        Some("exit") => std::process::exit(70),
        Some("hang") => loop {
            std::thread::park();
        },
        Some(_) => {
            return Err(ModuleRuntimeError::Contract(
                "unknown conformance failure".into(),
            ));
        }
        None => {}
    }
    let reviewed = verify_release_material(
        request.release.clone(),
        request.provenance.clone(),
        &arguments.trust,
        component_bytes,
    )?;
    write_frame(
        &mut writer,
        &runtime.execute_release(&request, &arguments.core_key, &reviewed),
    )
}

struct HostArguments {
    component: PathBuf,
    trust: ExecutionTrust,
    core_key: ed25519_dalek::VerifyingKey,
    failure: Option<String>,
}

fn parse_arguments() -> Result<HostArguments, ModuleRuntimeError> {
    let mut arguments = std::env::args().skip(1);
    let component = required_pair(&mut arguments, "--component")?;
    let publisher_key_id = required_pair(&mut arguments, "--publisher-key-id")?;
    let publisher_public_key = required_pair(&mut arguments, "--publisher-public-key")?;
    let provenance_key_id = required_pair(&mut arguments, "--provenance-key-id")?;
    let provenance_public_key = required_pair(&mut arguments, "--provenance-public-key")?;
    let provenance_class = required_pair(&mut arguments, "--provenance-class")?;
    let provenance_server_id = required_pair(&mut arguments, "--provenance-server-id")?;
    let core_key = required_pair(&mut arguments, "--core-public-key")?;
    let failure = match (arguments.next(), arguments.next(), arguments.next()) {
        (None, None, None) => None,
        (Some(flag), Some(value), None) if flag == "--conformance-failure" => Some(value),
        _ => {
            return Err(ModuleRuntimeError::Contract(
                "invalid module host arguments".into(),
            ));
        }
    };
    let provenance_server_id = if provenance_server_id == "none" {
        None
    } else {
        Some(
            Uuid::parse_str(&provenance_server_id)
                .map_err(|_| ModuleRuntimeError::Contract("invalid provenance server id".into()))?,
        )
    };
    Ok(HostArguments {
        component: PathBuf::from(component),
        trust: ExecutionTrust {
            publisher_key_id,
            publisher_public_key: decode_verifying_key(&publisher_public_key)?,
            provenance_key_id,
            provenance_public_key: decode_verifying_key(&provenance_public_key)?,
            provenance_class,
            provenance_server_id,
        },
        core_key: decode_verifying_key(&core_key)?,
        failure,
    })
}

fn required_pair(
    arguments: &mut impl Iterator<Item = String>,
    expected_flag: &str,
) -> Result<String, ModuleRuntimeError> {
    match (arguments.next(), arguments.next()) {
        (Some(flag), Some(value)) if flag == expected_flag && !value.is_empty() => Ok(value),
        _ => Err(ModuleRuntimeError::Contract(
            "invalid module host arguments".into(),
        )),
    }
}

fn read_component(path: &PathBuf) -> Result<Vec<u8>, ModuleRuntimeError> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let length = usize::try_from(metadata.len())
        .map_err(|_| ModuleRuntimeError::Contract("component size overflow".into()))?;
    if !metadata.is_file() || !(8..=MAX_ARTIFACT_BYTES).contains(&length) {
        return Err(ModuleRuntimeError::Contract(
            "component artifact is not a bounded regular file".into(),
        ));
    }
    let mut bytes = Vec::with_capacity(length);
    file.take(MAX_ARTIFACT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() != length || bytes.len() > MAX_ARTIFACT_BYTES || !bytes.starts_with(b"\0asm") {
        return Err(ModuleRuntimeError::Contract(
            "component artifact changed or is not binary Wasm".into(),
        ));
    }
    Ok(bytes)
}
