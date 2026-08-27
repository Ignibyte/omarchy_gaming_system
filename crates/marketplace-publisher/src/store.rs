use std::{
    collections::BTreeSet,
    fs,
    os::{
        fd::AsFd,
        unix::fs::{DirBuilderExt as _, MetadataExt, PermissionsExt as _, symlink},
    },
    path::{Path, PathBuf},
};

use omarchygs_marketplace_trust::{
    MarketplaceTrust, read_trust_root_public_key, signed_trust_bytes, trust_root_sha256,
    verify_trust_transition,
};
use rand_core::{OsRng, RngCore as _};
use rustix::fs::{FlockOperation, Mode, OFlags, flock, open};

use crate::{
    CHANNEL_NAMESPACE, MARKETPLACE_NAMESPACE, MAX_FINALIZED_VERSIONS,
    MAX_PUBLICATION_MANIFEST_BYTES, OperationReceipt, PUBLIC_DIRECTORY, PUBLICATION_MANIFEST_FILE,
    PublicationFile, PublicationManifest, PublicationNamespace, PublisherError, Result, TRUST_FILE,
    canonical_json, copy_public_file, create_directory, file_limit, fsync_tree, load_prepared,
    load_response, manifest_sha256, parse_canonical, publication_file_order, publication_record,
    read_regular_file, receipt_for_manifest, require_absolute, require_absolute_private_directory,
    safe_join, sha256, validate_authenticated_inventory, validate_file_inventory,
    validate_manifest, validate_publication_core, validate_release_entry, verify_prepared_files,
    verify_regular_file_exact, write_public_file,
};

const VERSIONS_DIRECTORY: &str = "versions";
const CURRENT_LINK: &str = "current";
const STORE_LOCK: &str = ".publication.lock";

/// Finalize a prepared candidate into an immutable version directory.
pub fn finalize_publication(
    prepared_root: &Path,
    response_path: &Path,
    store_root: &Path,
    now_unix: u64,
) -> Result<OperationReceipt> {
    require_absolute(prepared_root)?;
    require_absolute(response_path)?;
    require_absolute(store_root)?;
    let (prepared, request) = load_prepared(prepared_root)?;
    let (response, trust) = load_response(response_path, &request)?;
    trust
        .validate_now(now_unix)
        .map_err(|_| PublisherError::Rejected)?;
    if request.trust_payload.current_snapshot_version != prepared.plan.snapshot_version
        || request.trust_payload.bundle_version != prepared.plan.bundle_version
        || request.trust_payload.channel_origin != prepared.plan.channel_origin
        || request.trust_payload.marketplace_origin != prepared.plan.marketplace_origin
    {
        return Err(PublisherError::Rejected);
    }
    verify_prepared_files(prepared_root, &prepared.files)?;
    let trust_bytes =
        signed_trust_bytes(&response.signed_trust).map_err(|_| PublisherError::Rejected)?;
    let mut files = prepared.files.clone();
    files.push(PublicationFile {
        namespace: PublicationNamespace::Channel,
        relative_path: TRUST_FILE.to_owned(),
        media_type: "application/json".to_owned(),
        bytes: trust_bytes.len() as u64,
        sha256: sha256(&trust_bytes),
    });
    files.sort_by(publication_file_order);
    validate_file_inventory(&files, true)?;
    let active = trust.active_key();
    let manifest = PublicationManifest {
        format: "omarchygs.marketplace-publication/v1".to_owned(),
        publication_id: prepared.plan.publication_id.clone(),
        created_at_unix: prepared.plan.created_at_unix,
        channel_origin: prepared.plan.channel_origin.clone(),
        marketplace_origin: prepared.plan.marketplace_origin.clone(),
        bundle_version: prepared.plan.bundle_version,
        snapshot_version: prepared.plan.snapshot_version,
        root_sha256: trust_root_sha256(&request.root).map_err(|_| PublisherError::Rejected)?,
        catalog_key_sha256: omarchygs_marketplace_trust::catalog_key_sha256(active)
            .map_err(|_| PublisherError::Rejected)?,
        trust_sha256: sha256(&trust_bytes),
        snapshot_sha256: prepared.snapshot_sha256.clone(),
        files,
    };
    validate_manifest(&manifest)?;
    let publication_sha256 = manifest_sha256(&manifest)?;
    let version_name = version_name(manifest.bundle_version, &publication_sha256)?;

    initialize_store(store_root)?;
    let lock = open_store_lock(store_root)?;
    flock(lock.as_fd(), FlockOperation::LockExclusive).map_err(|_| PublisherError::Storage)?;
    let result = (|| {
        let versions = store_root.join(VERSIONS_DIRECTORY);
        let target = versions.join(&version_name);
        if target.exists() {
            let (existing, _) =
                verify_version_with_root(store_root, &version_name, &request.root, now_unix)?;
            if existing != manifest {
                return Err(PublisherError::Rejected);
            }
            return receipt_for_manifest("finalize", &existing);
        }
        if finalized_version_names(&versions)?.len() >= MAX_FINALIZED_VERSIONS {
            return Err(PublisherError::Storage);
        }
        let temporary_name = temporary_version_name();
        let temporary = versions.join(&temporary_name);
        create_directory(&temporary, 0o700)?;
        let build = (|| {
            let source_public = prepared_root.join(PUBLIC_DIRECTORY);
            for record in &prepared.files {
                let source = safe_join(
                    &source_public.join(record.namespace.directory()),
                    &record.relative_path,
                )?;
                let destination = safe_join(
                    &temporary.join(record.namespace.directory()),
                    &record.relative_path,
                )?;
                if record.media_type == "application/vnd.archlinux.package" {
                    let (copied_bytes, copied_sha256) =
                        copy_public_file(&source, &destination, file_limit(record))?;
                    if copied_bytes != record.bytes || copied_sha256 != record.sha256 {
                        return Err(PublisherError::Rejected);
                    }
                } else {
                    let bytes = read_regular_file(&source, file_limit(record), false)?;
                    write_public_file(&destination, &bytes)?;
                }
            }
            write_public_file(
                &temporary.join(CHANNEL_NAMESPACE).join(TRUST_FILE),
                &trust_bytes,
            )?;
            let manifest_bytes = canonical_json(&manifest)?;
            write_public_file(
                &temporary
                    .join(CHANNEL_NAMESPACE)
                    .join(PUBLICATION_MANIFEST_FILE),
                &manifest_bytes,
            )?;
            write_public_file(
                &temporary
                    .join(MARKETPLACE_NAMESPACE)
                    .join(PUBLICATION_MANIFEST_FILE),
                &manifest_bytes,
            )?;
            fsync_tree(&temporary)?;
            let (verified, _) = verify_tree(&temporary, &request.root, now_unix)?;
            if verified != manifest {
                return Err(PublisherError::Rejected);
            }
            fs::rename(&temporary, &target).map_err(|_| PublisherError::Storage)?;
            fs::File::open(&versions)
                .and_then(|file| file.sync_all())
                .map_err(|_| PublisherError::Storage)?;
            Ok(())
        })();
        if build.is_err() && temporary.exists() {
            let _ = fs::remove_dir_all(&temporary);
        }
        build?;
        receipt_for_manifest("finalize", &manifest)
    })();
    let unlock = flock(lock.as_fd(), FlockOperation::Unlock);
    if unlock.is_err() {
        return Err(PublisherError::Storage);
    }
    result
}

/// Atomically select a complete finalized publication as the locally served tree.
pub fn activate_publication(
    store_root: &Path,
    version: &str,
    root_public_key_path: &Path,
    now_unix: u64,
) -> Result<OperationReceipt> {
    require_absolute_private_directory(store_root)?;
    if !valid_version_name(version) {
        return Err(PublisherError::InvalidInput);
    }
    let root =
        read_trust_root_public_key(root_public_key_path).map_err(|_| PublisherError::Rejected)?;
    let lock = open_store_lock(store_root)?;
    flock(lock.as_fd(), FlockOperation::LockExclusive).map_err(|_| PublisherError::Storage)?;
    let result = (|| {
        let (candidate, candidate_trust) =
            verify_version_with_root(store_root, version, &root, now_unix)?;
        if let Some(current_version) = current_version_name(store_root)? {
            let (current, current_trust) =
                verify_version_with_root(store_root, &current_version, &root, now_unix)?;
            if current == candidate {
                return receipt_for_manifest("activate", &candidate);
            }
            if candidate.bundle_version <= current.bundle_version
                || candidate.snapshot_version < current.snapshot_version
            {
                return Err(PublisherError::Rollback);
            }
            verify_trust_transition(&current_trust, &candidate_trust)
                .map_err(|_| PublisherError::Rollback)?;
        } else {
            let versions = finalized_version_names(&store_root.join(VERSIONS_DIRECTORY))?;
            let highest = versions
                .iter()
                .map(|name| &name[..20])
                .max()
                .ok_or(PublisherError::Rejected)?;
            let highest_versions = versions
                .iter()
                .filter(|name| &name[..20] == highest)
                .collect::<Vec<_>>();
            if highest_versions.len() != 1 || highest_versions[0].as_str() != version {
                return Err(PublisherError::Rollback);
            }
        }
        let temporary_name = temporary_link_name();
        let temporary = store_root.join(&temporary_name);
        symlink(format!("{VERSIONS_DIRECTORY}/{version}"), &temporary)
            .map_err(|_| PublisherError::Storage)?;
        let metadata = fs::symlink_metadata(&temporary).map_err(|_| PublisherError::Storage)?;
        let expected_target = format!("{VERSIONS_DIRECTORY}/{version}");
        if !metadata.file_type().is_symlink()
            || fs::read_link(&temporary).map_err(|_| PublisherError::Storage)?
                != Path::new(&expected_target)
        {
            let _ = fs::remove_file(&temporary);
            return Err(PublisherError::Rejected);
        }
        if fs::rename(&temporary, store_root.join(CURRENT_LINK)).is_err() {
            let _ = fs::remove_file(&temporary);
            return Err(PublisherError::Storage);
        }
        fs::File::open(store_root)
            .and_then(|file| file.sync_all())
            .map_err(|_| PublisherError::Storage)?;
        receipt_for_manifest("activate", &candidate)
    })();
    let unlock = flock(lock.as_fd(), FlockOperation::Unlock);
    if unlock.is_err() {
        return Err(PublisherError::Storage);
    }
    result
}

/// Verify one immutable local publication version.
pub fn verify_version(
    store_root: &Path,
    version: &str,
    root_public_key_path: &Path,
    now_unix: u64,
) -> Result<OperationReceipt> {
    require_absolute_private_directory(store_root)?;
    if !valid_version_name(version) {
        return Err(PublisherError::InvalidInput);
    }
    let root =
        read_trust_root_public_key(root_public_key_path).map_err(|_| PublisherError::Rejected)?;
    let (manifest, _) = verify_version_with_root(store_root, version, &root, now_unix)?;
    receipt_for_manifest("verify", &manifest)
}

/// Verify the current atomic publication pointer and its complete tree.
pub fn verify_current(
    store_root: &Path,
    root_public_key_path: &Path,
    now_unix: u64,
) -> Result<OperationReceipt> {
    require_absolute_private_directory(store_root)?;
    let version = current_version_name(store_root)?.ok_or(PublisherError::Rejected)?;
    let root =
        read_trust_root_public_key(root_public_key_path).map_err(|_| PublisherError::Rejected)?;
    let (manifest, _) = verify_version_with_root(store_root, &version, &root, now_unix)?;
    receipt_for_manifest("verify_current", &manifest)
}

pub(crate) fn verify_version_with_root(
    store_root: &Path,
    version: &str,
    root: &omarchygs_marketplace_trust::TrustRootPublicKey,
    now_unix: u64,
) -> Result<(PublicationManifest, MarketplaceTrust)> {
    if !valid_version_name(version) {
        return Err(PublisherError::InvalidInput);
    }
    let path = store_root.join(VERSIONS_DIRECTORY).join(version);
    verify_tree(&path, root, now_unix)
}

pub(crate) fn verify_tree(
    tree: &Path,
    root: &omarchygs_marketplace_trust::TrustRootPublicKey,
    now_unix: u64,
) -> Result<(PublicationManifest, MarketplaceTrust)> {
    let metadata = fs::symlink_metadata(tree).map_err(|_| PublisherError::Storage)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PublisherError::Rejected);
    }
    validate_private_directory(&metadata)?;
    let channel_manifest = read_regular_file(
        &tree.join(CHANNEL_NAMESPACE).join(PUBLICATION_MANIFEST_FILE),
        MAX_PUBLICATION_MANIFEST_BYTES as u64,
        false,
    )?;
    let marketplace_manifest = read_regular_file(
        &tree
            .join(MARKETPLACE_NAMESPACE)
            .join(PUBLICATION_MANIFEST_FILE),
        MAX_PUBLICATION_MANIFEST_BYTES as u64,
        false,
    )?;
    if channel_manifest != marketplace_manifest {
        return Err(PublisherError::Rejected);
    }
    for path in [
        tree.join(CHANNEL_NAMESPACE).join(PUBLICATION_MANIFEST_FILE),
        tree.join(MARKETPLACE_NAMESPACE)
            .join(PUBLICATION_MANIFEST_FILE),
    ] {
        let metadata = fs::metadata(path).map_err(|_| PublisherError::Storage)?;
        if metadata.mode() & 0o777 != 0o444 || metadata.nlink() != 1 {
            return Err(PublisherError::Rejected);
        }
    }
    let manifest: PublicationManifest =
        parse_canonical(&channel_manifest, MAX_PUBLICATION_MANIFEST_BYTES)?;
    validate_manifest(&manifest)?;
    let expected = expected_tree_paths(&manifest)?;
    let actual = collect_tree_paths(tree)?;
    if actual != expected {
        return Err(PublisherError::Rejected);
    }
    let trust_bytes =
        read_tree_record(tree, &manifest, &PublicationNamespace::Channel, TRUST_FILE)?;
    let snapshot_bytes = read_tree_record(
        tree,
        &manifest,
        &PublicationNamespace::Marketplace,
        crate::SNAPSHOT_FILE,
    )?;
    let (trust, snapshot) =
        validate_publication_core(&manifest, &trust_bytes, &snapshot_bytes, root, now_unix)?;
    validate_authenticated_inventory(&manifest, &trust, &snapshot)?;
    let catalog_key = trust.active_key();
    for entry in &snapshot.releases {
        let archive = read_tree_record(
            tree,
            &manifest,
            &PublicationNamespace::Marketplace,
            &format!(
                "{}{}",
                entry.release_path,
                omarchygs_game_cartridge::RELEASE_ARCHIVE_PATH
            ),
        )?;
        let conformance = read_tree_record(
            tree,
            &manifest,
            &PublicationNamespace::Marketplace,
            &format!(
                "{}{}",
                entry.release_path,
                omarchygs_game_cartridge::RELEASE_CONFORMANCE_PATH
            ),
        )?;
        let attestation = read_tree_record(
            tree,
            &manifest,
            &PublicationNamespace::Marketplace,
            &format!(
                "{}{}",
                entry.release_path,
                omarchygs_game_cartridge::RELEASE_ATTESTATION_PATH
            ),
        )?;
        validate_release_entry(entry, &archive, &conformance, &attestation, catalog_key)?;
    }
    for artifact in &trust.payload().packages {
        let record = publication_record(
            &manifest,
            &PublicationNamespace::Channel,
            &artifact.relative_path,
        )?;
        let path = safe_join(&tree.join(CHANNEL_NAMESPACE), &artifact.relative_path)?;
        validate_public_file_mode(&path)?;
        verify_regular_file_exact(&path, record)?;
    }
    Ok((manifest, trust))
}

fn read_tree_record(
    tree: &Path,
    manifest: &PublicationManifest,
    namespace: &PublicationNamespace,
    relative_path: &str,
) -> Result<Vec<u8>> {
    let record = publication_record(manifest, namespace, relative_path)?;
    let path = safe_join(&tree.join(namespace.directory()), relative_path)?;
    validate_public_file_mode(&path)?;
    let bytes = read_regular_file(&path, file_limit(record), false)?;
    if bytes.len() as u64 != record.bytes || sha256(&bytes) != record.sha256 {
        return Err(PublisherError::Rejected);
    }
    Ok(bytes)
}

fn validate_public_file_mode(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).map_err(|_| PublisherError::Storage)?;
    if metadata.mode() & 0o777 != 0o444 || metadata.nlink() != 1 {
        Err(PublisherError::Rejected)
    } else {
        Ok(())
    }
}

pub(crate) fn current_version_name(store_root: &Path) -> Result<Option<String>> {
    let current = store_root.join(CURRENT_LINK);
    let metadata = match fs::symlink_metadata(&current) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(PublisherError::Storage),
    };
    if !metadata.file_type().is_symlink() {
        return Err(PublisherError::Rejected);
    }
    let target = fs::read_link(&current).map_err(|_| PublisherError::Storage)?;
    let text = target.to_str().ok_or(PublisherError::Rejected)?;
    let version = text
        .strip_prefix(&format!("{VERSIONS_DIRECTORY}/"))
        .ok_or(PublisherError::Rejected)?;
    if !valid_version_name(version) || text.contains('/') && target.components().count() != 2 {
        return Err(PublisherError::Rejected);
    }
    Ok(Some(version.to_owned()))
}

fn initialize_store(root: &Path) -> Result<()> {
    ensure_private_directory(root)?;
    let versions = root.join(VERSIONS_DIRECTORY);
    ensure_private_directory(&versions)?;
    let lock = root.join(STORE_LOCK);
    match open(
        &lock,
        OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW,
        Mode::RUSR | Mode::WUSR,
    ) {
        Ok(file) => {
            let mut file = fs::File::from(file);
            std::io::Write::write_all(&mut file, b"marketplace-publication-lock-v1\n")
                .map_err(|_| PublisherError::Storage)?;
            file.sync_all().map_err(|_| PublisherError::Storage)?;
        }
        Err(rustix::io::Errno::EXIST) => {}
        Err(_) => return Err(PublisherError::Storage),
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    match builder.create(path) {
        Ok(()) => {
            let directory = open(
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
                Mode::empty(),
            )
            .map_err(|_| PublisherError::Storage)?;
            fs::File::from(directory)
                .set_permissions(fs::Permissions::from_mode(0o700))
                .map_err(|_| PublisherError::Storage)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(PublisherError::Storage),
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| PublisherError::Storage)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || validate_private_directory(&metadata).is_err()
    {
        return Err(PublisherError::Rejected);
    }
    Ok(())
}

fn open_store_lock(root: &Path) -> Result<fs::File> {
    let path = root.join(STORE_LOCK);
    let metadata = fs::symlink_metadata(&path).map_err(|_| PublisherError::Storage)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.mode() & 0o777 != 0o600
        || metadata.nlink() != 1
    {
        return Err(PublisherError::Rejected);
    }
    open(&path, OFlags::RDWR | OFlags::NOFOLLOW, Mode::empty())
        .map(fs::File::from)
        .map_err(|_| PublisherError::Storage)
}

fn finalized_version_names(versions: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut temporary = 0_usize;
    for entry in fs::read_dir(versions).map_err(|_| PublisherError::Storage)? {
        let entry = entry.map_err(|_| PublisherError::Storage)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| PublisherError::Rejected)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|_| PublisherError::Storage)?;
        if name.starts_with('.') {
            temporary += 1;
            if temporary > MAX_FINALIZED_VERSIONS
                || !valid_temporary_version_name(&name)
                || !metadata.is_dir()
                || metadata.file_type().is_symlink()
                || validate_private_directory(&metadata).is_err()
            {
                return Err(PublisherError::Rejected);
            }
            continue;
        }
        if !valid_version_name(&name) || !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(PublisherError::Rejected);
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn expected_tree_paths(manifest: &PublicationManifest) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::from([
        format!("{CHANNEL_NAMESPACE}/{PUBLICATION_MANIFEST_FILE}"),
        format!("{MARKETPLACE_NAMESPACE}/{PUBLICATION_MANIFEST_FILE}"),
    ]);
    for record in &manifest.files {
        let path = format!("{}/{}", record.namespace.directory(), record.relative_path);
        if !paths.insert(path) {
            return Err(PublisherError::Rejected);
        }
    }
    Ok(paths)
}

fn collect_tree_paths(root: &Path) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    let mut directories = vec![PathBuf::new()];
    while let Some(relative) = directories.pop() {
        let directory = root.join(&relative);
        for entry in fs::read_dir(&directory).map_err(|_| PublisherError::Storage)? {
            let entry = entry.map_err(|_| PublisherError::Storage)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| PublisherError::Rejected)?;
            if name.is_empty() || name == "." || name == ".." {
                return Err(PublisherError::Rejected);
            }
            let child = relative.join(&name);
            let metadata =
                fs::symlink_metadata(entry.path()).map_err(|_| PublisherError::Storage)?;
            if metadata.file_type().is_symlink() {
                return Err(PublisherError::Rejected);
            }
            if metadata.is_dir() {
                validate_private_directory(&metadata)?;
                directories.push(child);
            } else if metadata.is_file() && metadata.nlink() == 1 {
                paths.insert(child.to_str().ok_or(PublisherError::Rejected)?.to_owned());
            } else {
                return Err(PublisherError::Rejected);
            }
        }
    }
    Ok(paths)
}

fn version_name(bundle_version: u64, digest: &str) -> Result<String> {
    if digest.len() != 64 {
        return Err(PublisherError::InvalidInput);
    }
    Ok(format!("{bundle_version:020}-{digest}"))
}

fn valid_version_name(value: &str) -> bool {
    let Some((version, digest)) = value.split_once('-') else {
        return false;
    };
    version.len() == 20
        && version.bytes().all(|byte| byte.is_ascii_digit())
        && version.parse::<u64>().is_ok_and(|value| value > 0)
        && digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn temporary_version_name() -> String {
    format!(".publication-{}", random_hex())
}

fn valid_temporary_version_name(value: &str) -> bool {
    value.strip_prefix(".publication-").is_some_and(|suffix| {
        suffix.len() == 32
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn validate_private_directory(metadata: &fs::Metadata) -> Result<()> {
    if metadata.uid() != rustix::process::geteuid().as_raw() || metadata.mode() & 0o777 != 0o700 {
        Err(PublisherError::Rejected)
    } else {
        Ok(())
    }
}

fn temporary_link_name() -> String {
    format!(".current-{}", random_hex())
}

fn random_hex() -> String {
    let mut bytes = [0_u8; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
