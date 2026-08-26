use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::{Read as _, Write as _},
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[path = "../operator_admin.rs"]
mod operator_admin;
#[path = "../registration_invites.rs"]
mod registration_invites;

use omarchy_gaming_system_server::{
    cartridge_catalog::{CatalogCommand, CatalogError, apply_catalog_command, list_inventory},
    marketplace_sync::{self, LocalCatalogConfig, MarketplaceSyncConfig, MarketplaceSyncError},
};
use omarchygs_game_cartridge::rich_2d_host_profile;
use operator_admin::{
    InvitationFilter, MAX_OPERATOR_DOCUMENT_BYTES, OperatorCommand, OperatorError, ReportFilter,
    apply_command, list_invitations, list_reports,
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

#[derive(Debug, Clone, Copy)]
enum AdminError {
    Operator(OperatorError),
    Marketplace(MarketplaceSyncError),
    Catalog(CatalogError),
}

impl AdminError {
    const fn code(self) -> &'static str {
        match self {
            Self::Operator(error) => error.code(),
            Self::Marketplace(error) => error.code(),
            Self::Catalog(error) => error.code(),
        }
    }
}

impl From<OperatorError> for AdminError {
    fn from(error: OperatorError) -> Self {
        Self::Operator(error)
    }
}

impl From<MarketplaceSyncError> for AdminError {
    fn from(error: MarketplaceSyncError) -> Self {
        Self::Marketplace(error)
    }
}

impl From<CatalogError> for AdminError {
    fn from(error: CatalogError) -> Self {
        Self::Catalog(error)
    }
}

async fn run() -> Result<(), AdminError> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let action = arguments
        .next()
        .ok_or(AdminError::Operator(OperatorError::InvalidInput))?;
    let database_url =
        env::var("DATABASE_URL").map_err(|_| AdminError::Operator(OperatorError::InvalidInput))?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .map_err(|_| AdminError::Operator(OperatorError::Internal))?;

    let output = if action == OsStr::new("reports") {
        let filter = arguments
            .next()
            .map(|value| {
                value
                    .to_str()
                    .ok_or(OperatorError::InvalidInput)
                    .and_then(ReportFilter::parse)
            })
            .transpose()?
            .unwrap_or(ReportFilter::Open);
        let limit = arguments
            .next()
            .map(|value| {
                value
                    .to_str()
                    .ok_or(OperatorError::InvalidInput)?
                    .parse::<u16>()
                    .map_err(|_| OperatorError::InvalidInput)
            })
            .transpose()?
            .unwrap_or(100);
        if arguments.next().is_some() {
            return Err(OperatorError::InvalidInput.into());
        }
        serde_json::to_value(list_reports(&pool, filter, limit).await?)
            .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else if action == OsStr::new("invites") {
        let filter = arguments
            .next()
            .map(|value| {
                value
                    .to_str()
                    .ok_or(OperatorError::InvalidInput)
                    .and_then(InvitationFilter::parse)
            })
            .transpose()?
            .unwrap_or(InvitationFilter::Issued);
        let limit = arguments
            .next()
            .map(|value| {
                value
                    .to_str()
                    .ok_or(OperatorError::InvalidInput)?
                    .parse::<u16>()
                    .map_err(|_| OperatorError::InvalidInput)
            })
            .transpose()?
            .unwrap_or(100);
        if arguments.next().is_some() {
            return Err(OperatorError::InvalidInput.into());
        }
        serde_json::to_value(list_invitations(&pool, filter, limit).await?)
            .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else if action == OsStr::new("apply") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Operator(OperatorError::InvalidInput))?;
        if arguments.next().is_some() {
            return Err(OperatorError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_OPERATOR_DOCUMENT_BYTES)?;
        let command: OperatorCommand =
            serde_json::from_slice(&document).map_err(|_| OperatorError::InvalidInput)?;
        command.validate()?;
        serde_json::to_value(apply_command(&pool, &command).await?)
            .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else if action == OsStr::new("marketplace-sync") {
        if arguments.next().is_some() {
            return Err(OperatorError::InvalidInput.into());
        }
        let config = MarketplaceSyncConfig::from_environment()?;
        serde_json::to_value(marketplace_sync::synchronize(&pool, &config).await?)
            .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else if action == OsStr::new("cartridges") {
        if arguments.next().is_some() {
            return Err(OperatorError::InvalidInput.into());
        }
        serde_json::to_value(list_inventory(&pool).await?)
            .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else if action == OsStr::new("catalog-apply") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Operator(OperatorError::InvalidInput))?;
        if arguments.next().is_some() {
            return Err(OperatorError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_OPERATOR_DOCUMENT_BYTES)?;
        let command: CatalogCommand = serde_json::from_slice(&document)
            .map_err(|_| AdminError::Catalog(CatalogError::InvalidInput))?;
        command.validate()?;
        let config = LocalCatalogConfig::from_environment()?;
        let store = config.open_store()?;
        serde_json::to_value(
            apply_catalog_command(
                &pool,
                &store,
                &config.marketplace_key,
                &rich_2d_host_profile(),
                &command,
            )
            .await?,
        )
        .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else {
        return Err(OperatorError::InvalidInput.into());
    };

    serde_json::to_writer(std::io::stdout(), &output)
        .map_err(|_| AdminError::Operator(OperatorError::Internal))?;
    writeln!(std::io::stdout()).map_err(|_| AdminError::Operator(OperatorError::Internal))?;
    Ok(())
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, OperatorError> {
    let link_metadata = std::fs::symlink_metadata(path).map_err(|_| OperatorError::InvalidInput)?;
    if link_metadata.file_type().is_symlink() || !link_metadata.is_file() {
        return Err(OperatorError::InvalidInput);
    }
    let file = File::open(path).map_err(|_| OperatorError::InvalidInput)?;
    let metadata = file.metadata().map_err(|_| OperatorError::InvalidInput)?;
    if !metadata.is_file()
        || metadata.len() > limit as u64
        || metadata.dev() != link_metadata.dev()
        || metadata.ino() != link_metadata.ino()
    {
        return Err(OperatorError::InvalidInput);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| OperatorError::InvalidInput)?;
    if bytes.is_empty() || bytes.len() > limit {
        Err(OperatorError::InvalidInput)
    } else {
        Ok(bytes)
    }
}
