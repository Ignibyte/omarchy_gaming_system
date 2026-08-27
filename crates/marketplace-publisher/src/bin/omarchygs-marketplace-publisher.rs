use std::{env, path::PathBuf, process::ExitCode};

use omarchygs_marketplace_publisher::{
    PrepareOptions, ProbeFloor, ProbeOrigin, PublisherError, activate_publication,
    finalize_publication, offline_sign, prepare_publication, probe_mirrors, verify_current,
    verify_version,
};
use omarchygs_marketplace_trust::read_trust_root_public_key;
use serde::Serialize;

#[derive(Serialize)]
struct ErrorReceipt {
    format: &'static str,
    ok: bool,
    code: &'static str,
}

#[tokio::main]
async fn main() -> ExitCode {
    match dispatch(env::args().skip(1).collect()).await {
        Ok(value) => match serde_json::to_string(&value) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(_) => emit_error(PublisherError::Storage),
        },
        Err(error) => emit_error(error),
    }
}

async fn dispatch(arguments: Vec<String>) -> Result<serde_json::Value, PublisherError> {
    let (command, values) = arguments
        .split_first()
        .ok_or(PublisherError::InvalidInput)?;
    let receipt = match command.as_str() {
        "prepare" if values.len() == 7 => {
            let previous = (values[5] != "-").then(|| PathBuf::from(&values[5]));
            prepare_publication(PrepareOptions {
                plan_path: &PathBuf::from(&values[0]),
                input_root: &PathBuf::from(&values[1]),
                sdk_root: &PathBuf::from(&values[2]),
                catalog_private_key_path: &PathBuf::from(&values[3]),
                root_public_key_path: &PathBuf::from(&values[4]),
                previous_trust_path: previous.as_deref(),
                output_root: &PathBuf::from(&values[6]),
            })?
        }
        "offline-sign" if values.len() == 3 => offline_sign(
            &PathBuf::from(&values[0]),
            &PathBuf::from(&values[1]),
            &PathBuf::from(&values[2]),
        )?,
        "finalize" if values.len() == 4 => finalize_publication(
            &PathBuf::from(&values[0]),
            &PathBuf::from(&values[1]),
            &PathBuf::from(&values[2]),
            parse_time(&values[3])?,
        )?,
        "activate" if values.len() == 4 => activate_publication(
            &PathBuf::from(&values[0]),
            &values[1],
            &PathBuf::from(&values[2]),
            parse_time(&values[3])?,
        )?,
        "verify" if values.len() == 4 => {
            let store = PathBuf::from(&values[0]);
            let root = PathBuf::from(&values[2]);
            let now = parse_time(&values[3])?;
            if values[1] == "current" {
                verify_current(&store, &root, now)?
            } else {
                verify_version(&store, &values[1], &root, now)?
            }
        }
        "probe" if values.len() >= 7 && (values.len() - 5).is_multiple_of(2) => {
            let root = read_trust_root_public_key(&PathBuf::from(&values[0]))
                .map_err(|_| PublisherError::Rejected)?;
            let now = parse_time(&values[1])?;
            let floor = ProbeFloor {
                minimum_bundle_version: parse_time(&values[2])?,
                minimum_snapshot_version: parse_time(&values[3])?,
                expected_publication_sha256: (values[4] != "-").then(|| values[4].clone()),
            };
            let origins = values[5..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| ProbeOrigin {
                    channel_origin: pair[0].clone(),
                    marketplace_origin: pair[1].clone(),
                })
                .collect::<Vec<_>>();
            return serde_json::to_value(probe_mirrors(&origins, &root, &floor, now).await?)
                .map_err(|_| PublisherError::Storage);
        }
        _ => return Err(PublisherError::InvalidInput),
    };
    serde_json::to_value(receipt).map_err(|_| PublisherError::Storage)
}

fn parse_time(value: &str) -> Result<u64, PublisherError> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(PublisherError::InvalidInput)
}

fn emit_error(error: PublisherError) -> ExitCode {
    let receipt = ErrorReceipt {
        format: "omarchygs.marketplace-publication-error/v1",
        ok: false,
        code: error.code(),
    };
    eprintln!(
        "{}",
        serde_json::to_string(&receipt).unwrap_or_else(|_| {
            "{\"format\":\"omarchygs.marketplace-publication-error/v1\",\"ok\":false,\"code\":\"marketplace_publication_storage_failure\"}".to_owned()
        })
    );
    ExitCode::from(2)
}
