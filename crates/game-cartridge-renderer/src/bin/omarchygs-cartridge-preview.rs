use std::{
    env, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use omarchygs_game_cartridge::{
    CartridgeError, core_host_profile, read_public_key, rich_2d_host_profile, verify_archive,
};
use omarchygs_game_cartridge_renderer::{
    RenderProfile, RendererError, RendererPreferences, SurfaceState, compile_render_plan,
    write_prepared_preview,
};
use serde::Serialize;
use thiserror::Error;

const MAX_PREFERENCES_BYTES: u64 = 64 * 1024;

#[derive(Debug, Error)]
enum CliError {
    #[error("invalid preview command arguments")]
    InvalidArguments,
    #[error(transparent)]
    Cartridge(#[from] CartridgeError),
    #[error(transparent)]
    Renderer(#[from] RendererError),
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("JSON operation failed")]
    Json(#[from] serde_json::Error),
}

impl CliError {
    fn code(&self) -> &'static str {
        match self {
            Self::InvalidArguments => "preview_invalid_arguments",
            Self::Cartridge(error) => error.code(),
            Self::Renderer(error) => error.code(),
            Self::Io(_) => "preview_io_failure",
            Self::Json(_) => "preview_invalid_json",
        }
    }
}

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let document = serde_json::json!({
                "report_format": "omarchygs.cartridge-preview.error/v1",
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

fn run(arguments: Vec<std::ffi::OsString>) -> Result<(), CliError> {
    if arguments.len() != 8 || arguments[0] != "prepare" {
        return Err(CliError::InvalidArguments);
    }
    let archive_path = PathBuf::from(&arguments[1]);
    let public_key_path = PathBuf::from(&arguments[2]);
    let profile = parse_profile(&arguments[3])?;
    let view_path = PathBuf::from(&arguments[4]);
    let state = parse_state(&arguments[5])?;
    let preferences_path = PathBuf::from(&arguments[6]);
    let output_path = PathBuf::from(&arguments[7]);

    let key = read_public_key(&public_key_path)?;
    let host = match profile {
        RenderProfile::Core => core_host_profile(),
        RenderProfile::Rich2d => rich_2d_host_profile(),
    };
    let cartridge = verify_archive(&archive_path, &key, &host)?;
    let view = read_bounded_regular_file(&view_path, profile.limits().max_view_bytes as u64)?;
    let preferences_bytes = read_bounded_regular_file(&preferences_path, MAX_PREFERENCES_BYTES)?;
    let preferences: RendererPreferences = serde_json::from_slice(&preferences_bytes)?;
    let prepared = compile_render_plan(&cartridge, None, &view, profile, preferences, state)?;
    let receipt = write_prepared_preview(&prepared, &output_path)?;
    print_json(&receipt)?;
    Ok(())
}

fn parse_profile(value: &std::ffi::OsStr) -> Result<RenderProfile, CliError> {
    match value.to_str() {
        Some("core") => Ok(RenderProfile::Core),
        Some("rich2d") => Ok(RenderProfile::Rich2d),
        _ => Err(CliError::InvalidArguments),
    }
}

fn parse_state(value: &std::ffi::OsStr) -> Result<SurfaceState, CliError> {
    match value.to_str() {
        Some("ready") => Ok(SurfaceState::Ready),
        Some("loading") => Ok(SurfaceState::Loading),
        Some("offline") => Ok(SurfaceState::Offline),
        Some("stale") => Ok(SurfaceState::Stale),
        Some("empty") => Ok(SurfaceState::Empty),
        Some("protocol_error") => Ok(SurfaceState::ProtocolError),
        Some("unsupported_capability") => Ok(SurfaceState::UnsupportedCapability),
        Some("revoked") => Ok(SurfaceState::Revoked),
        _ => Err(CliError::InvalidArguments),
    }
}

fn read_bounded_regular_file(path: &Path, limit: u64) -> Result<Vec<u8>, CliError> {
    let path_metadata = fs::symlink_metadata(path)?;
    if !path_metadata.file_type().is_file() || path_metadata.file_type().is_symlink() {
        return Err(CliError::InvalidArguments);
    }
    let file = fs::File::open(path)?;
    let handle_metadata = file.metadata()?;
    if !handle_metadata.is_file() || handle_metadata.len() > limit {
        return Err(CliError::InvalidArguments);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != handle_metadata.dev()
            || path_metadata.ino() != handle_metadata.ino()
        {
            return Err(CliError::InvalidArguments);
        }
    }
    let mut bytes = Vec::with_capacity(handle_metadata.len() as usize);
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit || bytes.len() as u64 != handle_metadata.len() {
        return Err(CliError::InvalidArguments);
    }
    Ok(bytes)
}

fn print_json(value: &impl Serialize) -> Result<(), CliError> {
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, value)?;
    stdout.write_all(b"\n")?;
    Ok(())
}
