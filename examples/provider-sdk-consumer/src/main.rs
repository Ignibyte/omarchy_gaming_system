use std::{env, fs, path::PathBuf};

use omarchygs_provider_sdk::release::{
    ProviderSdkReleaseSigner, export_sdk, verify_sdk_directory,
};

const SOURCE_REVISION: &str = "1111111111111111111111111111111111111111";
const BUILDER_SHA256: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn main() {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: provider-sdk-consumer <empty-output-directory>");
    fs::create_dir(&output).expect("output directory should create");
    let signer = ProviderSdkReleaseSigner::new(
        "omarchygs",
        "provider-sdk-preview-v1",
        [9_u8; 32],
    )
    .expect("fixture release signer should construct");
    let exported = export_sdk(&output, &signer, SOURCE_REVISION, BUILDER_SHA256)
        .expect("SDK should export");
    let verified = verify_sdk_directory(
        &output,
        &signer.verifying_key(),
        "omarchygs",
        "provider-sdk-preview-v1",
    )
    .expect("SDK should verify");
    assert_eq!(exported, verified);
    println!(
        "{} {} {}",
        verified.lock_sha256, verified.release_sha256, verified.source_revision
    );
}
