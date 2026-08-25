use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Signer};
use sha2::{Digest, Sha256};
use zip::{CompressionMethod, DateTime, System, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    compatibility::evaluate_compatibility,
    contract::{
        CartridgeManifest, FORMAT_VERSION, FileProvenance, HostProfile, INTEGRITY_PATH,
        IntegrityEntry, IntegrityIndex, MANIFEST_PATH, MAX_ARCHIVE_BYTES, MAX_ENTRIES,
        MAX_ENTRY_BYTES, MAX_EXPANDED_BYTES, MAX_JSON_BYTES, PRESENTATION_PATH, Presentation,
        SignatureAlgorithm, SignedIntegrity, VerifiedCartridge,
    },
    error::{CartridgeError, Result},
    io::read_bounded_regular_file,
    keys::{PublisherPrivateKey, PublisherPublicKey},
    validate::{
        canonical_json, canonicalize_payload, descriptors_by_path, integrity_media_type,
        valid_archive_path, valid_locale_path, valid_schema_path, validate_asset,
        validate_inventory, validate_localization, validate_manifest, validate_presentation,
        validate_schema,
    },
};

const SIGNATURE_DOMAIN: &[u8] = b"omarchygs-cartridge-integrity-v1\0";

pub fn pack_directory(source: &Path, key: &PublisherPrivateKey) -> Result<Vec<u8>> {
    key.validate()?;
    let signing_key = key.decode()?;
    let mut files = collect_source_files(source)?;
    if files.contains_key(INTEGRITY_PATH) {
        return Err(CartridgeError::InvalidArchiveEntry);
    }
    validate_payload_files(&files, Some((&key.publisher_id, &key.key_id)))?;

    let index = build_integrity_index(&files, &key.publisher_id)?;
    let payload = canonical_json(&index)?;
    let mut message = Vec::with_capacity(SIGNATURE_DOMAIN.len() + payload.len());
    message.extend_from_slice(SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload);
    let signature = signing_key.sign(&message);
    let envelope = SignedIntegrity {
        algorithm: SignatureAlgorithm::Ed25519,
        key_id: key.key_id.clone(),
        payload: URL_SAFE_NO_PAD.encode(payload),
        signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
    };
    files.insert(INTEGRITY_PATH.to_owned(), canonical_json(&envelope)?);
    let archive = write_canonical_zip(&files)?;
    if archive.len() > MAX_ARCHIVE_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    Ok(archive)
}

pub fn verify_archive(
    path: &Path,
    key: &PublisherPublicKey,
    host: &HostProfile,
) -> Result<VerifiedCartridge> {
    let bytes = read_bounded_regular_file(path, MAX_ARCHIVE_BYTES as u64)?;
    verify_archive_bytes(&bytes, key, host)
}

pub fn verify_archive_bytes(
    archive_bytes: &[u8],
    key: &PublisherPublicKey,
    host: &HostProfile,
) -> Result<VerifiedCartridge> {
    key.validate()?;
    if archive_bytes.is_empty() || archive_bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes))?;
    if archive.is_empty()
        || archive.len() > MAX_ENTRIES
        || !archive.comment().is_empty()
        || archive.offset() != 0
        || archive.raw_zip64_extensible_data_sector().is_some()
    {
        return Err(CartridgeError::InvalidArchiveEntry);
    }

    let mut files = BTreeMap::<String, Vec<u8>>::new();
    let mut prior_path: Option<String> = None;
    let mut expanded_bytes = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index_raw(index)?;
        let path = std::str::from_utf8(entry.name_raw())
            .map_err(|_| CartridgeError::InvalidPath)?
            .to_owned();
        if !path.is_ascii() || !valid_archive_path(&path) {
            return Err(CartridgeError::InvalidPath);
        }
        if prior_path.as_ref().is_some_and(|prior| prior >= &path) {
            return Err(CartridgeError::DuplicateOrUnsortedPath);
        }
        prior_path = Some(path.clone());
        if entry.encrypted()
            || entry.compression() != CompressionMethod::Stored
            || !entry.is_file()
            || entry.is_dir()
            || entry.is_symlink()
            || !entry.comment().is_empty()
            || entry.extra_data().is_some_and(|data| !data.is_empty())
            || entry.size() != entry.compressed_size()
            || entry.size() > MAX_ENTRY_BYTES
            || entry.last_modified() != Some(DateTime::DEFAULT)
            || entry.unix_mode().is_some_and(|mode| mode & 0o777 != 0o444)
        {
            return Err(CartridgeError::InvalidArchiveEntry);
        }
        expanded_bytes = expanded_bytes
            .checked_add(entry.size())
            .ok_or(CartridgeError::LimitExceeded)?;
        if expanded_bytes > MAX_EXPANDED_BYTES {
            return Err(CartridgeError::LimitExceeded);
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .by_ref()
            .take(MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != entry.size() {
            return Err(CartridgeError::InvalidArchiveEntry);
        }
        files.insert(path, bytes);
    }

    let envelope_bytes = files
        .get(INTEGRITY_PATH)
        .ok_or(CartridgeError::InvalidIntegrity)?;
    if envelope_bytes.len() > MAX_JSON_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    let envelope: SignedIntegrity = serde_json::from_slice(envelope_bytes)?;
    if canonical_json(&envelope)? != *envelope_bytes || envelope.key_id != key.key_id {
        return Err(CartridgeError::InvalidIntegrity);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(&envelope.payload)
        .map_err(|_| CartridgeError::InvalidIntegrity)?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&envelope.signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;
    let signature =
        Signature::from_slice(&signature_bytes).map_err(|_| CartridgeError::InvalidSignature)?;
    let mut message = Vec::with_capacity(SIGNATURE_DOMAIN.len() + payload.len());
    message.extend_from_slice(SIGNATURE_DOMAIN);
    message.extend_from_slice(&payload);
    key.decode()?
        .verify_strict(&message, &signature)
        .map_err(|_| CartridgeError::InvalidSignature)?;

    let integrity: IntegrityIndex = serde_json::from_slice(&payload)?;
    if integrity.format_version != FORMAT_VERSION
        || integrity.publisher_id != key.publisher_id
        || canonical_json(&integrity)? != payload
    {
        return Err(CartridgeError::InvalidIntegrity);
    }
    validate_integrity(&integrity, &files)?;
    let authenticated_files = files
        .iter()
        .filter(|(path, _)| path.as_str() != INTEGRITY_PATH)
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .collect::<BTreeMap<_, _>>();
    let (manifest, presentation) =
        validate_payload_files(&authenticated_files, Some((&key.publisher_id, &key.key_id)))?;

    let canonical_archive = write_canonical_zip(&files)?;
    if canonical_archive != archive_bytes {
        return Err(CartridgeError::NonCanonicalArchive);
    }

    let archive_sha256 = sha256_hex(archive_bytes);
    let signed_identity_sha256 = sha256_hex(&message);
    let provenance = integrity
        .files
        .iter()
        .map(|entry| FileProvenance {
            path: entry.path.clone(),
            media_type: entry.media_type.clone(),
            bytes: entry.bytes,
            sha256: entry.sha256.clone(),
        })
        .collect();
    let compatibility = evaluate_compatibility(&manifest, host);
    Ok(VerifiedCartridge {
        archive_bytes: archive_bytes.to_vec(),
        archive_sha256,
        signed_identity_sha256,
        key_id: key.key_id.clone(),
        manifest,
        presentation,
        files: provenance,
        expanded_bytes,
        compatibility,
        authenticated_files,
    })
}

fn collect_source_files(source: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    if !fs::symlink_metadata(source)?.file_type().is_dir() {
        return Err(CartridgeError::UnsafeFilesystemPath);
    }
    let mut files = BTreeMap::new();
    let mut pending = vec![(source.to_path_buf(), String::new())];
    let mut total = 0u64;
    while let Some((directory, prefix)) = pending.pop() {
        let mut entries = fs::read_dir(&directory)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries.into_iter().rev() {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| CartridgeError::InvalidPath)?;
            let relative = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() {
                return Err(CartridgeError::UnsafeFilesystemPath);
            }
            if metadata.is_dir() {
                if !matches!(relative.as_str(), "schemas" | "locales" | "assets") {
                    return Err(CartridgeError::InvalidPath);
                }
                pending.push((entry.path(), relative));
                continue;
            }
            if !metadata.is_file() || !valid_archive_path(&relative) || relative == INTEGRITY_PATH {
                return Err(CartridgeError::InvalidPath);
            }
            if files.len() >= MAX_ENTRIES - 1 || metadata.len() > MAX_ENTRY_BYTES {
                return Err(CartridgeError::LimitExceeded);
            }
            let bytes = read_bounded_regular_file(&entry.path(), MAX_ENTRY_BYTES)?;
            let canonical = canonicalize_payload(&relative, &bytes)?;
            total = total
                .checked_add(canonical.len() as u64)
                .ok_or(CartridgeError::LimitExceeded)?;
            if total > MAX_EXPANDED_BYTES {
                return Err(CartridgeError::LimitExceeded);
            }
            files.insert(relative, canonical);
        }
    }
    Ok(files)
}

fn validate_payload_files(
    files: &BTreeMap<String, Vec<u8>>,
    expected_publisher: Option<(&str, &str)>,
) -> Result<(CartridgeManifest, Presentation)> {
    let manifest_bytes = files
        .get(MANIFEST_PATH)
        .ok_or(CartridgeError::InvalidManifest)?;
    let manifest: CartridgeManifest = serde_json::from_slice(manifest_bytes)?;
    validate_manifest(&manifest)?;
    if canonical_json(&manifest)? != *manifest_bytes {
        return Err(CartridgeError::InvalidManifest);
    }
    if let Some((publisher_id, _key_id)) = expected_publisher
        && manifest.publisher_id != publisher_id
    {
        return Err(CartridgeError::PublisherMismatch);
    }

    let presentation_bytes = files
        .get(PRESENTATION_PATH)
        .ok_or(CartridgeError::InvalidPresentation)?;
    let presentation: Presentation = serde_json::from_slice(presentation_bytes)?;
    if canonical_json(&presentation)? != *presentation_bytes {
        return Err(CartridgeError::InvalidPresentation);
    }
    validate_presentation(
        &presentation,
        &manifest.entry_screen,
        &manifest.required_capabilities,
        &manifest.optional_capabilities,
        &manifest.schemas,
        &manifest.assets,
    )?;

    let paths = files.keys().cloned().collect::<BTreeSet<_>>();
    validate_inventory(&manifest, &paths)?;
    let assets = descriptors_by_path(&manifest);
    let mut total_decoded_assets = 0u64;
    for (path, bytes) in files {
        if valid_schema_path(path) {
            let schema = validate_schema(bytes)?;
            if canonical_json(&schema)? != *bytes {
                return Err(CartridgeError::InvalidSchema);
            }
        } else if valid_locale_path(path) {
            let localization = validate_localization(bytes)?;
            if canonical_json(&localization)? != *bytes {
                return Err(CartridgeError::InvalidLocalization);
            }
        } else if let Some(descriptor) = assets.get(path.as_str()) {
            validate_asset(bytes, descriptor)?;
            total_decoded_assets = total_decoded_assets
                .checked_add(descriptor.decoded_bytes)
                .ok_or(CartridgeError::LimitExceeded)?;
        }
    }
    if total_decoded_assets > crate::contract::MAX_DECODED_ASSET_BYTES {
        return Err(CartridgeError::LimitExceeded);
    }
    Ok((manifest, presentation))
}

fn build_integrity_index(
    files: &BTreeMap<String, Vec<u8>>,
    publisher_id: &str,
) -> Result<IntegrityIndex> {
    let entries = files
        .iter()
        .map(|(path, bytes)| {
            Ok(IntegrityEntry {
                path: path.clone(),
                media_type: integrity_media_type(path)
                    .ok_or(CartridgeError::InvalidPath)?
                    .to_owned(),
                bytes: bytes.len() as u64,
                sha256: sha256_hex(bytes),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(IntegrityIndex {
        format_version: FORMAT_VERSION,
        publisher_id: publisher_id.to_owned(),
        files: entries,
    })
}

fn validate_integrity(
    integrity: &IntegrityIndex,
    archive_files: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let payload_files = archive_files
        .iter()
        .filter(|(path, _)| path.as_str() != INTEGRITY_PATH)
        .collect::<Vec<_>>();
    if payload_files.len() != integrity.files.len() || integrity.files.is_empty() {
        return Err(CartridgeError::InvalidIntegrity);
    }
    for ((path, bytes), entry) in payload_files.into_iter().zip(&integrity.files) {
        if path != &entry.path
            || entry.bytes != bytes.len() as u64
            || entry.sha256 != sha256_hex(bytes)
            || integrity_media_type(path) != Some(entry.media_type.as_str())
            || !valid_sha256(&entry.sha256)
        {
            return Err(CartridgeError::InvalidIntegrity);
        }
    }
    Ok(())
}

fn write_canonical_zip(files: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::DEFAULT
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(DateTime::DEFAULT)
        .unix_permissions(0o444)
        .system(System::Unix);
    for (path, bytes) in files {
        writer.start_file(path, options)?;
        writer.write_all(bytes)?;
    }
    Ok(writer.finish()?.into_inner())
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
