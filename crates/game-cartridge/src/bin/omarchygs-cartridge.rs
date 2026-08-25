use std::{
    env, fs,
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use omarchygs_game_cartridge::{
    CartridgeError, CatalogStatus, HostProfile, PublisherPrivateKey, PublisherPublicKey, Result,
    SecureCartridgeStore, baseline_host_profile, builder_sha256, create_release, export_sdk,
    generate_catalog_keypair, generate_keypair, install_cartridge, pack_directory, policy_report,
    read_catalog_private_key, read_catalog_public_key, read_private_key, read_public_key,
    revoke_cartridge, sign_catalog_policy, verify_archive, verify_release_directory,
    verify_sdk_directory,
};
use serde::Serialize;

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let document = serde_json::json!({
                "report_format": "omarchygs.cartridge.error/v1",
                "ok": false,
                "code": error.code(),
                "message": error.to_string(),
            });
            println!(
                "{}",
                serde_json::to_string(&document).expect("error report serializes")
            );
            ExitCode::from(2)
        }
    }
}

fn run(arguments: Vec<std::ffi::OsString>) -> Result<u8> {
    let mut arguments = arguments.into_iter();
    let command = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or(CartridgeError::InvalidActivation)?;
    let rest = arguments.map(PathBuf::from).collect::<Vec<_>>();
    match command.as_str() {
        "keygen" => command_keygen(&rest),
        "pack" => command_pack(&rest),
        "conform" => command_conform(&rest),
        "install" => command_install(&rest),
        "revoke" => command_revoke(&rest),
        "sdk-export" => command_sdk_export(&rest),
        "sdk-verify" => command_sdk_verify(&rest),
        "release" => command_release(&rest),
        "verify-release" => command_verify_release(&rest),
        "catalog-keygen" => command_catalog_keygen(&rest),
        "catalog-policy" => command_catalog_policy(&rest),
        "secure-import" => command_secure_import(&rest),
        _ => Err(CartridgeError::InvalidActivation),
    }
}

fn command_sdk_export(arguments: &[PathBuf]) -> Result<u8> {
    if arguments.len() != 1 {
        return Err(CartridgeError::InvalidSdk);
    }
    let identity = export_sdk(&arguments[0])?;
    print_json(&serde_json::json!({
        "report_format": "omarchygs.cartridge.sdk-export/v1",
        "ok": true,
        "identity": identity,
        "database_required": false,
        "provider_contacted": false,
        "platform_credentials_read": false,
    }))?;
    Ok(0)
}

fn command_sdk_verify(arguments: &[PathBuf]) -> Result<u8> {
    if arguments.len() != 1 {
        return Err(CartridgeError::InvalidSdk);
    }
    let identity = verify_sdk_directory(&arguments[0])?;
    print_json(&serde_json::json!({
        "report_format": "omarchygs.cartridge.sdk-verification/v1",
        "ok": true,
        "identity": identity,
        "database_required": false,
        "provider_contacted": false,
        "platform_credentials_read": false,
    }))?;
    Ok(0)
}

fn command_release(arguments: &[PathBuf]) -> Result<u8> {
    if !(5..=6).contains(&arguments.len()) {
        return Err(CartridgeError::InvalidRelease);
    }
    let key = read_private_key(&arguments[1])?;
    let source_revision = path_argument(&arguments[3])?;
    let host = read_host(arguments.get(5))?;
    let executable = env::current_exe()?;
    let report = create_release(
        &arguments[0],
        &key,
        &arguments[2],
        source_revision,
        &builder_sha256(&executable)?,
        &host,
        &arguments[4],
    )?;
    print_json(&report)?;
    Ok(0)
}

fn command_verify_release(arguments: &[PathBuf]) -> Result<u8> {
    if !(3..=4).contains(&arguments.len()) {
        return Err(CartridgeError::InvalidRelease);
    }
    let key = read_public_key(&arguments[1])?;
    let host = read_host(arguments.get(3))?;
    let release = verify_release_directory(&arguments[0], &key, &arguments[2], &host)?;
    print_json(&release.report())?;
    Ok(0)
}

fn command_catalog_keygen(arguments: &[PathBuf]) -> Result<u8> {
    if arguments.len() != 4 {
        return Err(CartridgeError::InvalidKey);
    }
    let authority_id = path_argument(&arguments[0])?;
    let key_id = path_argument(&arguments[1])?;
    let (private, public) = generate_catalog_keypair(key_id, authority_id)?;
    write_new_json(&arguments[2], &private, true)?;
    if let Err(error) = write_new_json(&arguments[3], &public, false) {
        let _ = fs::remove_file(&arguments[2]);
        return Err(error);
    }
    print_json(&serde_json::json!({
        "report_format": "omarchygs.catalog-keygen/v1",
        "ok": true,
        "authority_id": authority_id,
        "key_id": key_id,
    }))?;
    Ok(0)
}

fn command_catalog_policy(arguments: &[PathBuf]) -> Result<u8> {
    if !(8..=9).contains(&arguments.len()) {
        return Err(CartridgeError::InvalidCatalogPolicy);
    }
    let publisher_key = read_public_key(&arguments[1])?;
    let host = read_host(arguments.get(8))?;
    let release = verify_release_directory(&arguments[0], &publisher_key, &arguments[2], &host)?;
    let catalog_key = read_catalog_private_key(&arguments[3])?;
    let version = path_argument(&arguments[4])?
        .parse::<u64>()
        .map_err(|_| CartridgeError::InvalidCatalogPolicy)?;
    let status = parse_catalog_status(path_argument(&arguments[5])?)?;
    let reason = path_argument(&arguments[6])?;
    let signed = sign_catalog_policy(&release, &catalog_key, version, status, reason)?;
    write_new_json(&arguments[7], &signed, false)?;
    let public = catalog_key.public_key()?;
    let bytes = serde_json::to_vec(&signed)?;
    let policy = omarchygs_game_cartridge::verify_catalog_policy_bytes(&bytes, &public, &release)?;
    print_json(&policy_report(&policy))?;
    Ok(0)
}

fn command_secure_import(arguments: &[PathBuf]) -> Result<u8> {
    if !(6..=7).contains(&arguments.len()) {
        return Err(CartridgeError::InvalidActivation);
    }
    let publisher_key = read_public_key(&arguments[1])?;
    let host = read_host(arguments.get(6))?;
    let release = verify_release_directory(&arguments[0], &publisher_key, &arguments[2], &host)?;
    let policy_bytes = read_small_regular_file(&arguments[3], 256 * 1024)?;
    let catalog_key = read_catalog_public_key(&arguments[4])?;
    let store = SecureCartridgeStore::open_existing(&arguments[5])?;
    let report = store.import_release(&release, &policy_bytes, &catalog_key)?;
    print_json(&report)?;
    Ok(0)
}

fn command_keygen(arguments: &[PathBuf]) -> Result<u8> {
    if arguments.len() != 4 {
        return Err(CartridgeError::InvalidKey);
    }
    let publisher_id = path_argument(&arguments[0])?;
    let key_id = path_argument(&arguments[1])?;
    let (private, public) = generate_keypair(key_id, publisher_id)?;
    write_new_json(&arguments[2], &private, true)?;
    if let Err(error) = write_new_json(&arguments[3], &public, false) {
        let _ = fs::remove_file(&arguments[2]);
        return Err(error);
    }
    print_json(&serde_json::json!({
        "report_format": "omarchygs.cartridge.keygen/v1",
        "ok": true,
        "publisher_id": publisher_id,
        "key_id": key_id,
        "private_key_written": true,
        "public_key_written": true,
    }))?;
    Ok(0)
}

fn command_pack(arguments: &[PathBuf]) -> Result<u8> {
    if arguments.len() != 3 {
        return Err(CartridgeError::InvalidManifest);
    }
    let key: PublisherPrivateKey = read_private_key(&arguments[1])?;
    let archive = pack_directory(&arguments[0], &key)?;
    write_new(&arguments[2], &archive, false)?;
    let digest = sha256_hex(&archive);
    print_json(&serde_json::json!({
        "report_format": "omarchygs.cartridge.pack/v1",
        "ok": true,
        "archive_sha256": digest,
        "archive_bytes": archive.len(),
        "output_written": true,
    }))?;
    Ok(0)
}

fn command_conform(arguments: &[PathBuf]) -> Result<u8> {
    if !(2..=3).contains(&arguments.len()) {
        return Err(CartridgeError::InvalidActivation);
    }
    let key: PublisherPublicKey = read_public_key(&arguments[1])?;
    let host = read_host(arguments.get(2))?;
    let verified = verify_archive(&arguments[0], &key, &host)?;
    let report = verified.conformance_report();
    let compatible = report.conformant;
    print_json(&report)?;
    Ok(if compatible { 0 } else { 3 })
}

fn command_install(arguments: &[PathBuf]) -> Result<u8> {
    if !(3..=4).contains(&arguments.len()) {
        return Err(CartridgeError::InvalidActivation);
    }
    let key: PublisherPublicKey = read_public_key(&arguments[1])?;
    let host = read_host(arguments.get(3))?;
    let activation = install_cartridge(&arguments[0], &key, &host, &arguments[2])?;
    print_json(&serde_json::json!({
        "report_format": "omarchygs.cartridge.install/v1",
        "ok": true,
        "installed": true,
        "activation": activation,
        "provider_contacted": false,
        "database_required": false,
        "platform_credentials_read": false,
    }))?;
    Ok(0)
}

fn command_revoke(arguments: &[PathBuf]) -> Result<u8> {
    if arguments.len() != 3 {
        return Err(CartridgeError::InvalidActivation);
    }
    let digest = path_argument(&arguments[1])?;
    let reason = path_argument(&arguments[2])?;
    revoke_cartridge(&arguments[0], digest, reason)?;
    print_json(&serde_json::json!({
        "report_format": "omarchygs.cartridge.revoke/v1",
        "ok": true,
        "archive_sha256": digest,
        "revoked": true,
    }))?;
    Ok(0)
}

fn read_host(path: Option<&PathBuf>) -> Result<HostProfile> {
    match path {
        Some(path) => {
            const MAX_HOST_PROFILE_BYTES: u64 = 64 * 1024;
            let metadata = fs::symlink_metadata(path)?;
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                return Err(CartridgeError::UnsafeFilesystemPath);
            }
            let file = fs::File::open(path)?;
            let handle_metadata = file.metadata()?;
            if !handle_metadata.is_file() || handle_metadata.len() > MAX_HOST_PROFILE_BYTES {
                return Err(CartridgeError::LimitExceeded);
            }
            let mut bytes = Vec::with_capacity(handle_metadata.len() as usize);
            file.take(MAX_HOST_PROFILE_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_HOST_PROFILE_BYTES {
                return Err(CartridgeError::LimitExceeded);
            }
            Ok(serde_json::from_slice(&bytes)?)
        }
        None => Ok(baseline_host_profile()),
    }
}

fn read_small_regular_file(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > max_bytes
    {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    let file = fs::File::open(path)?;
    let handle_metadata = file.metadata()?;
    if !handle_metadata.is_file() || handle_metadata.len() > max_bytes {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    let mut bytes = Vec::with_capacity(handle_metadata.len() as usize);
    file.take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(CartridgeError::LimitExceeded);
    }
    Ok(bytes)
}

fn parse_catalog_status(value: &str) -> Result<CatalogStatus> {
    match value {
        "active" => Ok(CatalogStatus::Active),
        "deprecated" => Ok(CatalogStatus::Deprecated),
        "suspended" => Ok(CatalogStatus::Suspended),
        "revoked" => Ok(CatalogStatus::Revoked),
        "retired" => Ok(CatalogStatus::Retired),
        _ => Err(CartridgeError::InvalidCatalogPolicy),
    }
}

fn write_new_json(path: &Path, value: &impl Serialize, private: bool) -> Result<()> {
    write_new(path, &serde_json::to_vec(value)?, private)
}

fn write_new(path: &Path, bytes: &[u8], private: bool) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(if private { 0o600 } else { 0o444 });
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn print_json(value: &impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

fn path_argument(path: &Path) -> Result<&str> {
    path.to_str().ok_or(CartridgeError::InvalidActivation)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
