use std::{
    env,
    fs::File,
    io::{Read as _, Write as _},
    path::PathBuf,
    process::ExitCode,
};

use omarchy_game_provider::{
    ProviderError,
    model::{MAX_OPERATOR_DOCUMENT_BYTES, OperatorCommand},
    registry::ProviderRegistry,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{}", error.code());
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), ProviderError> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let action = arguments.next();
    let document_path = arguments.next().map(PathBuf::from);
    if action.as_deref() != Some(std::ffi::OsStr::new("apply"))
        || document_path.is_none()
        || arguments.next().is_some()
    {
        return Err(ProviderError::InvalidInput);
    }
    let database_url = env::var("DATABASE_URL").map_err(|_| ProviderError::InvalidInput)?;
    let document = read_bounded(
        &document_path.ok_or(ProviderError::InvalidInput)?,
        MAX_OPERATOR_DOCUMENT_BYTES,
    )?;
    let command: OperatorCommand =
        serde_json::from_slice(&document).map_err(|_| ProviderError::InvalidInput)?;
    command.validate()?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .map_err(|_| ProviderError::Internal)?;
    let receipt = ProviderRegistry::new(pool)
        .apply_operator_command(&command)
        .await?;
    serde_json::to_writer(std::io::stdout(), &receipt).map_err(|_| ProviderError::Internal)?;
    writeln!(std::io::stdout()).map_err(|_| ProviderError::Internal)?;
    Ok(())
}

fn read_bounded(path: &PathBuf, limit: usize) -> Result<Vec<u8>, ProviderError> {
    let file = File::open(path).map_err(|_| ProviderError::InvalidInput)?;
    let metadata = file.metadata().map_err(|_| ProviderError::InvalidInput)?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err(ProviderError::InvalidInput);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ProviderError::InvalidInput)?;
    if bytes.is_empty() || bytes.len() > limit {
        Err(ProviderError::InvalidInput)
    } else {
        Ok(bytes)
    }
}
