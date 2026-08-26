use thiserror::Error;

#[derive(Debug, Error)]
pub enum CartridgeError {
    #[error("archive exceeds a cartridge limit")]
    LimitExceeded,
    #[error("archive is not canonical")]
    NonCanonicalArchive,
    #[error("archive contains an invalid entry")]
    InvalidArchiveEntry,
    #[error("archive contains an invalid path")]
    InvalidPath,
    #[error("archive contains a duplicate or unsorted path")]
    DuplicateOrUnsortedPath,
    #[error("cartridge signature is invalid")]
    InvalidSignature,
    #[error("publisher key does not match the cartridge")]
    PublisherMismatch,
    #[error("cartridge integrity index is invalid")]
    InvalidIntegrity,
    #[error("cartridge manifest is invalid")]
    InvalidManifest,
    #[error("cartridge presentation is invalid")]
    InvalidPresentation,
    #[error("cartridge schema is invalid")]
    InvalidSchema,
    #[error("cartridge localization is invalid")]
    InvalidLocalization,
    #[error("cartridge asset is invalid")]
    InvalidAsset,
    #[error("publisher key material is invalid")]
    InvalidKey,
    #[error("cartridge is incompatible with this host")]
    Incompatible,
    #[error("cartridge has been revoked")]
    Revoked,
    #[error("activation record is invalid")]
    InvalidActivation,
    #[error("path is not safe for this operation")]
    UnsafeFilesystemPath,
    #[error("OmarchyGS SDK export is invalid")]
    InvalidSdk,
    #[error("cartridge release attestation is invalid")]
    InvalidRelease,
    #[error("catalog lifecycle policy is invalid")]
    InvalidCatalogPolicy,
    #[error("catalog lifecycle policy denies this operation")]
    LifecycleDenied,
    #[error("marketplace snapshot is invalid")]
    InvalidMarketplaceSnapshot,
    #[error("secure cartridge store is unsupported on this platform")]
    UnsupportedSecureStore,
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("ZIP operation failed")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON operation failed")]
    Json(#[from] serde_json::Error),
}

impl CartridgeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::LimitExceeded => "cartridge_limit_exceeded",
            Self::NonCanonicalArchive => "non_canonical_archive",
            Self::InvalidArchiveEntry => "invalid_archive_entry",
            Self::InvalidPath => "invalid_cartridge_path",
            Self::DuplicateOrUnsortedPath => "duplicate_or_unsorted_path",
            Self::InvalidSignature => "invalid_cartridge_signature",
            Self::PublisherMismatch => "publisher_key_mismatch",
            Self::InvalidIntegrity => "invalid_integrity_index",
            Self::InvalidManifest => "invalid_cartridge_manifest",
            Self::InvalidPresentation => "invalid_cartridge_presentation",
            Self::InvalidSchema => "invalid_cartridge_schema",
            Self::InvalidLocalization => "invalid_cartridge_localization",
            Self::InvalidAsset => "invalid_cartridge_asset",
            Self::InvalidKey => "invalid_publisher_key",
            Self::Incompatible => "incompatible_cartridge",
            Self::Revoked => "revoked_cartridge",
            Self::InvalidActivation => "invalid_activation_record",
            Self::UnsafeFilesystemPath => "unsafe_filesystem_path",
            Self::InvalidSdk => "invalid_cartridge_sdk",
            Self::InvalidRelease => "invalid_cartridge_release",
            Self::InvalidCatalogPolicy => "invalid_catalog_policy",
            Self::LifecycleDenied => "cartridge_lifecycle_denied",
            Self::InvalidMarketplaceSnapshot => "invalid_marketplace_snapshot",
            Self::UnsupportedSecureStore => "secure_store_unsupported",
            Self::Io(_) => "cartridge_io_failure",
            Self::Zip(_) => "invalid_zip_archive",
            Self::Json(_) => "invalid_cartridge_json",
        }
    }
}

pub type Result<T> = std::result::Result<T, CartridgeError>;
