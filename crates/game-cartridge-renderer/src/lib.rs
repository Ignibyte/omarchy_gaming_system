//! Trusted render-plan compilation for verified OmarchyGS cartridges.
//!
//! This crate accepts only already-authenticated cartridge data, validates one
//! bounded view model, and emits inert tags consumed by repository-owned QML.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use omarchygs_game_cartridge::{
    AssetDescriptor, AssetMediaType, CapabilityFallback, ParticlePreset, PresentationNode,
    VerifiedCartridge, navigation_target,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const PLAN_FORMAT: &str = "omarchygs.render-plan/v1";
const MAX_BINDING_TEXT_CHARS: usize = 65_536;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("cartridge is incompatible with the selected renderer profile")]
    Incompatible,
    #[error("requested cartridge screen does not exist")]
    UnknownScreen,
    #[error("view model does not conform to the signed screen schema")]
    InvalidView,
    #[error("presentation binding has an invalid value")]
    InvalidBinding,
    #[error("presentation references unavailable authenticated content")]
    MissingAuthenticatedContent,
    #[error("render plan exceeds the selected profile budget")]
    BudgetExceeded,
    #[error("renderer output directory is not a private empty directory")]
    UnsafeOutputDirectory,
    #[error("I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("JSON operation failed")]
    Json(#[from] serde_json::Error),
}

impl RendererError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Incompatible => "renderer_incompatible",
            Self::UnknownScreen => "renderer_unknown_screen",
            Self::InvalidView => "renderer_invalid_view",
            Self::InvalidBinding => "renderer_invalid_binding",
            Self::MissingAuthenticatedContent => "renderer_missing_authenticated_content",
            Self::BudgetExceeded => "renderer_budget_exceeded",
            Self::UnsafeOutputDirectory => "renderer_unsafe_output_directory",
            Self::Io(_) => "renderer_io_failure",
            Self::Json(_) => "renderer_invalid_json",
        }
    }
}

pub type Result<T> = std::result::Result<T, RendererError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RenderProfile {
    Core,
    Rich2d,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RendererPreferences {
    pub scale: f64,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub muted_audio: bool,
}

impl Default for RendererPreferences {
    fn default() -> Self {
        Self {
            scale: 1.0,
            high_contrast: false,
            reduced_motion: false,
            muted_audio: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceState {
    Ready,
    Loading,
    Offline,
    Stale,
    Empty,
    ProtocolError,
    UnsupportedCapability,
    Revoked,
}

impl SurfaceState {
    fn platform_message(self) -> &'static str {
        match self {
            Self::Ready => "Ready",
            Self::Loading => "Loading game",
            Self::Offline => "Game provider offline",
            Self::Stale => "Game state may be stale",
            Self::Empty => "No game state available",
            Self::ProtocolError => "Game protocol error",
            Self::UnsupportedCapability => "Cartridge capability is not supported",
            Self::Revoked => "Cartridge has been revoked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileLimits {
    pub max_view_bytes: usize,
    pub max_plan_bytes: usize,
    pub max_nodes: usize,
    pub max_grid_cells: usize,
    pub max_images: usize,
    pub max_sprites: usize,
    pub max_particles: usize,
    pub max_audio_cues: usize,
    pub max_animations: usize,
    pub max_raster_dimension: u32,
    pub max_raster_pixels: u64,
    pub max_decoded_raster_bytes: u64,
    pub max_scene_decoded_raster_bytes: u64,
    pub soft_rss_mib: usize,
    pub hard_rss_mib: usize,
}

impl RenderProfile {
    pub fn limits(self) -> ProfileLimits {
        match self {
            Self::Core => ProfileLimits {
                max_view_bytes: 256 * 1024,
                max_plan_bytes: 1024 * 1024,
                max_nodes: 256,
                max_grid_cells: 1024,
                max_images: 32,
                max_sprites: 0,
                max_particles: 0,
                max_audio_cues: 0,
                max_animations: 32,
                max_raster_dimension: 1024,
                max_raster_pixels: 1024 * 1024,
                max_decoded_raster_bytes: 4 * 1024 * 1024,
                max_scene_decoded_raster_bytes: 16 * 1024 * 1024,
                soft_rss_mib: 256,
                hard_rss_mib: 384,
            },
            Self::Rich2d => ProfileLimits {
                max_view_bytes: 512 * 1024,
                max_plan_bytes: 2 * 1024 * 1024,
                max_nodes: 512,
                max_grid_cells: 4096,
                max_images: 64,
                max_sprites: 128,
                max_particles: 2048,
                max_audio_cues: 16,
                max_animations: 128,
                max_raster_dimension: 2048,
                max_raster_pixels: 4 * 1024 * 1024,
                max_decoded_raster_bytes: 16 * 1024 * 1024,
                max_scene_decoded_raster_bytes: 64 * 1024 * 1024,
                soft_rss_mib: 384,
                hard_rss_mib: 512,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CartridgeOrigin {
    pub publisher_id: String,
    pub game_key: String,
    pub cartridge_version: u32,
    pub archive_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RenderPlan {
    pub format: String,
    pub profile: RenderProfile,
    pub state: SurfaceState,
    pub state_message: String,
    pub origin: CartridgeOrigin,
    pub title: String,
    pub preferences: RendererPreferences,
    pub nodes: Vec<RenderedNode>,
    pub requested_actions_are_unconfirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RenderedNode {
    Terminal {
        id: String,
        text: String,
        accessible_label: String,
    },
    Grid {
        id: String,
        rows: u16,
        columns: u16,
        cells: Vec<String>,
        action: String,
        accessible_label: String,
    },
    Status {
        id: String,
        text: String,
        accessible_label: String,
    },
    Button {
        id: String,
        label: String,
        action: String,
        accessible_label: String,
    },
    Image {
        id: String,
        asset_token: String,
        accessible_label: String,
    },
    Meter {
        id: String,
        value: f64,
        minimum: i64,
        maximum: i64,
        accessible_label: String,
    },
    Sprite {
        id: String,
        asset_token: String,
        frame_width: u16,
        frame_height: u16,
        frame_count: u16,
        frames_per_second: u16,
        animated: bool,
        accessible_label: String,
    },
    ParticleField {
        id: String,
        particle_count: u16,
        preset: ParticlePreset,
        running: bool,
        accessible_label: String,
    },
    AudioCue {
        id: String,
        asset_token: String,
        looped: bool,
        muted: bool,
        accessible_label: String,
    },
    PlatformPlaceholder {
        id: String,
        message: String,
        accessible_label: String,
    },
}

#[derive(Debug, Clone)]
pub struct PreparedPreview {
    pub plan: RenderPlan,
    pub assets: BTreeMap<String, Vec<u8>>,
    pub screen_id: String,
    pub entry_screen_id: String,
    pub navigation: Vec<PreparedNavigation>,
}

/// One authenticated host-local destination emitted by a Button on the
/// prepared screen.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparedNavigation {
    pub action: String,
    pub target_screen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreviewReceipt {
    pub report_format: String,
    pub ok: bool,
    pub plan_file: String,
    pub asset_directory: String,
    pub plan_sha256: String,
    pub asset_count: usize,
    pub provider_contacted: bool,
    pub database_required: bool,
    pub platform_credentials_read: bool,
}

#[derive(Default)]
struct Usage {
    nodes: usize,
    plan_bytes: usize,
    grid_cells: usize,
    images: usize,
    sprites: usize,
    particles: usize,
    audio_cues: usize,
    animations: usize,
    decoded_raster_bytes: u64,
}

#[derive(Default)]
struct AuthenticatedAssetCache {
    token_by_path: BTreeMap<String, String>,
    path_by_token: BTreeMap<String, String>,
    #[cfg(test)]
    digest_computations: usize,
}

impl AuthenticatedAssetCache {
    fn token_for(&mut self, path: &str, extension: &str, bytes: &[u8]) -> String {
        if let Some(token) = self.token_by_path.get(path) {
            return token.clone();
        }
        #[cfg(test)]
        {
            self.digest_computations += 1;
        }
        let token = format!("{}.{}", sha256_hex(bytes), extension);
        self.token_by_path.insert(path.to_owned(), token.clone());
        self.path_by_token
            .entry(token.clone())
            .or_insert_with(|| path.to_owned());
        token
    }
}

#[derive(Default)]
struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(buffer.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "JSON size overflow"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn compile_render_plan(
    cartridge: &VerifiedCartridge,
    screen_id: Option<&str>,
    view_bytes: &[u8],
    profile: RenderProfile,
    preferences: RendererPreferences,
    state: SurfaceState,
) -> Result<PreparedPreview> {
    validate_preferences(preferences)?;
    let limits = profile.limits();
    if view_bytes.len() > limits.max_view_bytes {
        return Err(RendererError::BudgetExceeded);
    }
    let screen_id = screen_id.unwrap_or(&cartridge.manifest().entry_screen);
    let screen = cartridge
        .presentation()
        .screens
        .iter()
        .find(|screen| screen.id == screen_id)
        .ok_or(RendererError::UnknownScreen)?;
    let navigation = screen
        .nodes
        .iter()
        .filter_map(|node| match node {
            PresentationNode::Button { action, .. } => {
                navigation_target(action).map(|target| (action.clone(), target.to_owned()))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(action, target_screen)| PreparedNavigation {
            action,
            target_screen,
        })
        .collect::<Vec<_>>();
    let origin = CartridgeOrigin {
        publisher_id: cartridge.manifest().publisher_id.clone(),
        game_key: cartridge.manifest().game_key.clone(),
        cartridge_version: cartridge.manifest().cartridge_version,
        archive_sha256: cartridge.archive_sha256().to_owned(),
    };

    if state != SurfaceState::Ready {
        return finish_plan(
            RenderPlan {
                format: PLAN_FORMAT.to_owned(),
                profile,
                state,
                state_message: state.platform_message().to_owned(),
                origin,
                title: screen.title.clone(),
                preferences,
                nodes: Vec::new(),
                requested_actions_are_unconfirmed: true,
            },
            BTreeMap::new(),
            limits,
            screen_id,
            &cartridge.manifest().entry_screen,
            Vec::new(),
        );
    }
    if !cartridge.compatibility().compatible {
        return Err(RendererError::Incompatible);
    }
    let schema_bytes = cartridge
        .authenticated_file(&screen.view_schema)
        .ok_or(RendererError::MissingAuthenticatedContent)?;
    let schema: Value = serde_json::from_slice(schema_bytes)?;
    let view: Value = serde_json::from_slice(view_bytes).map_err(|_| RendererError::InvalidView)?;
    validate_instance(&schema, &view, 0)?;

    let declared_actions = cartridge
        .presentation()
        .actions
        .iter()
        .map(|action| action.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut assets = BTreeMap::new();
    let mut asset_cache = AuthenticatedAssetCache::default();
    let mut rendered = Vec::new();
    let mut usage = Usage::default();
    for node in &screen.nodes {
        let optional_fallback = cartridge
            .compatibility()
            .selected_optional_fallbacks
            .iter()
            .find(|selection| selection.capability == node.capability())
            .map(|selection| &selection.fallback);
        let is_optional = cartridge
            .manifest()
            .optional_capabilities
            .iter()
            .any(|optional| optional.capability == node.capability());
        let lowered = lower_node(
            cartridge,
            node,
            &view,
            preferences,
            optional_fallback,
            &declared_actions,
            &mut asset_cache,
        )?;
        let Some(lowered) = lowered else {
            continue;
        };
        let raster = raster_descriptor(cartridge, &lowered, &asset_cache)?;
        if let Err(error) = charge(&mut usage, &lowered, raster, limits) {
            if is_optional && matches!(error, RendererError::BudgetExceeded) {
                continue;
            }
            return Err(error);
        }
        publish_node_asset(cartridge, &lowered, &asset_cache, &mut assets)?;
        rendered.push(lowered);
    }
    finish_plan(
        RenderPlan {
            format: PLAN_FORMAT.to_owned(),
            profile,
            state,
            state_message: state.platform_message().to_owned(),
            origin,
            title: screen.title.clone(),
            preferences,
            nodes: rendered,
            requested_actions_are_unconfirmed: true,
        },
        assets,
        limits,
        screen_id,
        &cartridge.manifest().entry_screen,
        navigation,
    )
}

fn finish_plan(
    plan: RenderPlan,
    assets: BTreeMap<String, Vec<u8>>,
    limits: ProfileLimits,
    screen_id: &str,
    entry_screen_id: &str,
    navigation: Vec<PreparedNavigation>,
) -> Result<PreparedPreview> {
    if serialized_json_len(&plan)? > limits.max_plan_bytes {
        return Err(RendererError::BudgetExceeded);
    }
    Ok(PreparedPreview {
        plan,
        assets,
        screen_id: screen_id.to_owned(),
        entry_screen_id: entry_screen_id.to_owned(),
        navigation,
    })
}

fn validate_preferences(preferences: RendererPreferences) -> Result<()> {
    if !preferences.scale.is_finite() || !(0.75..=2.0).contains(&preferences.scale) {
        return Err(RendererError::InvalidView);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn lower_node(
    cartridge: &VerifiedCartridge,
    node: &PresentationNode,
    view: &Value,
    preferences: RendererPreferences,
    fallback: Option<&CapabilityFallback>,
    actions: &BTreeSet<&str>,
    asset_cache: &mut AuthenticatedAssetCache,
) -> Result<Option<RenderedNode>> {
    if matches!(fallback, Some(CapabilityFallback::Omit)) {
        return Ok(None);
    }
    if matches!(fallback, Some(CapabilityFallback::PlatformPlaceholder)) {
        return Ok(Some(RenderedNode::PlatformPlaceholder {
            id: node_id(node).to_owned(),
            message: "Optional cartridge visual is unavailable".to_owned(),
            accessible_label: node_accessible_label(node).to_owned(),
        }));
    }
    match node {
        PresentationNode::Terminal {
            id,
            text_binding,
            accessible_label,
        } => Ok(Some(RenderedNode::Terminal {
            id: id.clone(),
            text: binding_string(view, text_binding)?,
            accessible_label: accessible_label.clone(),
        })),
        PresentationNode::Grid {
            id,
            rows,
            columns,
            cells_binding,
            action,
            accessible_label,
        } => {
            require_action(actions, action)?;
            let cells = binding_strings(view, cells_binding)?;
            let expected = usize::from(*rows)
                .checked_mul(usize::from(*columns))
                .ok_or(RendererError::BudgetExceeded)?;
            if cells.len() != expected {
                return Err(RendererError::InvalidBinding);
            }
            Ok(Some(RenderedNode::Grid {
                id: id.clone(),
                rows: *rows,
                columns: *columns,
                cells,
                action: action.clone(),
                accessible_label: accessible_label.clone(),
            }))
        }
        PresentationNode::Status {
            id,
            text_binding,
            accessible_label,
        } => Ok(Some(RenderedNode::Status {
            id: id.clone(),
            text: binding_string(view, text_binding)?,
            accessible_label: accessible_label.clone(),
        })),
        PresentationNode::Button {
            id,
            label_binding,
            action,
            accessible_label,
        } => {
            require_action(actions, action)?;
            Ok(Some(RenderedNode::Button {
                id: id.clone(),
                label: binding_string(view, label_binding)?,
                action: action.clone(),
                accessible_label: accessible_label.clone(),
            }))
        }
        PresentationNode::Image {
            id,
            asset,
            accessible_label,
        } => Ok(Some(RenderedNode::Image {
            id: id.clone(),
            asset_token: authenticate_asset(
                cartridge,
                asset,
                AssetMediaType::ImagePng,
                asset_cache,
            )?,
            accessible_label: accessible_label.clone(),
        })),
        PresentationNode::Meter {
            id,
            value_binding,
            minimum,
            maximum,
            accessible_label,
        } => {
            let value = binding_number(view, value_binding)?;
            if value < *minimum as f64 || value > *maximum as f64 {
                return Err(RendererError::InvalidBinding);
            }
            Ok(Some(RenderedNode::Meter {
                id: id.clone(),
                value,
                minimum: *minimum,
                maximum: *maximum,
                accessible_label: accessible_label.clone(),
            }))
        }
        PresentationNode::Sprite {
            id,
            asset,
            frame_width,
            frame_height,
            frame_count,
            frames_per_second,
            accessible_label,
        } => {
            let asset_token =
                authenticate_asset(cartridge, asset, AssetMediaType::ImagePng, asset_cache)?;
            if matches!(
                fallback,
                Some(
                    CapabilityFallback::Static
                        | CapabilityFallback::ReducedMotion
                        | CapabilityFallback::SimplerCapability { .. }
                )
            ) {
                return Ok(Some(RenderedNode::Image {
                    id: id.clone(),
                    asset_token,
                    accessible_label: accessible_label.clone(),
                }));
            }
            Ok(Some(RenderedNode::Sprite {
                id: id.clone(),
                asset_token,
                frame_width: *frame_width,
                frame_height: *frame_height,
                frame_count: *frame_count,
                frames_per_second: *frames_per_second,
                animated: !preferences.reduced_motion,
                accessible_label: accessible_label.clone(),
            }))
        }
        PresentationNode::ParticleField {
            id,
            particle_count,
            preset,
            accessible_label,
        } => {
            let running = fallback.is_none() && !preferences.reduced_motion;
            Ok(Some(RenderedNode::ParticleField {
                id: id.clone(),
                particle_count: if running { *particle_count } else { 0 },
                preset: *preset,
                running,
                accessible_label: accessible_label.clone(),
            }))
        }
        PresentationNode::AudioCue {
            id,
            asset,
            looped,
            accessible_label,
        } => Ok(Some(RenderedNode::AudioCue {
            id: id.clone(),
            asset_token: authenticate_asset(
                cartridge,
                asset,
                AssetMediaType::AudioWav,
                asset_cache,
            )?,
            looped: *looped,
            muted: preferences.muted_audio || matches!(fallback, Some(CapabilityFallback::Muted)),
            accessible_label: accessible_label.clone(),
        })),
    }
}

fn node_id(node: &PresentationNode) -> &str {
    match node {
        PresentationNode::Terminal { id, .. }
        | PresentationNode::Grid { id, .. }
        | PresentationNode::Status { id, .. }
        | PresentationNode::Button { id, .. }
        | PresentationNode::Image { id, .. }
        | PresentationNode::Meter { id, .. }
        | PresentationNode::Sprite { id, .. }
        | PresentationNode::ParticleField { id, .. }
        | PresentationNode::AudioCue { id, .. } => id,
    }
}

fn node_accessible_label(node: &PresentationNode) -> &str {
    match node {
        PresentationNode::Terminal {
            accessible_label, ..
        }
        | PresentationNode::Grid {
            accessible_label, ..
        }
        | PresentationNode::Status {
            accessible_label, ..
        }
        | PresentationNode::Button {
            accessible_label, ..
        }
        | PresentationNode::Image {
            accessible_label, ..
        }
        | PresentationNode::Meter {
            accessible_label, ..
        }
        | PresentationNode::Sprite {
            accessible_label, ..
        }
        | PresentationNode::ParticleField {
            accessible_label, ..
        }
        | PresentationNode::AudioCue {
            accessible_label, ..
        } => accessible_label,
    }
}

fn require_action(actions: &BTreeSet<&str>, action: &str) -> Result<()> {
    if actions.contains(action) {
        Ok(())
    } else {
        Err(RendererError::InvalidBinding)
    }
}

fn authenticate_asset(
    cartridge: &VerifiedCartridge,
    path: &str,
    expected_type: AssetMediaType,
    cache: &mut AuthenticatedAssetCache,
) -> Result<String> {
    let descriptor = cartridge
        .manifest()
        .assets
        .iter()
        .find(|descriptor| descriptor.path == path && descriptor.media_type == expected_type)
        .ok_or(RendererError::MissingAuthenticatedContent)?;
    let bytes = cartridge
        .authenticated_file(&descriptor.path)
        .ok_or(RendererError::MissingAuthenticatedContent)?;
    let extension = match expected_type {
        AssetMediaType::ImagePng => "png",
        AssetMediaType::AudioWav => "wav",
    };
    Ok(cache.token_for(path, extension, bytes))
}

fn publish_node_asset(
    cartridge: &VerifiedCartridge,
    node: &RenderedNode,
    cache: &AuthenticatedAssetCache,
    output: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let token = match node {
        RenderedNode::Image { asset_token, .. }
        | RenderedNode::Sprite { asset_token, .. }
        | RenderedNode::AudioCue { asset_token, .. } => asset_token,
        _ => return Ok(()),
    };
    if output.contains_key(token) {
        return Ok(());
    }
    let path = cache
        .path_by_token
        .get(token)
        .ok_or(RendererError::MissingAuthenticatedContent)?;
    let bytes = cartridge
        .authenticated_file(path)
        .ok_or(RendererError::MissingAuthenticatedContent)?;
    output.insert(token.clone(), bytes.to_vec());
    Ok(())
}

fn raster_descriptor<'a>(
    cartridge: &'a VerifiedCartridge,
    node: &RenderedNode,
    cache: &AuthenticatedAssetCache,
) -> Result<Option<&'a AssetDescriptor>> {
    let token = match node {
        RenderedNode::Image { asset_token, .. } | RenderedNode::Sprite { asset_token, .. } => {
            asset_token
        }
        _ => return Ok(None),
    };
    let path = cache
        .path_by_token
        .get(token)
        .ok_or(RendererError::MissingAuthenticatedContent)?;
    cartridge
        .manifest()
        .assets
        .iter()
        .find(|descriptor| {
            descriptor.path == *path && descriptor.media_type == AssetMediaType::ImagePng
        })
        .map(Some)
        .ok_or(RendererError::MissingAuthenticatedContent)
}

fn charge(
    usage: &mut Usage,
    node: &RenderedNode,
    raster: Option<&AssetDescriptor>,
    limits: ProfileLimits,
) -> Result<()> {
    let node_bytes = serialized_json_len(node)?;
    let mut next = Usage {
        nodes: usage
            .nodes
            .checked_add(1)
            .ok_or(RendererError::BudgetExceeded)?,
        plan_bytes: usage
            .plan_bytes
            .checked_add(node_bytes)
            .ok_or(RendererError::BudgetExceeded)?,
        ..*usage
    };
    if let Some(raster) = raster {
        let width = raster
            .width
            .ok_or(RendererError::MissingAuthenticatedContent)?;
        let height = raster
            .height
            .ok_or(RendererError::MissingAuthenticatedContent)?;
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(RendererError::BudgetExceeded)?;
        if width > limits.max_raster_dimension
            || height > limits.max_raster_dimension
            || pixels > limits.max_raster_pixels
            || raster.decoded_bytes > limits.max_decoded_raster_bytes
        {
            return Err(RendererError::BudgetExceeded);
        }
        next.decoded_raster_bytes = next
            .decoded_raster_bytes
            .checked_add(raster.decoded_bytes)
            .ok_or(RendererError::BudgetExceeded)?;
    }
    match node {
        RenderedNode::Grid { cells, .. } => next.grid_cells += cells.len(),
        RenderedNode::Image { .. } => next.images += 1,
        RenderedNode::Sprite { animated, .. } => {
            next.sprites += 1;
            next.animations += usize::from(*animated);
        }
        RenderedNode::ParticleField {
            particle_count,
            running,
            ..
        } => {
            next.particles += usize::from(*particle_count);
            next.animations += usize::from(*running);
        }
        RenderedNode::AudioCue { .. } => next.audio_cues += 1,
        _ => {}
    }
    if next.nodes > limits.max_nodes
        || next.plan_bytes > limits.max_plan_bytes
        || next.grid_cells > limits.max_grid_cells
        || next.images > limits.max_images
        || next.sprites > limits.max_sprites
        || next.particles > limits.max_particles
        || next.audio_cues > limits.max_audio_cues
        || next.animations > limits.max_animations
        || next.decoded_raster_bytes > limits.max_scene_decoded_raster_bytes
    {
        return Err(RendererError::BudgetExceeded);
    }
    *usage = next;
    Ok(())
}

fn serialized_json_len(value: &impl Serialize) -> Result<usize> {
    let mut counter = ByteCounter::default();
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.0)
}

fn binding<'a>(view: &'a Value, path: &str) -> Result<&'a Value> {
    let mut value = view;
    for segment in path.split('.') {
        value = value
            .as_object()
            .and_then(|object| object.get(segment))
            .ok_or(RendererError::InvalidBinding)?;
    }
    Ok(value)
}

fn binding_string(view: &Value, path: &str) -> Result<String> {
    let value = binding(view, path)?
        .as_str()
        .ok_or(RendererError::InvalidBinding)?;
    safe_text(value).map(str::to_owned)
}

fn binding_strings(view: &Value, path: &str) -> Result<Vec<String>> {
    binding(view, path)?
        .as_array()
        .ok_or(RendererError::InvalidBinding)?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or(RendererError::InvalidBinding)
                .and_then(safe_text)
                .map(str::to_owned)
        })
        .collect()
}

fn binding_number(view: &Value, path: &str) -> Result<f64> {
    let value = binding(view, path)?
        .as_f64()
        .ok_or(RendererError::InvalidBinding)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(RendererError::InvalidBinding)
    }
}

fn safe_text(value: &str) -> Result<&str> {
    if value.chars().count() > MAX_BINDING_TEXT_CHARS
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        Err(RendererError::InvalidBinding)
    } else {
        Ok(value)
    }
}

fn validate_instance(schema: &Value, instance: &Value, depth: usize) -> Result<()> {
    if depth > 8 {
        return Err(RendererError::InvalidView);
    }
    let object = schema.as_object().ok_or(RendererError::InvalidView)?;
    if let Some(values) = object.get("enum").and_then(Value::as_array)
        && !values.contains(instance)
    {
        return Err(RendererError::InvalidView);
    }
    match object.get("type").and_then(Value::as_str) {
        Some("object") => {
            let instance = instance.as_object().ok_or(RendererError::InvalidView)?;
            let properties = object
                .get("properties")
                .and_then(Value::as_object)
                .ok_or(RendererError::InvalidView)?;
            if instance.keys().any(|key| !properties.contains_key(key)) {
                return Err(RendererError::InvalidView);
            }
            if let Some(required) = object.get("required").and_then(Value::as_array) {
                for key in required {
                    if !key.as_str().is_some_and(|key| instance.contains_key(key)) {
                        return Err(RendererError::InvalidView);
                    }
                }
            }
            for (key, value) in instance {
                validate_instance(&properties[key], value, depth + 1)?;
            }
        }
        Some("array") => {
            let instance = instance.as_array().ok_or(RendererError::InvalidView)?;
            let minimum = object.get("minItems").and_then(Value::as_u64).unwrap_or(0);
            let maximum = object
                .get("maxItems")
                .and_then(Value::as_u64)
                .ok_or(RendererError::InvalidView)?;
            if (instance.len() as u64) < minimum || instance.len() as u64 > maximum {
                return Err(RendererError::InvalidView);
            }
            let items = object.get("items").ok_or(RendererError::InvalidView)?;
            for value in instance {
                validate_instance(items, value, depth + 1)?;
            }
        }
        Some("string") => {
            let instance = instance.as_str().ok_or(RendererError::InvalidView)?;
            let length = instance.chars().count() as u64;
            let minimum = object.get("minLength").and_then(Value::as_u64).unwrap_or(0);
            let maximum = object
                .get("maxLength")
                .and_then(Value::as_u64)
                .ok_or(RendererError::InvalidView)?;
            if length < minimum || length > maximum {
                return Err(RendererError::InvalidView);
            }
        }
        Some("integer") => {
            if !(instance.is_i64() || instance.is_u64()) {
                return Err(RendererError::InvalidView);
            }
            validate_numeric_range(object, instance)?;
        }
        Some("number") => {
            if !instance.is_number() {
                return Err(RendererError::InvalidView);
            }
            validate_numeric_range(object, instance)?;
        }
        Some("boolean") if !instance.is_boolean() => return Err(RendererError::InvalidView),
        Some("null") if !instance.is_null() => return Err(RendererError::InvalidView),
        Some("boolean" | "null") => {}
        _ => return Err(RendererError::InvalidView),
    }
    Ok(())
}

fn validate_numeric_range(object: &serde_json::Map<String, Value>, value: &Value) -> Result<()> {
    let value = value.as_f64().ok_or(RendererError::InvalidView)?;
    if !value.is_finite()
        || object
            .get("minimum")
            .and_then(Value::as_f64)
            .is_some_and(|minimum| value < minimum)
        || object
            .get("maximum")
            .and_then(Value::as_f64)
            .is_some_and(|maximum| value > maximum)
    {
        return Err(RendererError::InvalidView);
    }
    Ok(())
}

pub fn write_prepared_preview(prepared: &PreparedPreview, output: &Path) -> Result<PreviewReceipt> {
    validate_output_directory(output)?;
    let plan_bytes = serde_json::to_vec(&prepared.plan)?;
    let plan_path = output.join("render-plan.json");
    write_new_read_only(&plan_path, &plan_bytes)?;
    let asset_directory = output.join("assets");
    create_private_directory(&asset_directory)?;
    for (token, bytes) in &prepared.assets {
        if !valid_asset_token(token) {
            return Err(RendererError::MissingAuthenticatedContent);
        }
        write_new_read_only(&asset_directory.join(token), bytes)?;
    }
    sync_directory(&asset_directory)?;
    sync_directory(output)?;
    Ok(PreviewReceipt {
        report_format: "omarchygs.cartridge-preview/v1".to_owned(),
        ok: true,
        plan_file: plan_path.to_string_lossy().into_owned(),
        asset_directory: asset_directory.to_string_lossy().into_owned(),
        plan_sha256: sha256_hex(&plan_bytes),
        asset_count: prepared.assets.len(),
        provider_contacted: false,
        database_required: false,
        platform_credentials_read: false,
    })
}

fn validate_output_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| RendererError::UnsafeOutputDirectory)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(RendererError::UnsafeOutputDirectory);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(RendererError::UnsafeOutputDirectory);
        }
    }
    if fs::read_dir(path)?.next().is_some() {
        return Err(RendererError::UnsafeOutputDirectory);
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    Ok(())
}

fn write_new_read_only(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o444);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<()> {
    fs::File::open(path)?.sync_all()?;
    Ok(())
}

pub fn valid_asset_token(value: &str) -> bool {
    let Some((digest, extension)) = value.rsplit_once('.') else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && matches!(extension, "png" | "wav")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn plan_path(output: &Path) -> PathBuf {
    output.join("render-plan.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticated_asset_cache_hashes_each_path_once() {
        let mut cache = AuthenticatedAssetCache::default();
        let first = cache.token_for("assets/hero.png", "png", b"authenticated bytes");
        let second = cache.token_for("assets/hero.png", "png", b"authenticated bytes");

        assert_eq!(first, second);
        assert_eq!(cache.digest_computations, 1);
        assert_eq!(cache.token_by_path.len(), 1);
        assert_eq!(cache.path_by_token.len(), 1);
    }

    #[test]
    fn profile_admission_bounds_each_raster_and_the_complete_scene() {
        let node = RenderedNode::Image {
            id: "portrait".to_owned(),
            asset_token: format!("{}.png", "a".repeat(64)),
            accessible_label: "Portrait".to_owned(),
        };
        let core_raster = AssetDescriptor {
            path: "assets/portrait.png".to_owned(),
            media_type: AssetMediaType::ImagePng,
            decoded_bytes: 4 * 1024 * 1024,
            width: Some(1024),
            height: Some(1024),
            duration_ms: None,
        };
        let mut core_usage = Usage::default();
        for _ in 0..4 {
            charge(
                &mut core_usage,
                &node,
                Some(&core_raster),
                RenderProfile::Core.limits(),
            )
            .unwrap();
        }
        assert!(matches!(
            charge(
                &mut core_usage,
                &node,
                Some(&core_raster),
                RenderProfile::Core.limits(),
            ),
            Err(RendererError::BudgetExceeded)
        ));

        let rich_raster = AssetDescriptor {
            path: "assets/scene.png".to_owned(),
            media_type: AssetMediaType::ImagePng,
            decoded_bytes: 16 * 1024 * 1024,
            width: Some(2048),
            height: Some(2048),
            duration_ms: None,
        };
        let mut rich_usage = Usage::default();
        assert!(
            charge(
                &mut rich_usage,
                &node,
                Some(&rich_raster),
                RenderProfile::Rich2d.limits(),
            )
            .is_ok()
        );
        assert!(matches!(
            charge(
                &mut Usage::default(),
                &node,
                Some(&rich_raster),
                RenderProfile::Core.limits(),
            ),
            Err(RendererError::BudgetExceeded)
        ));

        let formerly_legal_trigger = AssetDescriptor {
            path: "assets/oversized.png".to_owned(),
            media_type: AssetMediaType::ImagePng,
            decoded_bytes: 64 * 1024 * 1024,
            width: Some(4096),
            height: Some(4096),
            duration_ms: None,
        };
        assert!(matches!(
            charge(
                &mut Usage::default(),
                &node,
                Some(&formerly_legal_trigger),
                RenderProfile::Rich2d.limits(),
            ),
            Err(RendererError::BudgetExceeded)
        ));
    }
}
