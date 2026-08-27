//! Single-request, single-release module host used only by the isolation proof.

use std::env;
use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use omarchygs_server_module_spike::{
    HostReady, HostRequest, ModuleRuntime, ProofError, read_bounded_artifact, read_frame,
    write_frame,
};

fn main() {
    if let Err(error) = run() {
        let _ = error;
        eprintln!("module host rejected startup/request");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ProofError> {
    let mut args = env::args().skip(1);
    let component_path = args
        .next()
        .ok_or_else(|| ProofError::Contract("missing component path".into()))?;
    let failure = match (args.next(), args.next(), args.next()) {
        (None, None, None) => None,
        (Some(flag), Some(value), None) if flag == "--proof-failure" => Some(value),
        _ => return Err(ProofError::Contract("invalid host arguments".into())),
    };
    let component_bytes = read_bounded_artifact(Path::new(&component_path))?;
    let runtime = ModuleRuntime::compile(&component_bytes)?;
    runtime.readiness()?;

    let ready = HostReady {
        format: "omarchygs.server-module-host-ready/v1".into(),
        component_ready: true,
        home_absent: !Path::new("/home").exists(),
        passwd_absent: !Path::new("/etc/passwd").exists(),
        server_environment_absent: server_environment_absent(),
        loopback_only: loopback_only()?,
        resident_kib: resident_kib()?,
    };
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_frame(&mut writer, &ready)?;

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let request: HostRequest = read_frame(&mut reader)?;
    match failure.as_deref() {
        Some("exit") => std::process::exit(70),
        Some("hang") => loop {
            std::thread::park();
        },
        Some(_) => return Err(ProofError::Contract("unknown proof failure".into())),
        None => {}
    }
    let response = runtime.execute(&request);
    write_frame(&mut writer, &response)
}

fn server_environment_absent() -> bool {
    let forbidden = [
        "DATABASE_URL",
        "OGS_SERVER_UUID",
        "OGS_TOKEN",
        "OMARCHYGS_OPERATOR_CUSTOM_PRIVATE_KEY",
        "OMARCHYGS_MARKETPLACE_KEY",
    ];
    !env::vars_os().any(|(key, _)| {
        key.to_str().is_some_and(|key| {
            forbidden.contains(&key)
                || key.starts_with("OMARCHYGS_SECRET_")
                || key.starts_with("OGS_SECRET_")
        })
    })
}

fn loopback_only() -> Result<bool, ProofError> {
    let network = fs::read_to_string("/proc/net/dev")?;
    Ok(network.lines().skip(2).all(|line| {
        line.split_once(':')
            .is_some_and(|(name, _)| name.trim() == "lo")
    }))
}

fn resident_kib() -> Result<u64, ProofError> {
    let status = fs::read_to_string("/proc/self/status")?;
    status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
        .ok_or_else(|| ProofError::Contract("host RSS is unavailable".into()))
}
