use std::path::PathBuf;

use omarchygs_provider_conformance::{
    DeveloperKitReleaseSigner, export_developer_kit, verify_developer_kit,
};

fn main() {
    let arguments = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    assert_eq!(
        arguments.len(),
        4,
        "usage: provider-kit-consumer SDK.crate STARTER.crate CONFORMANCE.crate OUTPUT"
    );
    let signer = DeveloperKitReleaseSigner::new("omarchygs", "developer-kit-1", [45; 32])
        .expect("developer kit signer");
    let identity = export_developer_kit(
        &arguments[3],
        &arguments[0],
        &arguments[1],
        &arguments[2],
        &signer,
        &"c".repeat(40),
        &"e".repeat(64),
    )
    .expect("developer kit export");
    assert_eq!(
        verify_developer_kit(
            &arguments[3],
            &signer.verifying_key(),
            "omarchygs",
            "developer-kit-1",
        )
        .expect("developer kit verification"),
        identity,
    );
    println!(
        "{}",
        serde_json::to_string(&identity).expect("identity JSON")
    );
}
