use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub const FORMAT_VERSION: u32 = 1;
pub const PRESENTATION_VERSION: u32 = 1;
pub const INTEGRITY_PATH: &str = "integrity.signed.json";
pub const MANIFEST_PATH: &str = "manifest.json";
pub const PRESENTATION_PATH: &str = "presentation.json";

pub const MAX_ARCHIVE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_EXPANDED_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_ENTRY_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 256;
pub const MAX_JSON_BYTES: usize = 1024 * 1024;
pub const MAX_SCHEMA_BYTES: usize = 256 * 1024;
pub const MAX_LOCALE_BYTES: usize = 256 * 1024;
pub const MAX_SCREENS: usize = 32;
pub const MAX_PRESENTATION_NODES: usize = 1024;
pub const MAX_GRID_SIDE: u16 = 64;
pub const MAX_SCHEMA_DEPTH: usize = 8;
pub const MAX_SCHEMA_NODES: usize = 256;
pub const MAX_LOCALIZATION_ENTRIES: usize = 4096;
pub const MAX_LOCALIZED_VALUE_CHARS: usize = 2048;
pub const MAX_RASTER_DIMENSION: u32 = 4096;
pub const MAX_RASTER_PIXELS: u64 = 32 * 1024 * 1024;
pub const MAX_DECODED_ASSET_BYTES: u64 = 128 * 1024 * 1024;
pub const MAX_AUDIO_DURATION_MS: u64 = 15 * 60 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CartridgeManifest {
    pub format_version: u32,
    pub game_key: String,
    pub publisher_id: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub sdk: VersionRange,
    pub presentation_protocol: VersionRange,
    pub display_name: String,
    pub entry_screen: String,
    pub required_capabilities: Vec<String>,
    pub optional_capabilities: Vec<OptionalCapability>,
    pub schemas: Vec<String>,
    pub locales: Vec<LocaleDescriptor>,
    pub assets: Vec<AssetDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VersionRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OptionalCapability {
    pub capability: String,
    pub fallback: CapabilityFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CapabilityFallback {
    Omit,
    Static,
    ReducedMotion,
    Muted,
    PlatformPlaceholder,
    SimplerCapability { capability: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocaleDescriptor {
    pub tag: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AssetDescriptor {
    pub path: String,
    pub media_type: AssetMediaType,
    pub decoded_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetMediaType {
    ImagePng,
    AudioWav,
}

impl AssetMediaType {
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::ImagePng => "image/png",
            Self::AudioWav => "audio/wav",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    pub format_version: u32,
    pub screens: Vec<Screen>,
    pub actions: Vec<ActionDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    pub id: String,
    pub title: String,
    pub view_schema: String,
    pub nodes: Vec<PresentationNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PresentationNode {
    Terminal {
        id: String,
        text_binding: String,
        accessible_label: String,
    },
    Grid {
        id: String,
        rows: u16,
        columns: u16,
        cells_binding: String,
        action: String,
        accessible_label: String,
    },
    Status {
        id: String,
        text_binding: String,
        accessible_label: String,
    },
    Button {
        id: String,
        label_binding: String,
        action: String,
        accessible_label: String,
    },
    Image {
        id: String,
        asset: String,
        accessible_label: String,
    },
    Meter {
        id: String,
        value_binding: String,
        minimum: i64,
        maximum: i64,
        accessible_label: String,
    },
    Sprite {
        id: String,
        asset: String,
        frame_width: u16,
        frame_height: u16,
        frame_count: u16,
        frames_per_second: u16,
        accessible_label: String,
    },
    ParticleField {
        id: String,
        particle_count: u16,
        preset: ParticlePreset,
        accessible_label: String,
    },
    AudioCue {
        id: String,
        asset: String,
        looped: bool,
        accessible_label: String,
    },
}

impl PresentationNode {
    pub(crate) fn id(&self) -> &str {
        match self {
            Self::Terminal { id, .. }
            | Self::Grid { id, .. }
            | Self::Status { id, .. }
            | Self::Button { id, .. }
            | Self::Image { id, .. }
            | Self::Meter { id, .. }
            | Self::Sprite { id, .. }
            | Self::ParticleField { id, .. }
            | Self::AudioCue { id, .. } => id,
        }
    }

    pub fn capability(&self) -> &'static str {
        match self {
            Self::Terminal { .. } => "presentation.terminal.v1",
            Self::Grid { .. } => "presentation.grid.v1",
            Self::Status { .. } => "presentation.status.v1",
            Self::Button { .. } => "presentation.button.v1",
            Self::Image { .. } => "presentation.image.v1",
            Self::Meter { .. } => "presentation.meter.v1",
            Self::Sprite { .. } => "presentation.sprite.v1",
            Self::ParticleField { .. } => "presentation.particles.v1",
            Self::AudioCue { .. } => "audio.effects.v1",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticlePreset {
    Stars,
    Sparks,
    Snow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionDefinition {
    pub id: String,
    pub payload_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IntegrityIndex {
    pub format_version: u32,
    pub publisher_id: String,
    pub files: Vec<IntegrityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IntegrityEntry {
    pub path: String,
    pub media_type: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedIntegrity {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureAlgorithm {
    Ed25519,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionCompatibility {
    Compatible,
    HostTooOld,
    HostTooNew,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OptionalFallbackSelection {
    pub capability: String,
    pub fallback: CapabilityFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityReport {
    pub compatible: bool,
    pub sdk: VersionCompatibility,
    pub presentation_protocol: VersionCompatibility,
    pub missing_required_capabilities: Vec<String>,
    pub selected_optional_fallbacks: Vec<OptionalFallbackSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostProfile {
    pub sdk_version: u32,
    pub presentation_protocol_version: u32,
    pub capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileProvenance {
    pub path: String,
    pub media_type: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConformanceReport {
    pub report_format: String,
    pub conformant: bool,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
    pub publisher_id: String,
    pub key_id: String,
    pub game_key: String,
    pub rules_version: u32,
    pub cartridge_version: u32,
    pub archive_bytes: u64,
    pub expanded_bytes: u64,
    pub files: Vec<FileProvenance>,
    pub compatibility: CompatibilityReport,
    pub installed: bool,
    pub provider_contacted: bool,
    pub database_required: bool,
    pub platform_credentials_read: bool,
}

#[derive(Debug, Clone)]
pub struct VerifiedCartridge {
    pub(crate) archive_bytes: Vec<u8>,
    pub(crate) archive_sha256: String,
    pub(crate) signed_identity_sha256: String,
    pub(crate) key_id: String,
    pub(crate) manifest: CartridgeManifest,
    pub(crate) presentation: Presentation,
    pub(crate) files: Vec<FileProvenance>,
    pub(crate) expanded_bytes: u64,
    pub(crate) compatibility: CompatibilityReport,
    pub(crate) authenticated_files: BTreeMap<String, Vec<u8>>,
}

impl VerifiedCartridge {
    pub fn archive_bytes(&self) -> &[u8] {
        &self.archive_bytes
    }

    pub fn archive_sha256(&self) -> &str {
        &self.archive_sha256
    }

    pub fn signed_identity_sha256(&self) -> &str {
        &self.signed_identity_sha256
    }

    pub fn manifest(&self) -> &CartridgeManifest {
        &self.manifest
    }

    pub fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    pub fn compatibility(&self) -> &CompatibilityReport {
        &self.compatibility
    }

    /// Returns bytes authenticated by the cartridge signature and validation
    /// pass. Callers never need to reopen publisher-controlled paths.
    pub fn authenticated_file(&self, path: &str) -> Option<&[u8]> {
        self.authenticated_files.get(path).map(Vec::as_slice)
    }

    pub fn conformance_report(&self) -> ConformanceReport {
        ConformanceReport {
            report_format: "omarchygs.cartridge.conformance/v1".to_owned(),
            conformant: self.compatibility.compatible,
            archive_sha256: self.archive_sha256.clone(),
            signed_identity_sha256: self.signed_identity_sha256.clone(),
            publisher_id: self.manifest.publisher_id.clone(),
            key_id: self.key_id.clone(),
            game_key: self.manifest.game_key.clone(),
            rules_version: self.manifest.rules_version,
            cartridge_version: self.manifest.cartridge_version,
            archive_bytes: self.archive_bytes.len() as u64,
            expanded_bytes: self.expanded_bytes,
            files: self.files.clone(),
            compatibility: self.compatibility.clone(),
            installed: false,
            provider_contacted: false,
            database_required: false,
            platform_credentials_read: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActivationRecord {
    pub format_version: u32,
    pub game_key: String,
    pub cartridge_version: u32,
    pub archive_sha256: String,
    pub signed_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RevocationRecord {
    pub format_version: u32,
    pub archive_sha256: String,
    pub reason: String,
}
