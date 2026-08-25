use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Write},
    path::Path,
    process::Command,
};

use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use ed25519_dalek::{Signer, SigningKey};
use omarchygs_game_cartridge::{
    ActionDefinition, AssetDescriptor, AssetMediaType, CapabilityFallback, CartridgeError,
    CartridgeManifest, HostProfile, IntegrityEntry, IntegrityIndex, LocaleDescriptor,
    MAX_ARCHIVE_BYTES, OptionalCapability, Presentation, PresentationNode, PublisherPrivateKey,
    PublisherPublicKey, Screen, SignatureAlgorithm, SignedIntegrity, VersionCompatibility,
    VersionRange, baseline_host_profile, evaluate_compatibility, generate_keypair,
    install_cartridge, pack_directory, resolve_active_cartridge, revoke_cartridge, verify_archive,
    verify_archive_bytes,
};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use zip::{CompressionMethod, DateTime, System, ZipWriter, write::SimpleFileOptions};

struct Fixture {
    root: TempDir,
    private: PublisherPrivateKey,
    public: PublisherPublicKey,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let (private, public) = generate_keypair("ignibyte-primary-v1", "ignibyte").unwrap();
        write_fixture(root.path(), &valid_manifest(), &valid_presentation());
        Self {
            root,
            private,
            public,
        }
    }

    fn pack(&self) -> Vec<u8> {
        pack_directory(self.root.path(), &self.private).unwrap()
    }
}

#[test]
fn canonical_pack_is_deterministic_and_conformant() {
    let fixture = Fixture::new();
    let first = fixture.pack();
    let second = fixture.pack();
    assert_eq!(first, second);

    let verified = verify_archive_bytes(&first, &fixture.public, &baseline_host_profile()).unwrap();
    let report = verified.conformance_report();
    assert!(report.conformant);
    assert_eq!(report.game_key, "door-legends");
    assert_eq!(report.files.len(), 3);
    assert!(!report.installed);
    assert!(!report.provider_contacted);
    assert!(!report.database_required);
    assert!(!report.platform_credentials_read);
    assert_eq!(verified.manifest().game_key, "door-legends");
    assert!(
        verified
            .authenticated_file("schemas/game-state.schema.json")
            .is_some()
    );
    assert!(
        verified
            .authenticated_file("assets/not-declared.png")
            .is_none()
    );
}

#[test]
fn signed_optional_capability_selects_typed_fallback() {
    let fixture = Fixture::new();
    let verified =
        verify_archive_bytes(&fixture.pack(), &fixture.public, &baseline_host_profile()).unwrap();
    assert_eq!(
        verified.compatibility().selected_optional_fallbacks.len(),
        1
    );
    assert_eq!(
        verified.compatibility().selected_optional_fallbacks[0].fallback,
        CapabilityFallback::Muted
    );
}

#[test]
fn every_optional_fallback_kind_is_stable_and_missing_required_capabilities_fail() {
    let fallbacks = [
        CapabilityFallback::Omit,
        CapabilityFallback::Static,
        CapabilityFallback::ReducedMotion,
        CapabilityFallback::Muted,
        CapabilityFallback::PlatformPlaceholder,
        CapabilityFallback::SimplerCapability {
            capability: "presentation.grid.v1".to_owned(),
        },
    ];
    for fallback in fallbacks {
        let mut manifest = valid_manifest();
        manifest.optional_capabilities = vec![OptionalCapability {
            capability: "visual.optional.v1".to_owned(),
            fallback: fallback.clone(),
        }];
        let report = evaluate_compatibility(&manifest, &baseline_host_profile());
        assert!(report.compatible);
        assert_eq!(report.selected_optional_fallbacks[0].fallback, fallback);
    }

    let manifest = valid_manifest();
    let report = evaluate_compatibility(
        &manifest,
        &HostProfile {
            sdk_version: 1,
            presentation_protocol_version: 1,
            capabilities: BTreeSet::new(),
        },
    );
    assert!(!report.compatible);
    assert_eq!(
        report.missing_required_capabilities,
        manifest.required_capabilities
    );
}

#[test]
fn valid_but_incompatible_cartridge_has_stable_report() {
    let fixture = Fixture::new();
    let host = HostProfile {
        sdk_version: 99,
        presentation_protocol_version: 1,
        capabilities: baseline_host_profile().capabilities,
    };
    let verified = verify_archive_bytes(&fixture.pack(), &fixture.public, &host).unwrap();
    assert!(!verified.compatibility().compatible);
    assert_eq!(
        verified.compatibility().sdk,
        VersionCompatibility::HostTooNew
    );
}

#[test]
fn signature_tampering_is_rejected() {
    let fixture = Fixture::new();
    let mut archive = fixture.pack();
    let marker = b"\"signature\":\"";
    let start = find_bytes(&archive, marker).unwrap() + marker.len();
    archive[start] = if archive[start] == b'A' { b'B' } else { b'A' };
    assert!(verify_archive_bytes(&archive, &fixture.public, &baseline_host_profile()).is_err());
}

#[test]
fn a_valid_content_change_produces_new_signed_and_archive_identities() {
    let fixture = Fixture::new();
    let first =
        verify_archive_bytes(&fixture.pack(), &fixture.public, &baseline_host_profile()).unwrap();
    let mut presentation = valid_presentation();
    presentation.screens[0].title = "Door Legends: Second Edition".to_owned();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    let second =
        verify_archive_bytes(&fixture.pack(), &fixture.public, &baseline_host_profile()).unwrap();
    assert_ne!(
        first.signed_identity_sha256(),
        second.signed_identity_sha256()
    );
    assert_ne!(first.archive_sha256(), second.archive_sha256());
}

#[test]
fn oversized_archive_is_rejected_before_zip_parsing() {
    let (_, public) = generate_keypair("key-v1", "publisher").unwrap();
    let archive = vec![0u8; MAX_ARCHIVE_BYTES + 1];
    assert!(matches!(
        verify_archive_bytes(&archive, &public, &baseline_host_profile()),
        Err(CartridgeError::LimitExceeded)
    ));
}

#[test]
fn traversal_duplicate_symlink_and_executable_entries_are_rejected() {
    let (_, public) = generate_keypair("key-v1", "publisher").unwrap();
    let host = baseline_host_profile();

    let traversal = custom_zip(|writer| {
        writer
            .start_file("../manifest.json", canonical_options())
            .unwrap();
        writer.write_all(b"{}").unwrap();
    });
    assert!(matches!(
        verify_archive_bytes(&traversal, &public, &host),
        Err(CartridgeError::InvalidPath)
    ));

    let mut duplicate = custom_zip(|writer| {
        writer
            .start_file("assets/a.png", canonical_options())
            .unwrap();
        writer.write_all(b"a").unwrap();
        writer
            .start_file("assets/b.png", canonical_options())
            .unwrap();
        writer.write_all(b"b").unwrap();
    });
    replace_all_same_length(&mut duplicate, b"assets/b.png", b"assets/a.png");
    assert!(verify_archive_bytes(&duplicate, &public, &host).is_err());

    let symlink = custom_zip(|writer| {
        writer
            .add_symlink("manifest.json", "outside", canonical_options())
            .unwrap();
    });
    assert!(matches!(
        verify_archive_bytes(&symlink, &public, &host),
        Err(CartridgeError::InvalidArchiveEntry)
    ));

    let executable = custom_zip(|writer| {
        let options = canonical_options().unix_permissions(0o755);
        writer.start_file("manifest.json", options).unwrap();
        writer.write_all(b"{}").unwrap();
    });
    assert!(matches!(
        verify_archive_bytes(&executable, &public, &host),
        Err(CartridgeError::InvalidArchiveEntry)
    ));
}

#[test]
fn compressed_and_noncanonical_archives_are_rejected() {
    let fixture = Fixture::new();
    let mut compressed = fixture.pack();
    mutate_compression_method(&mut compressed, 8);
    assert!(verify_archive_bytes(&compressed, &fixture.public, &baseline_host_profile()).is_err());

    let noncanonical = custom_zip(|writer| {
        let options = SimpleFileOptions::DEFAULT
            .compression_method(CompressionMethod::Stored)
            .last_modified_time(DateTime::from_date_and_time(2024, 1, 1, 0, 0, 0).unwrap())
            .unix_permissions(0o444)
            .system(System::Unix);
        writer.start_file("manifest.json", options).unwrap();
        writer.write_all(b"{}").unwrap();
    });
    assert!(matches!(
        verify_archive_bytes(&noncanonical, &fixture.public, &baseline_host_profile()),
        Err(CartridgeError::InvalidArchiveEntry)
    ));
}

#[test]
fn pack_rejects_remote_schema_and_source_symlink() {
    let fixture = Fixture::new();
    fs::write(
        fixture.root.path().join("schemas/game-state.schema.json"),
        br#"{"$schema":"https://json-schema.org/draft/2020-12/schema","$ref":"https://attacker.invalid/schema"}"#,
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidSchema)
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        symlink("/etc/passwd", fixture.root.path().join("assets/escape.png")).unwrap();
        assert!(matches!(
            pack_directory(fixture.root.path(), &fixture.private),
            Err(CartridgeError::UnsafeFilesystemPath)
        ));
    }
}

#[test]
fn verifier_rejects_malicious_but_correctly_signed_schema_and_media() {
    let fixture = Fixture::new();
    let mut files = source_files(fixture.root.path());
    files.insert(
        "schemas/game-state.schema.json".to_owned(),
        br#"{"$ref":"https://attacker.invalid/schema","$schema":"https://json-schema.org/draft/2020-12/schema","type":"object"}"#.to_vec(),
    );
    let archive = sign_files(files, &fixture.private);
    assert!(matches!(
        verify_archive_bytes(&archive, &fixture.public, &baseline_host_profile()),
        Err(CartridgeError::InvalidSchema)
    ));

    let mut manifest = valid_manifest();
    manifest.assets = vec![AssetDescriptor {
        path: "assets/not-really.png".to_owned(),
        media_type: AssetMediaType::ImagePng,
        decoded_bytes: 4,
        width: Some(1),
        height: Some(1),
        duration_ms: None,
    }];
    let mut files = source_files(fixture.root.path());
    files.insert(
        "manifest.json".to_owned(),
        serde_json::to_vec(&manifest).unwrap(),
    );
    files.insert("assets/not-really.png".to_owned(), b"not a png".to_vec());
    let archive = sign_files(files, &fixture.private);
    assert!(matches!(
        verify_archive_bytes(&archive, &fixture.public, &baseline_host_profile()),
        Err(CartridgeError::InvalidAsset)
    ));
}

#[test]
fn presentation_nodes_require_their_declared_host_capabilities() {
    let fixture = Fixture::new();
    let mut manifest = valid_manifest();
    manifest
        .required_capabilities
        .retain(|capability| capability != "presentation.grid.v1");
    fs::write(
        fixture.root.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidPresentation)
    ));

    let mut files = source_files(fixture.root.path());
    files.insert(
        "manifest.json".to_owned(),
        serde_json::to_vec(&manifest).unwrap(),
    );
    let archive = sign_files(files, &fixture.private);
    assert!(matches!(
        verify_archive_bytes(&archive, &fixture.public, &baseline_host_profile()),
        Err(CartridgeError::InvalidPresentation)
    ));
}

#[test]
fn screens_actions_assets_and_fallbacks_are_cross_validated() {
    let fixture = Fixture::new();
    let mut presentation = valid_presentation();
    presentation.screens[0].view_schema = "schemas/not-declared.schema.json".to_owned();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidPresentation)
    ));

    let fixture = Fixture::new();
    let mut presentation = valid_presentation();
    presentation.actions[0].payload_fields.clear();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidPresentation)
    ));

    let fixture = Fixture::new();
    let mut presentation = valid_presentation();
    presentation.screens[0]
        .nodes
        .push(PresentationNode::Button {
            id: "continue".to_owned(),
            label_binding: "status.text".to_owned(),
            action: "continue".to_owned(),
            accessible_label: "Continue".to_owned(),
        });
    presentation.actions.push(ActionDefinition {
        id: "continue".to_owned(),
        payload_fields: vec!["unexpected".to_owned()],
    });
    let mut manifest = valid_manifest();
    manifest
        .required_capabilities
        .insert(0, "presentation.button.v1".to_owned());
    fs::write(
        fixture.root.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidPresentation)
    ));

    let fixture = Fixture::new();
    let mut presentation = valid_presentation();
    presentation.screens[0]
        .nodes
        .push(PresentationNode::Button {
            id: "continue".to_owned(),
            label_binding: "status.text".to_owned(),
            action: "not-declared".to_owned(),
            accessible_label: "Continue".to_owned(),
        });
    let mut manifest = valid_manifest();
    manifest
        .required_capabilities
        .insert(0, "presentation.button.v1".to_owned());
    fs::write(
        fixture.root.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidPresentation)
    ));

    let fixture = Fixture::new();
    let mut presentation = valid_presentation();
    presentation.screens[0].nodes.push(PresentationNode::Image {
        id: "portrait".to_owned(),
        asset: "assets/not-declared.png".to_owned(),
        accessible_label: "Portrait".to_owned(),
    });
    let mut manifest = valid_manifest();
    manifest
        .required_capabilities
        .insert(1, "presentation.image.v1".to_owned());
    fs::write(
        fixture.root.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        fixture.root.path().join("presentation.json"),
        serde_json::to_vec(&presentation).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidPresentation)
    ));
}

#[test]
fn png_and_pcm_wav_resources_are_bounded_and_authenticated() {
    let fixture = Fixture::new();
    let png = STANDARD
        .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
        .unwrap();
    let wav = one_sample_wav();
    fs::write(fixture.root.path().join("assets/pixel.png"), png).unwrap();
    fs::write(fixture.root.path().join("assets/tick.wav"), wav).unwrap();
    let mut manifest = valid_manifest();
    manifest.assets = vec![
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
    ];
    fs::write(
        fixture.root.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let archive = fixture.pack();
    assert!(verify_archive_bytes(&archive, &fixture.public, &baseline_host_profile()).is_ok());

    manifest.assets[0].width = Some(2);
    fs::write(
        fixture.root.path().join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        pack_directory(fixture.root.path(), &fixture.private),
        Err(CartridgeError::InvalidAsset)
    ));
}

#[test]
fn png_profile_rejects_16_bit_crc_tampering_and_ancillary_compression() {
    let fixture = Fixture::new();
    let base_png = STANDARD
        .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
        .unwrap();
    let manifest = manifest_with_png();

    let mut sixteen_bit = base_png.clone();
    sixteen_bit[24] = 16;
    rewrite_png_chunk_crc(&mut sixteen_bit, 8);
    assert_signed_png_rejected(&fixture, &manifest, sixteen_bit);

    let mut bad_crc = base_png.clone();
    bad_crc[41] ^= 1;
    assert_signed_png_rejected(&fixture, &manifest, bad_crc);

    let iend = base_png.len() - 12;
    let mut ancillary = base_png[..iend].to_vec();
    append_png_chunk(&mut ancillary, b"zTXt", b"note\0\0compressed");
    ancillary.extend_from_slice(&base_png[iend..]);
    assert_signed_png_rejected(&fixture, &manifest, ancillary);
}

#[test]
fn path_verification_rejects_non_regular_and_symlink_inputs() {
    let fixture = Fixture::new();
    assert!(matches!(
        verify_archive(
            fixture.root.path(),
            &fixture.public,
            &baseline_host_profile()
        ),
        Err(CartridgeError::UnsafeFilesystemPath)
    ));

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let archive_path = fixture.root.path().join("game.ogsc");
        let linked_path = fixture.root.path().join("linked.ogsc");
        fs::write(&archive_path, fixture.pack()).unwrap();
        symlink(&archive_path, &linked_path).unwrap();
        assert!(matches!(
            verify_archive(&linked_path, &fixture.public, &baseline_host_profile()),
            Err(CartridgeError::UnsafeFilesystemPath)
        ));
    }
}

#[test]
fn install_is_content_addressed_and_revocation_fails_closed() {
    let fixture = Fixture::new();
    let archive = fixture.pack();
    let workspace = tempfile::tempdir().unwrap();
    let archive_path = workspace.path().join("game.ogsc");
    fs::write(&archive_path, archive).unwrap();
    let store = workspace.path().join("store");
    let activation = install_cartridge(
        &archive_path,
        &fixture.public,
        &baseline_host_profile(),
        &store,
    )
    .unwrap();
    let resolved = resolve_active_cartridge(
        &store,
        "door-legends",
        &fixture.public,
        &baseline_host_profile(),
    )
    .unwrap();
    assert_eq!(resolved.archive_sha256(), activation.archive_sha256);
    assert!(
        store
            .join("blobs/sha256")
            .join(format!("{}.ogsc", activation.archive_sha256))
            .exists()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let blob_mode = fs::metadata(
            store
                .join("blobs/sha256")
                .join(format!("{}.ogsc", activation.archive_sha256)),
        )
        .unwrap()
        .permissions()
        .mode()
            & 0o777;
        let activation_mode = fs::metadata(store.join("active/door-legends.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(blob_mode, 0o444);
        assert_eq!(activation_mode, 0o444);
    }

    revoke_cartridge(&store, &activation.archive_sha256, "publisher withdrawal").unwrap();
    revoke_cartridge(&store, &activation.archive_sha256, "publisher withdrawal").unwrap();
    assert!(matches!(
        resolve_active_cartridge(
            &store,
            "door-legends",
            &fixture.public,
            &baseline_host_profile()
        ),
        Err(CartridgeError::Revoked)
    ));
    assert!(matches!(
        install_cartridge(
            &archive_path,
            &fixture.public,
            &baseline_host_profile(),
            &store
        ),
        Err(CartridgeError::Revoked)
    ));
}

#[cfg(unix)]
#[test]
fn malformed_revocation_path_fails_closed() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let workspace = tempfile::tempdir().unwrap();
    let archive_path = workspace.path().join("game.ogsc");
    fs::write(&archive_path, fixture.pack()).unwrap();
    let store = workspace.path().join("store");
    let activation = install_cartridge(
        &archive_path,
        &fixture.public,
        &baseline_host_profile(),
        &store,
    )
    .unwrap();
    symlink(
        "missing-revocation-record",
        store
            .join("revoked")
            .join(format!("{}.json", activation.archive_sha256)),
    )
    .unwrap();

    assert!(
        resolve_active_cartridge(
            &store,
            "door-legends",
            &fixture.public,
            &baseline_host_profile(),
        )
        .is_err()
    );
    assert!(
        install_cartridge(
            &archive_path,
            &fixture.public,
            &baseline_host_profile(),
            &store,
        )
        .is_err()
    );
}

#[test]
fn incompatible_install_creates_no_store_state() {
    let fixture = Fixture::new();
    let workspace = tempfile::tempdir().unwrap();
    let archive_path = workspace.path().join("game.ogsc");
    fs::write(&archive_path, fixture.pack()).unwrap();
    let store = workspace.path().join("store");
    let host = HostProfile {
        sdk_version: 99,
        presentation_protocol_version: 1,
        capabilities: baseline_host_profile().capabilities,
    };
    assert!(matches!(
        install_cartridge(&archive_path, &fixture.public, &host, &store),
        Err(CartridgeError::Incompatible)
    ));
    assert!(!store.exists());
}

#[test]
fn conform_cli_emits_machine_readable_isolation_evidence() {
    let fixture = Fixture::new();
    let workspace = tempfile::tempdir().unwrap();
    let archive = workspace.path().join("game.ogsc");
    let public_key = workspace.path().join("publisher.public.json");
    fs::write(&archive, fixture.pack()).unwrap();
    fs::write(&public_key, serde_json::to_vec(&fixture.public).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_omarchygs-cartridge"))
        .args(["conform"])
        .arg(&archive)
        .arg(&public_key)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["conformant"], true);
    assert_eq!(report["installed"], false);
    assert_eq!(report["provider_contacted"], false);
    assert_eq!(report["database_required"], false);
    assert_eq!(report["platform_credentials_read"], false);
}

fn write_fixture(root: &Path, manifest: &CartridgeManifest, presentation: &Presentation) {
    fs::create_dir(root.join("schemas")).unwrap();
    fs::create_dir(root.join("locales")).unwrap();
    fs::create_dir(root.join("assets")).unwrap();
    fs::write(
        root.join("manifest.json"),
        serde_json::to_vec(manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("presentation.json"),
        serde_json::to_vec(presentation).unwrap(),
    )
    .unwrap();
    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "status": {"type": "string", "maxLength": 128}
        },
        "required": ["status"],
        "additionalProperties": false
    });
    fs::write(
        root.join("schemas/game-state.schema.json"),
        serde_json::to_vec(&schema).unwrap(),
    )
    .unwrap();
}

fn valid_manifest() -> CartridgeManifest {
    CartridgeManifest {
        format_version: 1,
        game_key: "door-legends".to_owned(),
        publisher_id: "ignibyte".to_owned(),
        rules_version: 1,
        cartridge_version: 1,
        sdk: VersionRange { min: 1, max: 1 },
        presentation_protocol: VersionRange { min: 1, max: 1 },
        display_name: "Door Legends".to_owned(),
        entry_screen: "main".to_owned(),
        required_capabilities: vec![
            "presentation.grid.v1".to_owned(),
            "presentation.status.v1".to_owned(),
            "presentation.terminal.v1".to_owned(),
        ],
        optional_capabilities: vec![OptionalCapability {
            capability: "audio.effects.v1".to_owned(),
            fallback: CapabilityFallback::Muted,
        }],
        schemas: vec!["schemas/game-state.schema.json".to_owned()],
        locales: Vec::<LocaleDescriptor>::new(),
        assets: Vec::new(),
    }
}

fn manifest_with_png() -> CartridgeManifest {
    let mut manifest = valid_manifest();
    manifest.assets = vec![AssetDescriptor {
        path: "assets/pixel.png".to_owned(),
        media_type: AssetMediaType::ImagePng,
        decoded_bytes: 4,
        width: Some(1),
        height: Some(1),
        duration_ms: None,
    }];
    manifest
}

fn assert_signed_png_rejected(fixture: &Fixture, manifest: &CartridgeManifest, png: Vec<u8>) {
    let mut files = source_files(fixture.root.path());
    files.insert(
        "manifest.json".to_owned(),
        serde_json::to_vec(manifest).unwrap(),
    );
    files.insert("assets/pixel.png".to_owned(), png);
    let archive = sign_files(files, &fixture.private);
    assert!(matches!(
        verify_archive_bytes(&archive, &fixture.public, &baseline_host_profile()),
        Err(CartridgeError::InvalidAsset)
    ));
}

fn valid_presentation() -> Presentation {
    Presentation {
        format_version: 1,
        screens: vec![Screen {
            id: "main".to_owned(),
            title: "Door Legends".to_owned(),
            view_schema: "schemas/game-state.schema.json".to_owned(),
            nodes: vec![
                PresentationNode::Grid {
                    id: "board".to_owned(),
                    rows: 8,
                    columns: 8,
                    cells_binding: "board.cells".to_owned(),
                    action: "move".to_owned(),
                    accessible_label: "Game board".to_owned(),
                },
                PresentationNode::Status {
                    id: "game-status".to_owned(),
                    text_binding: "status.text".to_owned(),
                    accessible_label: "Game status".to_owned(),
                },
                PresentationNode::Terminal {
                    id: "game-log".to_owned(),
                    text_binding: "log.text".to_owned(),
                    accessible_label: "Game log".to_owned(),
                },
            ],
        }],
        actions: vec![ActionDefinition {
            id: "move".to_owned(),
            payload_fields: vec!["column".to_owned(), "row".to_owned()],
        }],
    }
}

fn canonical_options() -> SimpleFileOptions {
    SimpleFileOptions::DEFAULT
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(DateTime::DEFAULT)
        .unix_permissions(0o444)
        .system(System::Unix)
}

fn custom_zip(action: impl FnOnce(&mut ZipWriter<Cursor<Vec<u8>>>)) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    action(&mut writer);
    writer.finish().unwrap().into_inner()
}

fn source_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([
        (
            "manifest.json".to_owned(),
            fs::read(root.join("manifest.json")).unwrap(),
        ),
        (
            "presentation.json".to_owned(),
            fs::read(root.join("presentation.json")).unwrap(),
        ),
        (
            "schemas/game-state.schema.json".to_owned(),
            fs::read(root.join("schemas/game-state.schema.json")).unwrap(),
        ),
    ])
}

fn sign_files(mut files: BTreeMap<String, Vec<u8>>, private: &PublisherPrivateKey) -> Vec<u8> {
    let entries = files
        .iter()
        .map(|(path, bytes)| IntegrityEntry {
            path: path.clone(),
            media_type: if path.ends_with(".png") {
                "image/png".to_owned()
            } else if path.ends_with(".wav") {
                "audio/wav".to_owned()
            } else {
                "application/json".to_owned()
            },
            bytes: bytes.len() as u64,
            sha256: sha256_hex(bytes),
        })
        .collect();
    let payload = serde_json::to_vec(&IntegrityIndex {
        format_version: 1,
        publisher_id: private.publisher_id.clone(),
        files: entries,
    })
    .unwrap();
    let mut message = b"omarchygs-cartridge-integrity-v1\0".to_vec();
    message.extend_from_slice(&payload);
    let signing: SigningKey = private.decode().unwrap();
    let envelope = SignedIntegrity {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: private.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload),
        signature: URL_SAFE_NO_PAD.encode(signing.sign(&message).to_bytes()),
    };
    files.insert(
        "integrity.signed.json".to_owned(),
        serde_json::to_vec(&envelope).unwrap(),
    );
    custom_zip(|writer| {
        for (path, bytes) in &files {
            writer.start_file(path, canonical_options()).unwrap();
            writer.write_all(bytes).unwrap();
        }
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn mutate_compression_method(archive: &mut [u8], method: u16) {
    let mut offset = 0usize;
    while offset + 10 < archive.len() {
        if archive[offset..].starts_with(b"PK\x03\x04") {
            archive[offset + 8..offset + 10].copy_from_slice(&method.to_le_bytes());
            offset += 4;
        } else if archive[offset..].starts_with(b"PK\x01\x02") {
            archive[offset + 10..offset + 12].copy_from_slice(&method.to_le_bytes());
            offset += 4;
        } else {
            offset += 1;
        }
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn replace_all_same_length(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) {
    assert_eq!(needle.len(), replacement.len());
    let mut offset = 0usize;
    while let Some(relative) = find_bytes(&haystack[offset..], needle) {
        let start = offset + relative;
        haystack[start..start + needle.len()].copy_from_slice(replacement);
        offset = start + needle.len();
    }
}

fn rewrite_png_chunk_crc(png: &mut [u8], offset: usize) {
    let length = u32::from_be_bytes(png[offset..offset + 4].try_into().unwrap()) as usize;
    let data_end = offset + 8 + length;
    let crc = crc32fast::hash(&png[offset + 4..data_end]);
    png[data_end..data_end + 4].copy_from_slice(&crc.to_be_bytes());
}

fn append_png_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(kind);
    hasher.update(data);
    png.extend_from_slice(&hasher.finalize().to_be_bytes());
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
