use std::{collections::BTreeSet, fs, process::Command};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use omarchygs_game_cartridge::{
    ActionDefinition, AssetDescriptor, AssetMediaType, CapabilityFallback, CartridgeManifest,
    LocaleDescriptor, OptionalCapability, ParticlePreset, Presentation, PresentationNode,
    PublisherPrivateKey, PublisherPublicKey, Screen, VersionRange, core_host_profile,
    generate_keypair, pack_directory, rich_2d_host_profile, verify_archive_bytes,
};
use omarchygs_game_cartridge_renderer::{
    PLAN_FORMAT, RenderProfile, RenderedNode, RendererError, RendererPreferences, SurfaceState,
    compile_render_plan, valid_asset_token, write_prepared_preview,
};
use tempfile::TempDir;

struct Fixture {
    root: TempDir,
    private: PublisherPrivateKey,
    public: PublisherPublicKey,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let (private, public) = generate_keypair("ignibyte-renderer-v1", "ignibyte").unwrap();
        write_fixture(root.path());
        Self {
            root,
            private,
            public,
        }
    }

    fn verified(&self, profile: RenderProfile) -> omarchygs_game_cartridge::VerifiedCartridge {
        let bytes = pack_directory(self.root.path(), &self.private).unwrap();
        let host = match profile {
            RenderProfile::Core => core_host_profile(),
            RenderProfile::Rich2d => rich_2d_host_profile(),
        };
        verify_archive_bytes(&bytes, &self.public, &host).unwrap()
    }
}

#[test]
fn rich_plan_contains_only_typed_tags_and_digest_assets() {
    let fixture = Fixture::new();
    let verified = fixture.verified(RenderProfile::Rich2d);
    let prepared = compile_render_plan(
        &verified,
        None,
        &valid_view(),
        RenderProfile::Rich2d,
        RendererPreferences::default(),
        SurfaceState::Ready,
    )
    .unwrap();

    assert_eq!(prepared.plan.format, PLAN_FORMAT);
    assert_eq!(prepared.plan.nodes.len(), 9);
    assert!(prepared.plan.requested_actions_are_unconfirmed);
    let kinds = prepared
        .plan
        .nodes
        .iter()
        .map(kind)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        kinds,
        BTreeSet::from([
            "audio_cue",
            "button",
            "grid",
            "image",
            "meter",
            "particle_field",
            "sprite",
            "status",
            "terminal",
        ])
    );
    assert_eq!(prepared.assets.len(), 2);
    assert!(prepared.assets.keys().all(|token| valid_asset_token(token)));
    let serialized = serde_json::to_string(&prepared.plan).unwrap();
    assert!(!serialized.contains("assets/pixel.png"));
    assert!(!serialized.contains(".qml"));
    assert!(!serialized.contains("javascript"));
}

#[test]
fn core_profile_applies_signed_rich_fallbacks() {
    let fixture = Fixture::new();
    let verified = fixture.verified(RenderProfile::Core);
    let prepared = compile_render_plan(
        &verified,
        None,
        &valid_view(),
        RenderProfile::Core,
        RendererPreferences::default(),
        SurfaceState::Ready,
    )
    .unwrap();

    assert!(prepared.plan.nodes.iter().any(|node| matches!(
        node,
        RenderedNode::Image { id, .. } if id == "hero"
    )));
    assert!(prepared.plan.nodes.iter().any(|node| matches!(
        node,
        RenderedNode::ParticleField {
            particle_count: 0,
            running: false,
            ..
        }
    )));
    assert!(
        !prepared
            .plan
            .nodes
            .iter()
            .any(|node| matches!(node, RenderedNode::Sprite { .. }))
    );
    assert!(
        !prepared
            .plan
            .nodes
            .iter()
            .any(|node| matches!(node, RenderedNode::AudioCue { .. }))
    );
}

#[test]
fn trusted_preferences_disable_motion_and_audio() {
    let fixture = Fixture::new();
    let verified = fixture.verified(RenderProfile::Rich2d);
    let prepared = compile_render_plan(
        &verified,
        None,
        &valid_view(),
        RenderProfile::Rich2d,
        RendererPreferences {
            scale: 1.5,
            high_contrast: true,
            reduced_motion: true,
            muted_audio: true,
        },
        SurfaceState::Ready,
    )
    .unwrap();
    assert!(prepared.plan.nodes.iter().any(|node| matches!(
        node,
        RenderedNode::Sprite {
            animated: false,
            ..
        }
    )));
    assert!(
        prepared
            .plan
            .nodes
            .iter()
            .any(|node| matches!(node, RenderedNode::ParticleField { running: false, .. }))
    );
    assert!(
        prepared
            .plan
            .nodes
            .iter()
            .any(|node| matches!(node, RenderedNode::AudioCue { muted: true, .. }))
    );
}

#[test]
fn every_non_ready_state_uses_fixed_chrome_and_zero_cartridge_nodes() {
    let fixture = Fixture::new();
    let verified = fixture.verified(RenderProfile::Rich2d);
    for state in [
        SurfaceState::Loading,
        SurfaceState::Offline,
        SurfaceState::Stale,
        SurfaceState::Empty,
        SurfaceState::ProtocolError,
        SurfaceState::UnsupportedCapability,
        SurfaceState::Revoked,
    ] {
        let prepared = compile_render_plan(
            &verified,
            None,
            b"not-json-but-never-rendered",
            RenderProfile::Rich2d,
            RendererPreferences::default(),
            state,
        )
        .unwrap();
        assert!(prepared.plan.nodes.is_empty());
        assert!(!prepared.plan.state_message.is_empty());
        assert_eq!(prepared.plan.origin.publisher_id, "ignibyte");
        assert!(prepared.assets.is_empty());
    }
}

#[test]
fn schema_bindings_ranges_and_profile_limits_fail_closed() {
    let fixture = Fixture::new();
    let verified = fixture.verified(RenderProfile::Rich2d);

    let missing = serde_json::json!({"status": {"text": "missing everything else"}});
    assert!(matches!(
        compile_render_plan(
            &verified,
            None,
            &serde_json::to_vec(&missing).unwrap(),
            RenderProfile::Rich2d,
            RendererPreferences::default(),
            SurfaceState::Ready,
        ),
        Err(RendererError::InvalidView)
    ));

    let mut out_of_range: serde_json::Value = serde_json::from_slice(&valid_view()).unwrap();
    out_of_range["meter"]["value"] = serde_json::json!(101);
    assert!(matches!(
        compile_render_plan(
            &verified,
            None,
            &serde_json::to_vec(&out_of_range).unwrap(),
            RenderProfile::Rich2d,
            RendererPreferences::default(),
            SurfaceState::Ready,
        ),
        Err(RendererError::InvalidView)
    ));

    assert!(matches!(
        compile_render_plan(
            &verified,
            None,
            &vec![b' '; RenderProfile::Core.limits().max_view_bytes + 1],
            RenderProfile::Core,
            RendererPreferences::default(),
            SurfaceState::Ready,
        ),
        Err(RendererError::BudgetExceeded)
    ));
}

#[test]
fn repeated_large_bindings_are_rejected_by_the_plan_byte_budget() {
    let fixture = Fixture::new();
    let mut presentation = presentation();
    presentation.screens[0].nodes = (0..64)
        .map(|index| PresentationNode::Terminal {
            id: format!("log-{index}"),
            text_binding: "log.text".to_owned(),
            accessible_label: format!("Game log {index}"),
        })
        .collect();
    presentation.actions.clear();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    let mut view_schema = schema();
    view_schema["properties"]["log"]["properties"]["text"]["maxLength"] = serde_json::json!(65_536);
    fs::write(
        fixture.root.path().join("schemas/view.schema.json"),
        serde_json::to_vec(&view_schema).unwrap(),
    )
    .unwrap();
    let verified = fixture.verified(RenderProfile::Rich2d);
    let mut view: serde_json::Value = serde_json::from_slice(&valid_view()).unwrap();
    view["log"]["text"] = serde_json::Value::String("x".repeat(65_536));

    assert!(matches!(
        compile_render_plan(
            &verified,
            None,
            &serde_json::to_vec(&view).unwrap(),
            RenderProfile::Rich2d,
            RendererPreferences::default(),
            SurfaceState::Ready,
        ),
        Err(RendererError::BudgetExceeded)
    ));
}

#[test]
fn repeated_optional_asset_references_publish_one_authenticated_buffer() {
    let fixture = Fixture::new();
    let mut presentation = presentation();
    presentation.screens[0].nodes = (0..200)
        .map(|index| PresentationNode::Sprite {
            id: format!("hero-{index}"),
            asset: "assets/pixel.png".to_owned(),
            frame_width: 1,
            frame_height: 1,
            frame_count: 1,
            frames_per_second: 12,
            accessible_label: format!("Hero {index}"),
        })
        .collect();
    presentation.actions.clear();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    let verified = fixture.verified(RenderProfile::Rich2d);

    let prepared = compile_render_plan(
        &verified,
        None,
        &valid_view(),
        RenderProfile::Rich2d,
        RendererPreferences::default(),
        SurfaceState::Ready,
    )
    .unwrap();

    assert_eq!(prepared.plan.nodes.len(), 128);
    assert_eq!(prepared.assets.len(), 1);
}

#[test]
fn prepared_output_requires_private_empty_directory_and_is_read_only() {
    let fixture = Fixture::new();
    let verified = fixture.verified(RenderProfile::Rich2d);
    let prepared = compile_render_plan(
        &verified,
        None,
        &valid_view(),
        RenderProfile::Rich2d,
        RendererPreferences::default(),
        SurfaceState::Ready,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }
    let receipt = write_prepared_preview(&prepared, output.path()).unwrap();
    assert!(!receipt.provider_contacted);
    assert!(!receipt.database_required);
    assert!(!receipt.platform_credentials_read);
    assert_eq!(receipt.asset_count, 2);
    assert!(output.path().join("render-plan.json").is_file());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(output.path().join("render-plan.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o444
        );
        assert_eq!(
            fs::metadata(output.path().join("assets"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    assert!(matches!(
        write_prepared_preview(&prepared, output.path()),
        Err(RendererError::UnsafeOutputDirectory)
    ));
}

#[test]
fn preview_cli_uses_production_verification_and_reports_isolation() {
    let fixture = Fixture::new();
    let inputs = tempfile::tempdir().unwrap();
    let archive = inputs.path().join("demo.ogsc");
    let public_key = inputs.path().join("publisher.public.json");
    let view = inputs.path().join("view.json");
    let preferences = inputs.path().join("preferences.json");
    fs::write(
        &archive,
        pack_directory(fixture.root.path(), &fixture.private).unwrap(),
    )
    .unwrap();
    fs::write(&public_key, serde_json::to_vec(&fixture.public).unwrap()).unwrap();
    fs::write(&view, valid_view()).unwrap();
    fs::write(
        &preferences,
        serde_json::to_vec(&RendererPreferences::default()).unwrap(),
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output.path(), fs::Permissions::from_mode(0o700)).unwrap();
    }

    let result = Command::new(env!("CARGO_BIN_EXE_omarchygs-cartridge-preview"))
        .args([
            "prepare",
            archive.to_str().unwrap(),
            public_key.to_str().unwrap(),
            "rich2d",
            view.to_str().unwrap(),
            "ready",
            preferences.to_str().unwrap(),
            output.path().to_str().unwrap(),
        ])
        .env("DATABASE_URL", "postgres://unusable.invalid/no-access")
        .env("OMARCHYGS_DEVICE_TOKEN", "must-not-be-read")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["ok"], true);
    assert_eq!(receipt["provider_contacted"], false);
    assert_eq!(receipt["database_required"], false);
    assert_eq!(receipt["platform_credentials_read"], false);
    assert!(output.path().join("render-plan.json").is_file());
}

fn kind(node: &RenderedNode) -> &'static str {
    match node {
        RenderedNode::Terminal { .. } => "terminal",
        RenderedNode::Grid { .. } => "grid",
        RenderedNode::Status { .. } => "status",
        RenderedNode::Button { .. } => "button",
        RenderedNode::Image { .. } => "image",
        RenderedNode::Meter { .. } => "meter",
        RenderedNode::Sprite { .. } => "sprite",
        RenderedNode::ParticleField { .. } => "particle_field",
        RenderedNode::AudioCue { .. } => "audio_cue",
        RenderedNode::PlatformPlaceholder { .. } => "platform_placeholder",
    }
}

fn write_fixture(root: &std::path::Path) {
    fs::create_dir(root.join("schemas")).unwrap();
    fs::create_dir(root.join("assets")).unwrap();
    fs::write(
        root.join("assets/pixel.png"),
        STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .unwrap(),
    )
    .unwrap();
    fs::write(root.join("assets/tick.wav"), one_sample_wav()).unwrap();
    fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(&manifest()).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("presentation.json"),
        serde_json::to_vec(&presentation()).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("schemas/view.schema.json"),
        serde_json::to_vec(&schema()).unwrap(),
    )
    .unwrap();
}

fn manifest() -> CartridgeManifest {
    CartridgeManifest {
        format_version: 1,
        game_key: "renderer-demo".to_owned(),
        publisher_id: "ignibyte".to_owned(),
        rules_version: 1,
        cartridge_version: 1,
        sdk: VersionRange { min: 1, max: 1 },
        presentation_protocol: VersionRange { min: 1, max: 1 },
        display_name: "Renderer Demo".to_owned(),
        entry_screen: "main".to_owned(),
        required_capabilities: vec![
            "presentation.button.v1".to_owned(),
            "presentation.grid.v1".to_owned(),
            "presentation.image.v1".to_owned(),
            "presentation.meter.v1".to_owned(),
            "presentation.status.v1".to_owned(),
            "presentation.terminal.v1".to_owned(),
        ],
        optional_capabilities: vec![
            OptionalCapability {
                capability: "audio.effects.v1".to_owned(),
                fallback: CapabilityFallback::Muted,
            },
            OptionalCapability {
                capability: "presentation.particles.v1".to_owned(),
                fallback: CapabilityFallback::ReducedMotion,
            },
            OptionalCapability {
                capability: "presentation.sprite.v1".to_owned(),
                fallback: CapabilityFallback::SimplerCapability {
                    capability: "presentation.image.v1".to_owned(),
                },
            },
        ],
        schemas: vec!["schemas/view.schema.json".to_owned()],
        locales: Vec::<LocaleDescriptor>::new(),
        assets: vec![
            AssetDescriptor {
                path: "assets/pixel.png".to_owned(),
                media_type: AssetMediaType::ImagePng,
                decoded_bytes: 4,
                width: Some(1),
                height: Some(1),
                duration_ms: None,
            },
            AssetDescriptor {
                path: "assets/tick.wav".to_owned(),
                media_type: AssetMediaType::AudioWav,
                decoded_bytes: 1,
                width: None,
                height: None,
                duration_ms: Some(0),
            },
        ],
    }
}

fn presentation() -> Presentation {
    Presentation {
        format_version: 1,
        screens: vec![Screen {
            id: "main".to_owned(),
            title: "Renderer Demo".to_owned(),
            view_schema: "schemas/view.schema.json".to_owned(),
            nodes: vec![
                PresentationNode::Terminal {
                    id: "log".to_owned(),
                    text_binding: "log.text".to_owned(),
                    accessible_label: "Game log".to_owned(),
                },
                PresentationNode::Grid {
                    id: "board".to_owned(),
                    rows: 2,
                    columns: 2,
                    cells_binding: "board.cells".to_owned(),
                    action: "move".to_owned(),
                    accessible_label: "Game board".to_owned(),
                },
                PresentationNode::Status {
                    id: "status".to_owned(),
                    text_binding: "status.text".to_owned(),
                    accessible_label: "Game status".to_owned(),
                },
                PresentationNode::Button {
                    id: "end-turn".to_owned(),
                    label_binding: "button.label".to_owned(),
                    action: "end_turn".to_owned(),
                    accessible_label: "End turn".to_owned(),
                },
                PresentationNode::Image {
                    id: "portrait".to_owned(),
                    asset: "assets/pixel.png".to_owned(),
                    accessible_label: "Player portrait".to_owned(),
                },
                PresentationNode::Meter {
                    id: "health".to_owned(),
                    value_binding: "meter.value".to_owned(),
                    minimum: 0,
                    maximum: 100,
                    accessible_label: "Health".to_owned(),
                },
                PresentationNode::Sprite {
                    id: "hero".to_owned(),
                    asset: "assets/pixel.png".to_owned(),
                    frame_width: 1,
                    frame_height: 1,
                    frame_count: 1,
                    frames_per_second: 12,
                    accessible_label: "Hero".to_owned(),
                },
                PresentationNode::ParticleField {
                    id: "stars".to_owned(),
                    particle_count: 32,
                    preset: ParticlePreset::Stars,
                    accessible_label: "Star field".to_owned(),
                },
                PresentationNode::AudioCue {
                    id: "tick".to_owned(),
                    asset: "assets/tick.wav".to_owned(),
                    looped: false,
                    accessible_label: "Turn sound".to_owned(),
                },
            ],
        }],
        actions: vec![
            ActionDefinition {
                id: "end_turn".to_owned(),
                payload_fields: Vec::new(),
            },
            ActionDefinition {
                id: "move".to_owned(),
                payload_fields: vec!["column".to_owned(), "row".to_owned()],
            },
        ],
    }
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "board": {
                "type": "object",
                "properties": {
                    "cells": {
                        "type": "array",
                        "items": {"type": "string", "maxLength": 16},
                        "minItems": 4,
                        "maxItems": 4
                    }
                },
                "required": ["cells"],
                "additionalProperties": false
            },
            "button": {
                "type": "object",
                "properties": {"label": {"type": "string", "maxLength": 64}},
                "required": ["label"],
                "additionalProperties": false
            },
            "log": {
                "type": "object",
                "properties": {"text": {"type": "string", "maxLength": 4096}},
                "required": ["text"],
                "additionalProperties": false
            },
            "meter": {
                "type": "object",
                "properties": {"value": {"type": "number", "minimum": 0, "maximum": 100}},
                "required": ["value"],
                "additionalProperties": false
            },
            "status": {
                "type": "object",
                "properties": {"text": {"type": "string", "maxLength": 128}},
                "required": ["text"],
                "additionalProperties": false
            }
        },
        "required": ["board", "button", "log", "meter", "status"],
        "additionalProperties": false
    })
}

fn valid_view() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "board": {"cells": ["A", "B", "C", "D"]},
        "button": {"label": "End turn"},
        "log": {"text": "Welcome, operator."},
        "meter": {"value": 75},
        "status": {"text": "Your move"}
    }))
    .unwrap()
}

fn one_sample_wav() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&38u32.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.push(128);
    bytes.push(0);
    bytes
}
