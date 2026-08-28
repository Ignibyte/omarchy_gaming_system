//! Single-request host for one exact compiled-in server-module release.

use std::io::{self, BufReader, BufWriter};

use omarchygs_server_module_runtime::{
    FixtureKind, HostReady, HostRequest, ModuleRuntime, ModuleRuntimeError, decode_verifying_key,
    read_frame, write_frame,
};

fn main() {
    if run().is_err() {
        eprintln!("module_host_rejected");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ModuleRuntimeError> {
    let (kind, core_key, failure) = parse_arguments()?;
    let runtime = ModuleRuntime::compile(kind)?;
    runtime.readiness()?;
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_frame(&mut writer, &HostReady::measured()?)?;

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let request: HostRequest = read_frame(&mut reader)?;
    match failure.as_deref() {
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
    write_frame(&mut writer, &runtime.execute(&request, &core_key))
}

fn parse_arguments()
-> Result<(FixtureKind, ed25519_dalek::VerifyingKey, Option<String>), ModuleRuntimeError> {
    let mut arguments = std::env::args().skip(1);
    let fixture_flag = arguments.next();
    let fixture = arguments.next();
    let key_flag = arguments.next();
    let key = arguments.next();
    let failure = match (arguments.next(), arguments.next(), arguments.next()) {
        (None, None, None) => None,
        (Some(flag), Some(value), None) if flag == "--conformance-failure" => Some(value),
        _ => {
            return Err(ModuleRuntimeError::Contract(
                "invalid module host arguments".into(),
            ));
        }
    };
    if fixture_flag.as_deref() != Some("--fixture")
        || key_flag.as_deref() != Some("--core-public-key")
    {
        return Err(ModuleRuntimeError::Contract(
            "invalid module host arguments".into(),
        ));
    }
    Ok((
        FixtureKind::parse(
            fixture
                .as_deref()
                .ok_or_else(|| ModuleRuntimeError::Contract("missing fixture".into()))?,
        )?,
        decode_verifying_key(
            key.as_deref()
                .ok_or_else(|| ModuleRuntimeError::Contract("missing core key".into()))?,
        )?,
        failure,
    ))
}
