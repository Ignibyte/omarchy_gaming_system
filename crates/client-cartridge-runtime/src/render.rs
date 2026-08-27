//! Exact mounted-cartridge render-plan compilation.

use omarchygs_game_cartridge::CatalogPublicKey;
use omarchygs_game_cartridge_renderer::{
    PreparedPreview, RenderProfile, RendererPreferences, SurfaceState, compile_render_plan,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::{ClientCartridgeCache, CompanionError, Result, remote::selected_origin};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderRequest {
    pub server_origin: String,
    pub server_id: String,
    pub game_key: String,
    pub archive_sha256: String,
    pub admission_revision: u64,
    pub lifecycle_status: String,
    pub active_session_policy: String,
    #[serde(default)]
    pub screen_id: Option<String>,
    pub view: Value,
    #[serde(default)]
    pub preferences: RendererPreferences,
}

pub fn compile_mounted_render_plan(
    cache: &ClientCartridgeCache,
    request: &RenderRequest,
    trusted_marketplace_key: &CatalogPublicKey,
) -> Result<PreparedPreview> {
    let server_origin = selected_origin(&request.server_origin)?
        .origin()
        .ascii_serialization();
    let server_id = exact_uuid(&request.server_id)?;
    if request.active_session_policy != "continue"
        || !matches!(
            request.lifecycle_status.as_str(),
            "active" | "deprecated" | "retired"
        )
    {
        return Err(CompanionError::AdmissionChanged);
    }
    if request
        .screen_id
        .as_deref()
        .is_some_and(|screen_id| !valid_identifier(screen_id))
    {
        return Err(CompanionError::InvalidInput);
    }
    let resolution = cache.resolve_mounted(
        &server_origin,
        server_id,
        &request.game_key,
        &request.archive_sha256,
        request.admission_revision,
        trusted_marketplace_key,
    )?;
    let view_bytes = serde_json::to_vec(&request.view).map_err(|_| CompanionError::InvalidInput)?;
    compile_render_plan(
        resolution.cartridge(),
        request.screen_id.as_deref(),
        &view_bytes,
        RenderProfile::Rich2d,
        request.preferences,
        SurfaceState::Ready,
    )
    .map_err(|_| CompanionError::Render)
}

fn valid_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=96).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn exact_uuid(value: &str) -> Result<Uuid> {
    let parsed = Uuid::try_parse(value).map_err(|_| CompanionError::InvalidInput)?;
    if parsed.is_nil() || parsed.to_string() != value {
        Err(CompanionError::InvalidInput)
    } else {
        Ok(parsed)
    }
}
