use std::collections::BTreeSet;

use crate::contract::{
    CartridgeManifest, CompatibilityReport, HostProfile, OptionalFallbackSelection,
    VersionCompatibility, VersionRange,
};

pub fn evaluate_compatibility(
    manifest: &CartridgeManifest,
    host: &HostProfile,
) -> CompatibilityReport {
    let sdk = compare_version(host.sdk_version, &manifest.sdk);
    let presentation_protocol = compare_version(
        host.presentation_protocol_version,
        &manifest.presentation_protocol,
    );
    let missing_required_capabilities = manifest
        .required_capabilities
        .iter()
        .filter(|capability| !host.capabilities.contains(*capability))
        .cloned()
        .collect::<Vec<_>>();
    let selected_optional_fallbacks = manifest
        .optional_capabilities
        .iter()
        .filter(|optional| !host.capabilities.contains(&optional.capability))
        .map(|optional| OptionalFallbackSelection {
            capability: optional.capability.clone(),
            fallback: optional.fallback.clone(),
        })
        .collect::<Vec<_>>();

    CompatibilityReport {
        compatible: sdk == VersionCompatibility::Compatible
            && presentation_protocol == VersionCompatibility::Compatible
            && missing_required_capabilities.is_empty(),
        sdk,
        presentation_protocol,
        missing_required_capabilities,
        selected_optional_fallbacks,
    }
}

fn compare_version(host: u32, range: &VersionRange) -> VersionCompatibility {
    if host < range.min {
        VersionCompatibility::HostTooOld
    } else if host > range.max {
        VersionCompatibility::HostTooNew
    } else {
        VersionCompatibility::Compatible
    }
}

pub fn baseline_host_profile() -> HostProfile {
    core_host_profile()
}

pub fn core_host_profile() -> HostProfile {
    HostProfile {
        sdk_version: 1,
        presentation_protocol_version: 1,
        capabilities: BTreeSet::from([
            "presentation.grid.v1".to_owned(),
            "presentation.button.v1".to_owned(),
            "presentation.image.v1".to_owned(),
            "presentation.meter.v1".to_owned(),
            "presentation.navigation.v1".to_owned(),
            "presentation.status.v1".to_owned(),
            "presentation.terminal.v1".to_owned(),
        ]),
    }
}

pub fn rich_2d_host_profile() -> HostProfile {
    let mut profile = core_host_profile();
    profile.capabilities.extend([
        "audio.effects.v1".to_owned(),
        "presentation.particles.v1".to_owned(),
        "presentation.sprite.v1".to_owned(),
    ]);
    profile
}
