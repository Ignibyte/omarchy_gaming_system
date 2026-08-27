use std::collections::{BTreeMap, BTreeSet};

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    contract::{
        ActionDefinition, AssetDescriptor, AssetMediaType, CapabilityFallback, CartridgeManifest,
        FORMAT_VERSION, LocaleDescriptor, MAX_AUDIO_DURATION_MS, MAX_DECODED_ASSET_BYTES,
        MAX_GRID_SIDE, MAX_JSON_BYTES, MAX_LOCALE_BYTES, MAX_LOCALIZATION_ENTRIES,
        MAX_LOCALIZED_VALUE_CHARS, MAX_PRESENTATION_NODES, MAX_RASTER_DIMENSION, MAX_RASTER_PIXELS,
        MAX_SCHEMA_BYTES, MAX_SCHEMA_DEPTH, MAX_SCHEMA_NODES, MAX_SCREENS, OptionalCapability,
        PRESENTATION_VERSION, Presentation, PresentationNode, VerifiedCartridge,
    },
    error::{CartridgeError, Result},
    keys::valid_identifier,
};

/// Validate an untrusted action request against the exact signed entry-screen
/// emitter contract. Cartridges remain inert data: callers receive no general
/// expression, script, filesystem, credential, or network execution path.
pub fn validate_entry_screen_action(
    cartridge: &VerifiedCartridge,
    action: &str,
    payload: &serde_json::Value,
) -> Result<()> {
    validate_action_contract(
        cartridge.presentation(),
        &cartridge.manifest().entry_screen,
        action,
        payload,
    )
}

fn validate_action_contract(
    presentation: &Presentation,
    entry_screen_id: &str,
    action: &str,
    payload: &serde_json::Value,
) -> Result<()> {
    let entry_screen = presentation
        .screens
        .iter()
        .find(|screen| screen.id == entry_screen_id)
        .ok_or(CartridgeError::InvalidPresentation)?;
    let definition = presentation
        .actions
        .iter()
        .find(|definition| definition.id == action)
        .ok_or(CartridgeError::InvalidPresentation)?;
    let object = payload
        .as_object()
        .ok_or(CartridgeError::InvalidPresentation)?;

    let mut matching_emitter = false;
    for node in &entry_screen.nodes {
        match node {
            PresentationNode::Button {
                action: node_action,
                ..
            } if node_action == action => {
                matching_emitter = definition.payload_fields.is_empty() && object.is_empty();
            }
            PresentationNode::Grid {
                action: node_action,
                rows,
                columns,
                ..
            } if node_action == action => {
                let exact_fields = definition
                    .payload_fields
                    .iter()
                    .map(String::as_str)
                    .eq(["column", "row"])
                    && object.len() == 2
                    && object.contains_key("column")
                    && object.contains_key("row");
                let in_bounds = object
                    .get("column")
                    .and_then(serde_json::Value::as_u64)
                    .zip(object.get("row").and_then(serde_json::Value::as_u64))
                    .is_some_and(|(column, row)| {
                        column < u64::from(*columns) && row < u64::from(*rows)
                    });
                matching_emitter |= exact_fields && in_bounds;
            }
            _ => {}
        }
    }
    matching_emitter
        .then_some(())
        .ok_or(CartridgeError::InvalidPresentation)
}

pub(crate) fn parse_json<T: DeserializeOwned>(bytes: &[u8], limit: usize) -> Result<T> {
    if bytes.len() > limit {
        return Err(CartridgeError::LimitExceeded);
    }
    Ok(serde_json::from_slice(bytes)?)
}

pub(crate) fn canonical_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(value)?)
}

pub(crate) fn validate_manifest(manifest: &CartridgeManifest) -> Result<()> {
    if manifest.format_version != FORMAT_VERSION
        || !valid_identifier(&manifest.game_key)
        || !valid_identifier(&manifest.publisher_id)
        || manifest.rules_version == 0
        || manifest.cartridge_version == 0
        || !valid_version_range(manifest.sdk.min, manifest.sdk.max)
        || !valid_version_range(
            manifest.presentation_protocol.min,
            manifest.presentation_protocol.max,
        )
        || !valid_text(&manifest.display_name, 1, 128)
        || !valid_identifier(&manifest.entry_screen)
    {
        return Err(CartridgeError::InvalidManifest);
    }

    validate_sorted_unique(&manifest.required_capabilities, valid_capability)
        .map_err(|_| CartridgeError::InvalidManifest)?;
    validate_sorted_unique(&manifest.schemas, valid_schema_path)
        .map_err(|_| CartridgeError::InvalidManifest)?;
    if manifest.schemas.is_empty() {
        return Err(CartridgeError::InvalidManifest);
    }

    let mut optional_names = Vec::with_capacity(manifest.optional_capabilities.len());
    for optional in &manifest.optional_capabilities {
        if !valid_capability(&optional.capability) {
            return Err(CartridgeError::InvalidManifest);
        }
        if let CapabilityFallback::SimplerCapability { capability } = &optional.fallback
            && (!valid_capability(capability)
                || capability == &optional.capability
                || manifest
                    .required_capabilities
                    .binary_search(capability)
                    .is_err())
        {
            return Err(CartridgeError::InvalidManifest);
        }
        optional_names.push(optional.capability.clone());
    }
    validate_sorted_unique(&optional_names, valid_capability)
        .map_err(|_| CartridgeError::InvalidManifest)?;
    if optional_names
        .iter()
        .any(|name| manifest.required_capabilities.binary_search(name).is_ok())
    {
        return Err(CartridgeError::InvalidManifest);
    }

    validate_locales(&manifest.locales)?;
    validate_assets(&manifest.assets)?;
    Ok(())
}

fn validate_locales(locales: &[LocaleDescriptor]) -> Result<()> {
    let paths = locales
        .iter()
        .map(|item| item.path.clone())
        .collect::<Vec<_>>();
    validate_sorted_unique(&paths, valid_locale_path)
        .map_err(|_| CartridgeError::InvalidManifest)?;
    let tags = locales
        .iter()
        .map(|item| item.tag.clone())
        .collect::<Vec<_>>();
    validate_sorted_unique(&tags, valid_locale_tag).map_err(|_| CartridgeError::InvalidManifest)
}

fn validate_assets(assets: &[AssetDescriptor]) -> Result<()> {
    let paths = assets
        .iter()
        .map(|item| item.path.clone())
        .collect::<Vec<_>>();
    validate_sorted_unique(&paths, valid_asset_path)
        .map_err(|_| CartridgeError::InvalidManifest)?;
    for asset in assets {
        let expected = media_type_for_path(&asset.path).ok_or(CartridgeError::InvalidManifest)?;
        if expected != asset.media_type || asset.decoded_bytes == 0 {
            return Err(CartridgeError::InvalidManifest);
        }
        match asset.media_type {
            AssetMediaType::ImagePng => {
                if asset.width.is_none() || asset.height.is_none() || asset.duration_ms.is_some() {
                    return Err(CartridgeError::InvalidManifest);
                }
            }
            AssetMediaType::AudioWav => {
                if asset.width.is_some() || asset.height.is_some() || asset.duration_ms.is_none() {
                    return Err(CartridgeError::InvalidManifest);
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_presentation(
    presentation: &Presentation,
    entry_screen: &str,
    required_capabilities: &[String],
    optional_capabilities: &[OptionalCapability],
    schemas: &[String],
    assets: &[AssetDescriptor],
) -> Result<()> {
    if presentation.format_version != PRESENTATION_VERSION
        || presentation.screens.is_empty()
        || presentation.screens.len() > MAX_SCREENS
        || presentation.actions.len() > MAX_PRESENTATION_NODES
    {
        return Err(CartridgeError::InvalidPresentation);
    }

    let mut screen_ids = BTreeSet::new();
    let mut node_ids = BTreeSet::new();
    let mut node_count = 0usize;
    for screen in &presentation.screens {
        if !valid_identifier(&screen.id)
            || !screen_ids.insert(screen.id.as_str())
            || !valid_text(&screen.title, 1, 128)
            || !valid_schema_path(&screen.view_schema)
            || schemas.binary_search(&screen.view_schema).is_err()
            || screen.nodes.is_empty()
        {
            return Err(CartridgeError::InvalidPresentation);
        }
        node_count = node_count
            .checked_add(screen.nodes.len())
            .ok_or(CartridgeError::LimitExceeded)?;
        if node_count > MAX_PRESENTATION_NODES {
            return Err(CartridgeError::LimitExceeded);
        }
        for node in &screen.nodes {
            if !valid_identifier(node.id()) || !node_ids.insert(node.id()) {
                return Err(CartridgeError::InvalidPresentation);
            }
            match node {
                PresentationNode::Terminal {
                    text_binding,
                    accessible_label,
                    ..
                } => {
                    if !valid_binding(text_binding) || !valid_text(accessible_label, 1, 256) {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::Status {
                    text_binding,
                    accessible_label,
                    ..
                } => {
                    if !valid_binding(text_binding) || !valid_text(accessible_label, 1, 256) {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::Grid {
                    rows,
                    columns,
                    cells_binding,
                    action,
                    accessible_label,
                    ..
                } => {
                    if *rows == 0
                        || *columns == 0
                        || *rows > MAX_GRID_SIDE
                        || *columns > MAX_GRID_SIDE
                        || !valid_binding(cells_binding)
                        || !valid_identifier(action)
                        || !valid_text(accessible_label, 1, 256)
                    {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::Button {
                    label_binding,
                    action,
                    accessible_label,
                    ..
                } => {
                    if !valid_binding(label_binding)
                        || !valid_identifier(action)
                        || !valid_text(accessible_label, 1, 256)
                    {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::Image {
                    asset,
                    accessible_label,
                    ..
                } => {
                    validate_node_asset(assets, asset, AssetMediaType::ImagePng)?;
                    if !valid_text(accessible_label, 1, 256) {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::Meter {
                    value_binding,
                    minimum,
                    maximum,
                    accessible_label,
                    ..
                } => {
                    if !valid_binding(value_binding)
                        || minimum >= maximum
                        || *minimum < -9_007_199_254_740_991
                        || *maximum > 9_007_199_254_740_991
                        || !valid_text(accessible_label, 1, 256)
                    {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::Sprite {
                    asset,
                    frame_width,
                    frame_height,
                    frame_count,
                    frames_per_second,
                    accessible_label,
                    ..
                } => {
                    if *frame_width == 0 || *frame_height == 0 {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                    let descriptor = validate_node_asset(assets, asset, AssetMediaType::ImagePng)?;
                    let frame_slots = descriptor
                        .width
                        .zip(descriptor.height)
                        .map(|(width, height)| {
                            (width / u32::from(*frame_width))
                                .saturating_mul(height / u32::from(*frame_height))
                        })
                        .unwrap_or(0);
                    if *frame_count == 0
                        || *frame_count > 1024
                        || *frames_per_second == 0
                        || *frames_per_second > 120
                        || frame_slots < u32::from(*frame_count)
                        || !valid_text(accessible_label, 1, 256)
                    {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::ParticleField {
                    particle_count,
                    accessible_label,
                    ..
                } => {
                    if *particle_count == 0
                        || *particle_count > 4096
                        || !valid_text(accessible_label, 1, 256)
                    {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
                PresentationNode::AudioCue {
                    asset,
                    accessible_label,
                    ..
                } => {
                    validate_node_asset(assets, asset, AssetMediaType::AudioWav)?;
                    if !valid_text(accessible_label, 1, 256) {
                        return Err(CartridgeError::InvalidPresentation);
                    }
                }
            }
            let capability = node.capability();
            if required_capabilities
                .binary_search_by(|candidate| candidate.as_str().cmp(capability))
                .is_ok()
            {
                continue;
            }
            let optional = optional_capabilities
                .iter()
                .find(|candidate| candidate.capability == capability)
                .ok_or(CartridgeError::InvalidPresentation)?;
            validate_node_fallback(node, &optional.fallback)?;
        }
    }
    if !screen_ids.contains(entry_screen) {
        return Err(CartridgeError::InvalidPresentation);
    }

    let mut action_ids = BTreeSet::new();
    for action in &presentation.actions {
        validate_action(action, &mut action_ids)?;
    }
    let action_contracts = presentation
        .actions
        .iter()
        .map(|action| (action.id.as_str(), action.payload_fields.as_slice()))
        .collect::<BTreeMap<_, _>>();
    for screen in &presentation.screens {
        for node in &screen.nodes {
            match node {
                PresentationNode::Grid { action, .. }
                    if action_contracts.get(action.as_str()).is_none_or(|fields| {
                        !fields.iter().map(String::as_str).eq(["column", "row"])
                    }) =>
                {
                    return Err(CartridgeError::InvalidPresentation);
                }
                PresentationNode::Button { action, .. }
                    if action_contracts
                        .get(action.as_str())
                        .is_none_or(|fields| !fields.is_empty()) =>
                {
                    return Err(CartridgeError::InvalidPresentation);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn validate_node_asset<'a>(
    assets: &'a [AssetDescriptor],
    path: &str,
    media_type: AssetMediaType,
) -> Result<&'a AssetDescriptor> {
    if !valid_asset_path(path) {
        return Err(CartridgeError::InvalidPresentation);
    }
    let descriptor = assets
        .iter()
        .find(|descriptor| descriptor.path == path)
        .ok_or(CartridgeError::InvalidPresentation)?;
    if descriptor.media_type != media_type {
        return Err(CartridgeError::InvalidPresentation);
    }
    Ok(descriptor)
}

fn validate_node_fallback(node: &PresentationNode, fallback: &CapabilityFallback) -> Result<()> {
    let valid = match node {
        PresentationNode::Image { .. } => matches!(
            fallback,
            CapabilityFallback::Omit | CapabilityFallback::PlatformPlaceholder
        ),
        PresentationNode::Sprite { .. } => match fallback {
            CapabilityFallback::Omit
            | CapabilityFallback::Static
            | CapabilityFallback::ReducedMotion
            | CapabilityFallback::PlatformPlaceholder => true,
            CapabilityFallback::SimplerCapability { capability } => {
                capability == "presentation.image.v1"
            }
            CapabilityFallback::Muted => false,
        },
        PresentationNode::ParticleField { .. } => matches!(
            fallback,
            CapabilityFallback::Omit
                | CapabilityFallback::Static
                | CapabilityFallback::ReducedMotion
        ),
        PresentationNode::AudioCue { .. } => {
            matches!(
                fallback,
                CapabilityFallback::Omit | CapabilityFallback::Muted
            )
        }
        PresentationNode::Terminal { .. }
        | PresentationNode::Grid { .. }
        | PresentationNode::Status { .. }
        | PresentationNode::Button { .. }
        | PresentationNode::Meter { .. } => false,
    };
    if valid {
        Ok(())
    } else {
        Err(CartridgeError::InvalidPresentation)
    }
}

fn validate_action<'a>(action: &'a ActionDefinition, ids: &mut BTreeSet<&'a str>) -> Result<()> {
    if !valid_identifier(&action.id) || !ids.insert(&action.id) || action.payload_fields.len() > 32
    {
        return Err(CartridgeError::InvalidPresentation);
    }
    validate_sorted_unique(&action.payload_fields, valid_binding)
        .map_err(|_| CartridgeError::InvalidPresentation)
}

pub(crate) fn validate_schema(bytes: &[u8]) -> Result<Value> {
    let value: Value = parse_json(bytes, MAX_SCHEMA_BYTES)?;
    let root = value.as_object().ok_or(CartridgeError::InvalidSchema)?;
    if root.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
    {
        return Err(CartridgeError::InvalidSchema);
    }
    let mut nodes = 0usize;
    validate_schema_node(&value, 0, &mut nodes)?;
    Ok(value)
}

fn validate_schema_node(value: &Value, depth: usize, nodes: &mut usize) -> Result<()> {
    *nodes += 1;
    if depth > MAX_SCHEMA_DEPTH || *nodes > MAX_SCHEMA_NODES {
        return Err(CartridgeError::LimitExceeded);
    }
    let object = value.as_object().ok_or(CartridgeError::InvalidSchema)?;
    const ALLOWED: &[&str] = &[
        "$schema",
        "type",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "enum",
        "minimum",
        "maximum",
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
        "description",
    ];
    if object.keys().any(|key| !ALLOWED.contains(&key.as_str())) {
        return Err(CartridgeError::InvalidSchema);
    }
    if let Some(schema) = object.get("$schema")
        && (depth != 0 || schema.as_str() != Some("https://json-schema.org/draft/2020-12/schema"))
    {
        return Err(CartridgeError::InvalidSchema);
    }
    if let Some(description) = object.get("description")
        && !description
            .as_str()
            .is_some_and(|text| valid_text(text, 0, 1024))
    {
        return Err(CartridgeError::InvalidSchema);
    }
    let schema_type = object.get("type").and_then(Value::as_str);
    if !matches!(
        schema_type,
        Some("object" | "array" | "string" | "integer" | "number" | "boolean" | "null")
    ) {
        return Err(CartridgeError::InvalidSchema);
    }
    if ((object.contains_key("properties")
        || object.contains_key("required")
        || object.contains_key("additionalProperties"))
        && schema_type != Some("object"))
        || ((object.contains_key("items")
            || object.contains_key("minItems")
            || object.contains_key("maxItems"))
            && schema_type != Some("array"))
        || ((object.contains_key("minLength") || object.contains_key("maxLength"))
            && schema_type != Some("string"))
        || ((object.contains_key("minimum") || object.contains_key("maximum"))
            && !matches!(schema_type, Some("integer" | "number")))
    {
        return Err(CartridgeError::InvalidSchema);
    }
    if let Some(properties) = object.get("properties") {
        let properties = properties
            .as_object()
            .ok_or(CartridgeError::InvalidSchema)?;
        if schema_type != Some("object")
            || object.get("additionalProperties") != Some(&Value::Bool(false))
            || properties.len() > 128
        {
            return Err(CartridgeError::InvalidSchema);
        }
        for (key, schema) in properties {
            if !valid_identifier(key) {
                return Err(CartridgeError::InvalidSchema);
            }
            validate_schema_node(schema, depth + 1, nodes)?;
        }
    }
    if let Some(required) = object.get("required") {
        let required = required.as_array().ok_or(CartridgeError::InvalidSchema)?;
        let values = required
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or(CartridgeError::InvalidSchema)
            })
            .collect::<Result<Vec<_>>>()?;
        validate_sorted_unique(&values, valid_identifier)
            .map_err(|_| CartridgeError::InvalidSchema)?;
        let properties = object
            .get("properties")
            .and_then(Value::as_object)
            .ok_or(CartridgeError::InvalidSchema)?;
        if values.iter().any(|key| !properties.contains_key(key)) {
            return Err(CartridgeError::InvalidSchema);
        }
    }
    if schema_type == Some("object")
        && object.get("additionalProperties") != Some(&Value::Bool(false))
    {
        return Err(CartridgeError::InvalidSchema);
    }
    if let Some(items) = object.get("items") {
        if schema_type != Some("array") {
            return Err(CartridgeError::InvalidSchema);
        }
        validate_schema_node(items, depth + 1, nodes)?;
    }
    if schema_type == Some("array") {
        let max = object.get("maxItems").and_then(Value::as_u64);
        if !object.contains_key("items") || max.is_none_or(|value| value > 4096) {
            return Err(CartridgeError::InvalidSchema);
        }
        validate_u64_pair(object, "minItems", "maxItems")?;
    }
    if schema_type == Some("string") {
        let max = object.get("maxLength").and_then(Value::as_u64);
        if max.is_none_or(|value| value > 65_536) {
            return Err(CartridgeError::InvalidSchema);
        }
        validate_u64_pair(object, "minLength", "maxLength")?;
    }
    if let Some(values) = object.get("enum") {
        let values = values.as_array().ok_or(CartridgeError::InvalidSchema)?;
        if values.is_empty() || values.len() > 256 {
            return Err(CartridgeError::InvalidSchema);
        }
    }
    let minimum = object
        .get("minimum")
        .map(|value| value.as_f64().ok_or(CartridgeError::InvalidSchema))
        .transpose()?;
    let maximum = object
        .get("maximum")
        .map(|value| value.as_f64().ok_or(CartridgeError::InvalidSchema))
        .transpose()?;
    if minimum.is_some_and(|value| !value.is_finite())
        || maximum.is_some_and(|value| !value.is_finite())
        || minimum
            .zip(maximum)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
    {
        return Err(CartridgeError::InvalidSchema);
    }
    Ok(())
}

fn validate_u64_pair(object: &serde_json::Map<String, Value>, min: &str, max: &str) -> Result<()> {
    let minimum = object
        .get(min)
        .map(|value| value.as_u64().ok_or(CartridgeError::InvalidSchema))
        .transpose()?
        .unwrap_or(0);
    let maximum = object
        .get(max)
        .and_then(Value::as_u64)
        .ok_or(CartridgeError::InvalidSchema)?;
    if minimum > maximum {
        return Err(CartridgeError::InvalidSchema);
    }
    Ok(())
}

pub(crate) fn validate_localization(bytes: &[u8]) -> Result<Value> {
    let value: Value = parse_json(bytes, MAX_LOCALE_BYTES)?;
    let object = value
        .as_object()
        .ok_or(CartridgeError::InvalidLocalization)?;
    if object.len() > MAX_LOCALIZATION_ENTRIES {
        return Err(CartridgeError::LimitExceeded);
    }
    for (key, value) in object {
        if !valid_binding(key)
            || !value
                .as_str()
                .is_some_and(|text| valid_text(text, 0, MAX_LOCALIZED_VALUE_CHARS))
        {
            return Err(CartridgeError::InvalidLocalization);
        }
    }
    Ok(value)
}

pub(crate) fn validate_asset(bytes: &[u8], descriptor: &AssetDescriptor) -> Result<()> {
    match descriptor.media_type {
        AssetMediaType::ImagePng => validate_png(bytes, descriptor),
        AssetMediaType::AudioWav => validate_wav(bytes, descriptor),
    }
}

fn validate_png(bytes: &[u8], descriptor: &AssetDescriptor) -> Result<()> {
    const MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 33
        || &bytes[..8] != MAGIC
        || u32::from_be_bytes(bytes[8..12].try_into().unwrap()) != 13
        || &bytes[12..16] != b"IHDR"
    {
        return Err(CartridgeError::InvalidAsset);
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    let bit_depth = bytes[24];
    let color_type = bytes[25];
    let decoded = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(CartridgeError::LimitExceeded)?;
    if width == 0
        || height == 0
        || width > MAX_RASTER_DIMENSION
        || height > MAX_RASTER_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_RASTER_PIXELS
        || bit_depth != 8
        || !matches!(color_type, 0 | 2 | 3 | 4 | 6)
        || bytes[26] != 0
        || bytes[27] != 0
        || bytes[28] != 0
        || decoded > MAX_DECODED_ASSET_BYTES
        || descriptor.width != Some(width)
        || descriptor.height != Some(height)
        || descriptor.decoded_bytes != decoded
    {
        return Err(CartridgeError::InvalidAsset);
    }
    // V1 deliberately admits only a small, CRC-checked PNG subset. In
    // particular, compressed text/color-profile chunks cannot hide additional
    // decompression work outside the normalized RGBA decoded-byte budget.
    let mut offset = 8usize;
    let mut saw_ihdr = false;
    let mut saw_plte = false;
    let mut saw_idat = false;
    let mut ended_idat = false;
    let mut saw_iend = false;
    while offset < bytes.len() {
        if bytes.len() - offset < 12 {
            return Err(CartridgeError::InvalidAsset);
        }
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let end = offset
            .checked_add(12)
            .and_then(|value| value.checked_add(length))
            .ok_or(CartridgeError::LimitExceeded)?;
        if end > bytes.len() {
            return Err(CartridgeError::InvalidAsset);
        }
        let kind = &bytes[offset + 4..offset + 8];
        if !kind.iter().all(u8::is_ascii_alphabetic) {
            return Err(CartridgeError::InvalidAsset);
        }
        let data_end = offset + 8 + length;
        let expected_crc = u32::from_be_bytes(bytes[data_end..end].try_into().unwrap());
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(kind);
        hasher.update(&bytes[offset + 8..data_end]);
        if hasher.finalize() != expected_crc {
            return Err(CartridgeError::InvalidAsset);
        }
        match kind {
            b"IHDR" => {
                if saw_ihdr || offset != 8 || length != 13 {
                    return Err(CartridgeError::InvalidAsset);
                }
                saw_ihdr = true;
            }
            b"PLTE" => {
                if !saw_ihdr
                    || saw_plte
                    || saw_idat
                    || length == 0
                    || length > 768
                    || !length.is_multiple_of(3)
                    || matches!(color_type, 0 | 4)
                {
                    return Err(CartridgeError::InvalidAsset);
                }
                saw_plte = true;
            }
            b"IDAT" => {
                if !saw_ihdr || saw_iend || ended_idat || length == 0 {
                    return Err(CartridgeError::InvalidAsset);
                }
                saw_idat = true;
            }
            b"IEND" => {
                if !saw_ihdr
                    || !saw_idat
                    || saw_iend
                    || length != 0
                    || end != bytes.len()
                    || (color_type == 3 && !saw_plte)
                {
                    return Err(CartridgeError::InvalidAsset);
                }
                saw_iend = true;
            }
            _ => {
                return Err(CartridgeError::InvalidAsset);
            }
        }
        if saw_idat && kind != b"IDAT" {
            ended_idat = true;
        }
        offset = end;
    }
    if !saw_ihdr || !saw_idat || !saw_iend {
        return Err(CartridgeError::InvalidAsset);
    }
    Ok(())
}

fn validate_wav(bytes: &[u8], descriptor: &AssetDescriptor) -> Result<()> {
    if bytes.len() < 44 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(CartridgeError::InvalidAsset);
    }
    let riff_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    if riff_size.checked_add(8) != Some(bytes.len()) {
        return Err(CartridgeError::InvalidAsset);
    }
    let mut offset = 12usize;
    let mut format = None;
    let mut data_bytes = None;
    while offset < bytes.len() {
        if bytes.len() - offset < 8 {
            return Err(CartridgeError::InvalidAsset);
        }
        let kind = &bytes[offset..offset + 4];
        let length = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let start = offset + 8;
        let end = start
            .checked_add(length)
            .ok_or(CartridgeError::LimitExceeded)?;
        if end > bytes.len() {
            return Err(CartridgeError::InvalidAsset);
        }
        if kind == b"fmt " {
            if format.is_some() || length != 16 {
                return Err(CartridgeError::InvalidAsset);
            }
            format = Some(&bytes[start..end]);
        } else if kind == b"data" && data_bytes.replace(length as u64).is_some() {
            return Err(CartridgeError::InvalidAsset);
        }
        offset = end + (length & 1);
    }
    if offset != bytes.len() {
        return Err(CartridgeError::InvalidAsset);
    }
    let format = format.ok_or(CartridgeError::InvalidAsset)?;
    let audio_format = u16::from_le_bytes(format[0..2].try_into().unwrap());
    let channels = u16::from_le_bytes(format[2..4].try_into().unwrap());
    let sample_rate = u32::from_le_bytes(format[4..8].try_into().unwrap());
    let byte_rate = u32::from_le_bytes(format[8..12].try_into().unwrap());
    let block_align = u16::from_le_bytes(format[12..14].try_into().unwrap());
    let bits = u16::from_le_bytes(format[14..16].try_into().unwrap());
    let expected_align = channels
        .checked_mul(bits / 8)
        .ok_or(CartridgeError::InvalidAsset)?;
    let expected_rate = sample_rate
        .checked_mul(u32::from(expected_align))
        .ok_or(CartridgeError::InvalidAsset)?;
    let data_bytes = data_bytes.ok_or(CartridgeError::InvalidAsset)?;
    let duration_ms = data_bytes
        .checked_mul(1000)
        .and_then(|value| value.checked_div(u64::from(byte_rate)))
        .ok_or(CartridgeError::InvalidAsset)?;
    if audio_format != 1
        || !(1..=2).contains(&channels)
        || sample_rate == 0
        || sample_rate > 48_000
        || !matches!(bits, 8 | 16 | 24 | 32)
        || expected_align != block_align
        || expected_rate != byte_rate
        || data_bytes == 0
        || data_bytes % u64::from(block_align) != 0
        || data_bytes > MAX_DECODED_ASSET_BYTES
        || duration_ms > MAX_AUDIO_DURATION_MS
        || descriptor.decoded_bytes != data_bytes
        || descriptor.duration_ms != Some(duration_ms)
    {
        return Err(CartridgeError::InvalidAsset);
    }
    Ok(())
}

pub(crate) fn validate_inventory(
    manifest: &CartridgeManifest,
    paths: &BTreeSet<String>,
) -> Result<()> {
    let actual_schemas = paths
        .iter()
        .filter(|path| valid_schema_path(path))
        .cloned()
        .collect::<Vec<_>>();
    let actual_locales = paths
        .iter()
        .filter(|path| valid_locale_path(path))
        .cloned()
        .collect::<Vec<_>>();
    let actual_assets = paths
        .iter()
        .filter(|path| valid_asset_path(path))
        .cloned()
        .collect::<Vec<_>>();
    let declared_locales = manifest
        .locales
        .iter()
        .map(|item| item.path.clone())
        .collect::<Vec<_>>();
    let declared_assets = manifest
        .assets
        .iter()
        .map(|item| item.path.clone())
        .collect::<Vec<_>>();
    if actual_schemas != manifest.schemas
        || actual_locales != declared_locales
        || actual_assets != declared_assets
    {
        return Err(CartridgeError::InvalidManifest);
    }
    Ok(())
}

pub(crate) fn canonicalize_payload(path: &str, bytes: &[u8]) -> Result<Vec<u8>> {
    if path == "manifest.json" {
        let manifest: CartridgeManifest = parse_json(bytes, MAX_JSON_BYTES)?;
        validate_manifest(&manifest)?;
        canonical_json(&manifest)
    } else if path == "presentation.json" {
        let presentation: Presentation = parse_json(bytes, MAX_JSON_BYTES)?;
        canonical_json(&presentation)
    } else if valid_schema_path(path) {
        canonical_json(&validate_schema(bytes)?)
    } else if valid_locale_path(path) {
        canonical_json(&validate_localization(bytes)?)
    } else if valid_asset_path(path) {
        Ok(bytes.to_vec())
    } else {
        Err(CartridgeError::InvalidPath)
    }
}

pub(crate) fn valid_archive_path(path: &str) -> bool {
    path == "integrity.signed.json"
        || path == "manifest.json"
        || path == "presentation.json"
        || valid_schema_path(path)
        || valid_locale_path(path)
        || valid_asset_path(path)
}

pub(crate) fn valid_schema_path(path: &str) -> bool {
    valid_nested_path(path, "schemas/", ".schema.json")
}

pub(crate) fn valid_locale_path(path: &str) -> bool {
    valid_nested_path(path, "locales/", ".json")
}

pub(crate) fn valid_asset_path(path: &str) -> bool {
    valid_nested_path(path, "assets/", ".png") || valid_nested_path(path, "assets/", ".wav")
}

pub(crate) fn media_type_for_path(path: &str) -> Option<AssetMediaType> {
    if path.ends_with(".png") && valid_asset_path(path) {
        Some(AssetMediaType::ImagePng)
    } else if path.ends_with(".wav") && valid_asset_path(path) {
        Some(AssetMediaType::AudioWav)
    } else {
        None
    }
}

pub(crate) fn integrity_media_type(path: &str) -> Option<&'static str> {
    if path == "manifest.json" || path == "presentation.json" || path.ends_with(".json") {
        Some("application/json")
    } else if path.ends_with(".png") {
        Some("image/png")
    } else if path.ends_with(".wav") {
        Some("audio/wav")
    } else {
        None
    }
}

fn valid_nested_path(path: &str, prefix: &str, suffix: &str) -> bool {
    let Some(name) = path
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
    else {
        return false;
    };
    !name.is_empty()
        && !name.contains('/')
        && name.as_bytes()[0].is_ascii_lowercase()
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_version_range(min: u32, max: u32) -> bool {
    min > 0 && min <= max
}

fn valid_capability(value: &str) -> bool {
    valid_identifier(value) && value.contains('.')
}

fn valid_binding(value: &str) -> bool {
    valid_identifier(value)
}

fn valid_locale_tag(value: &str) -> bool {
    (2..=35).contains(&value.len())
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_text(value: &str, min: usize, max: usize) -> bool {
    let count = value.chars().count();
    (min..=max).contains(&count) && !value.chars().any(char::is_control)
}

fn validate_sorted_unique<F>(values: &[String], valid: F) -> std::result::Result<(), ()>
where
    F: Fn(&str) -> bool,
{
    if values.iter().any(|value| !valid(value)) || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(());
    }
    Ok(())
}

pub(crate) fn descriptors_by_path(
    manifest: &CartridgeManifest,
) -> BTreeMap<&str, &AssetDescriptor> {
    manifest
        .assets
        .iter()
        .map(|descriptor| (descriptor.path.as_str(), descriptor))
        .collect()
}

#[cfg(test)]
mod action_tests {
    use serde_json::json;

    use super::*;

    fn presentation() -> Presentation {
        Presentation {
            format_version: 1,
            screens: vec![
                crate::Screen {
                    id: "entry".to_owned(),
                    title: "Entry".to_owned(),
                    view_schema: "schemas/entry.json".to_owned(),
                    nodes: vec![
                        PresentationNode::Button {
                            id: "enter-button".to_owned(),
                            label_binding: "enter_label".to_owned(),
                            action: "enter".to_owned(),
                            accessible_label: "Enter".to_owned(),
                        },
                        PresentationNode::Grid {
                            id: "board".to_owned(),
                            rows: 2,
                            columns: 3,
                            cells_binding: "cells".to_owned(),
                            action: "move".to_owned(),
                            accessible_label: "Board".to_owned(),
                        },
                    ],
                },
                crate::Screen {
                    id: "later".to_owned(),
                    title: "Later".to_owned(),
                    view_schema: "schemas/later.json".to_owned(),
                    nodes: vec![PresentationNode::Button {
                        id: "later-button".to_owned(),
                        label_binding: "label".to_owned(),
                        action: "later".to_owned(),
                        accessible_label: "Later".to_owned(),
                    }],
                },
            ],
            actions: vec![
                ActionDefinition {
                    id: "enter".to_owned(),
                    payload_fields: vec![],
                },
                ActionDefinition {
                    id: "move".to_owned(),
                    payload_fields: vec!["column".to_owned(), "row".to_owned()],
                },
                ActionDefinition {
                    id: "later".to_owned(),
                    payload_fields: vec![],
                },
            ],
        }
    }

    #[test]
    fn entry_action_validation_is_exact_and_grid_bounded() {
        let presentation = presentation();
        assert!(validate_action_contract(&presentation, "entry", "enter", &json!({})).is_ok());
        assert!(
            validate_action_contract(
                &presentation,
                "entry",
                "move",
                &json!({"column": 2, "row": 1})
            )
            .is_ok()
        );
        for rejected in [
            ("enter", json!({"extra": true})),
            ("move", json!({"column": 3, "row": 1})),
            ("move", json!({"column": 1, "row": -1})),
            ("move", json!({"column": 1, "row": 0, "extra": 1})),
            ("later", json!({})),
            ("unknown", json!({})),
        ] {
            assert!(
                validate_action_contract(&presentation, "entry", rejected.0, &rejected.1).is_err()
            );
        }
    }
}
