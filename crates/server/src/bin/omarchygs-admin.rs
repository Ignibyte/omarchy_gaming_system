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
    cartridge_catalog::{
        CatalogCommand, CatalogError, apply_catalog_command_with_sources,
        authorize_marketplace_trust, list_inventory,
    },
    marketplace_sync::{self, LocalCatalogConfig, MarketplaceSyncConfig, MarketplaceSyncError},
    operator_custom::{
        CustomImportCommand, CustomPolicyCommand, MAX_CUSTOM_COMMAND_BYTES,
        OperatorCustomAdminConfig, OperatorCustomError, apply_custom_policy,
        authorize_public_authority, import_custom_release,
    },
    server_module_custom::{
        CustomModuleAdminConfig, MAX_CUSTOM_MODULE_COMMAND_BYTES, apply_custom_lifecycle,
        decode_import_command, decode_lifecycle_command, import_custom_module,
    },
    server_modules::{
        ModuleError, ModuleLifecycleCommand, apply_lifecycle_command, list_module_inventory,
        prepare_restored_modules,
    },
};
use omarchygs_game_cartridge::rich_2d_host_profile;
use operator_admin::{
    InvitationFilter, MAX_OPERATOR_DOCUMENT_BYTES, OperatorCommand, OperatorError, ReportFilter,
    apply_command, list_invitations, list_reports,
};
use rustix::{
    fs::{CWD, Mode, OFlags, ResolveFlags, openat2},
    process::geteuid,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

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
    Custom(OperatorCustomError),
    Module(ModuleError),
}

impl AdminError {
    const fn code(self) -> &'static str {
        match self {
            Self::Operator(error) => error.code(),
            Self::Marketplace(error) => error.code(),
            Self::Catalog(error) => error.code(),
            Self::Custom(error) => error.code(),
            Self::Module(error) => error.code(),
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

impl From<OperatorCustomError> for AdminError {
    fn from(error: OperatorCustomError) -> Self {
        Self::Custom(error)
    }
}

impl From<ModuleError> for AdminError {
    fn from(error: ModuleError) -> Self {
        Self::Module(error)
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleRestoreCommand {
    format: String,
    operation_id: Uuid,
    actor: String,
    reason: String,
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
        let marketplace = LocalCatalogConfig::optional_from_environment()?;
        let custom = if env::var_os("OGS_CUSTOM_CARTRIDGE_PRIVATE_KEY").is_some() {
            Some(OperatorCustomAdminConfig::from_environment()?)
        } else {
            None
        };
        if marketplace.is_none() && custom.is_none() {
            return Err(CatalogError::Denied.into());
        }
        if let Some(config) = marketplace.as_ref() {
            authorize_marketplace_trust(&pool, config.marketplace_trust.channel_trust()).await?;
        }
        let custom_public = custom
            .as_ref()
            .map(OperatorCustomAdminConfig::public_config);
        if let Some(config) = custom_public.as_ref() {
            authorize_public_authority(&pool, config).await?;
        }
        let store = match (marketplace.as_ref(), custom.as_ref()) {
            (Some(marketplace), Some(custom)) => {
                if marketplace.store_root != custom.store_root {
                    return Err(CatalogError::Denied.into());
                }
                marketplace.open_store()?
            }
            (Some(marketplace), None) => marketplace.open_store()?,
            (None, Some(custom)) => custom.open_store()?,
            (None, None) => return Err(CatalogError::Denied.into()),
        };
        let marketplace_key = marketplace
            .as_ref()
            .map(LocalCatalogConfig::active_key)
            .transpose()?;
        serde_json::to_value(
            apply_catalog_command_with_sources(
                &pool,
                &store,
                marketplace_key.as_ref(),
                custom.as_ref().map(|config| &config.public_key),
                &rich_2d_host_profile(),
                &command,
            )
            .await?,
        )
        .map_err(|_| AdminError::Operator(OperatorError::Internal))?
    } else if action == OsStr::new("custom-cartridge-import") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Custom(OperatorCustomError::InvalidInput))?;
        if arguments.next().is_some() {
            return Err(OperatorCustomError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_CUSTOM_COMMAND_BYTES)
            .map_err(|_| AdminError::Custom(OperatorCustomError::InvalidInput))?;
        let command: CustomImportCommand = serde_json::from_slice(&document)
            .map_err(|_| AdminError::Custom(OperatorCustomError::InvalidInput))?;
        command.validate()?;
        let config = OperatorCustomAdminConfig::from_environment()?;
        serde_json::to_value(import_custom_release(&pool, &config, &command).await?)
            .map_err(|_| AdminError::Custom(OperatorCustomError::Internal))?
    } else if action == OsStr::new("custom-cartridge-policy-apply") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Custom(OperatorCustomError::InvalidInput))?;
        if arguments.next().is_some() {
            return Err(OperatorCustomError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_CUSTOM_COMMAND_BYTES)
            .map_err(|_| AdminError::Custom(OperatorCustomError::InvalidInput))?;
        let command: CustomPolicyCommand = serde_json::from_slice(&document)
            .map_err(|_| AdminError::Custom(OperatorCustomError::InvalidInput))?;
        command.validate()?;
        let config = OperatorCustomAdminConfig::from_environment()?;
        serde_json::to_value(apply_custom_policy(&pool, &config, &command).await?)
            .map_err(|_| AdminError::Custom(OperatorCustomError::Internal))?
    } else if action == OsStr::new("modules") {
        if arguments.next().is_some() {
            return Err(ModuleError::InvalidInput.into());
        }
        serde_json::to_value(list_module_inventory(&pool).await?)
            .map_err(|_| AdminError::Module(ModuleError::Internal))?
    } else if action == OsStr::new("module-apply") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Module(ModuleError::InvalidInput))?;
        if arguments.next().is_some() {
            return Err(ModuleError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_OPERATOR_DOCUMENT_BYTES)
            .map_err(|_| AdminError::Module(ModuleError::InvalidInput))?;
        let command: ModuleLifecycleCommand = serde_json::from_slice(&document)
            .map_err(|_| AdminError::Module(ModuleError::InvalidInput))?;
        serde_json::to_value(apply_lifecycle_command(&pool, &command).await?)
            .map_err(|_| AdminError::Module(ModuleError::Internal))?
    } else if action == OsStr::new("custom-module-import") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Module(ModuleError::InvalidInput))?;
        if !document_path.is_absolute() || arguments.next().is_some() {
            return Err(ModuleError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_CUSTOM_MODULE_COMMAND_BYTES)
            .map_err(|_| AdminError::Module(ModuleError::InvalidInput))?;
        let command = decode_import_command(&document)?;
        let config = CustomModuleAdminConfig::from_environment()?;
        serde_json::to_value(import_custom_module(&pool, &config, &command).await?)
            .map_err(|_| AdminError::Module(ModuleError::Internal))?
    } else if action == OsStr::new("custom-module-apply") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Module(ModuleError::InvalidInput))?;
        if !document_path.is_absolute() || arguments.next().is_some() {
            return Err(ModuleError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_CUSTOM_MODULE_COMMAND_BYTES)
            .map_err(|_| AdminError::Module(ModuleError::InvalidInput))?;
        let command = decode_lifecycle_command(&document)?;
        let config = CustomModuleAdminConfig::from_environment()?;
        serde_json::to_value(apply_custom_lifecycle(&pool, &config, &command).await?)
            .map_err(|_| AdminError::Module(ModuleError::Internal))?
    } else if action == OsStr::new("module-restore") {
        let document_path = arguments
            .next()
            .map(PathBuf::from)
            .ok_or(AdminError::Module(ModuleError::InvalidInput))?;
        if arguments.next().is_some() {
            return Err(ModuleError::InvalidInput.into());
        }
        let document = read_bounded(&document_path, MAX_OPERATOR_DOCUMENT_BYTES)
            .map_err(|_| AdminError::Module(ModuleError::InvalidInput))?;
        let command: ModuleRestoreCommand = serde_json::from_slice(&document)
            .map_err(|_| AdminError::Module(ModuleError::InvalidInput))?;
        if command.format != "omarchygs.server-module-restore-command/v1" {
            return Err(ModuleError::InvalidInput.into());
        }
        serde_json::to_value(
            prepare_restored_modules(&pool, command.operation_id, &command.actor, &command.reason)
                .await?,
        )
        .map_err(|_| AdminError::Module(ModuleError::Internal))?
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
    if link_metadata.file_type().is_symlink()
        || !link_metadata.is_file()
        || link_metadata.len() == 0
        || link_metadata.len() > limit as u64
        || link_metadata.uid() != geteuid().as_raw()
        || link_metadata.mode() & 0o777 != 0o600
        || link_metadata.nlink() != 1
    {
        return Err(OperatorError::InvalidInput);
    }
    let mut file = File::from(
        openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        )
        .map_err(|_| OperatorError::InvalidInput)?,
    );
    let metadata = file.metadata().map_err(|_| OperatorError::InvalidInput)?;
    if !trusted_command_metadata(&metadata, &link_metadata, limit) {
        return Err(OperatorError::InvalidInput);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    (&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| OperatorError::InvalidInput)?;
    let final_metadata = file.metadata().map_err(|_| OperatorError::InvalidInput)?;
    if bytes.is_empty()
        || bytes.len() > limit
        || !trusted_command_metadata(&final_metadata, &metadata, limit)
    {
        Err(OperatorError::InvalidInput)
    } else {
        Ok(bytes)
    }
}

fn trusted_command_metadata(
    current: &std::fs::Metadata,
    expected: &std::fs::Metadata,
    limit: usize,
) -> bool {
    current.is_file()
        && current.len() > 0
        && current.len() <= limit as u64
        && current.dev() == expected.dev()
        && current.ino() == expected.ino()
        && current.len() == expected.len()
        && current.uid() == expected.uid()
        && current.uid() == geteuid().as_raw()
        && current.gid() == expected.gid()
        && current.mode() == expected.mode()
        && current.mode() & 0o777 == 0o600
        && current.nlink() == 1
        && current.nlink() == expected.nlink()
        && current.mtime() == expected.mtime()
        && current.mtime_nsec() == expected.mtime_nsec()
        && current.ctime() == expected.ctime()
        && current.ctime_nsec() == expected.ctime_nsec()
}
