import QtQuick

QtObject {
    id: root

    property string helperEndpoint: ""
    property string helperCredential: ""
    property bool configured: false
    property var trust: null
    property var packages: []
    property var stagedPackage: null
    property string statusText: configured
                                ? "Checking marketplace trust..."
                                : "This build has no marketplace trust configured."
    property string errorText: ""
    property string loadState: configured ? "idle" : "unavailable"
    readonly property bool busy: api.requestInFlight
    readonly property bool marketplaceReady: trust !== null
                                                 && trust.enrolled === true
                                                 && trust.state === "current"
    readonly property bool channelMode: trust !== null && trust.mode === "channel"

    property ApiClient _api: ApiClient {
        id: api
        maximumResponseBytes: 524288
        timeoutMilliseconds: 30000
        onFinished: function(generation, operation, status, body, transportError) {
            root._handle(generation, operation, status, body, transportError)
        }
    }

    onHelperEndpointChanged: reset()
    onHelperCredentialChanged: reset()
    onConfiguredChanged: reset()

    function reset() {
        api.cancel()
        trust = null
        packages = []
        stagedPackage = null
        errorText = ""
        loadState = configured ? "idle" : "unavailable"
        statusText = configured
                ? "Marketplace trust is ready to check."
                : "This build has no marketplace trust configured."
    }

    function refresh() {
        if (!configured || busy || !_configure())
            return false
        errorText = ""
        statusText = "Checking independently authenticated marketplace trust..."
        loadState = "loading"
        return api.request("trust_status", "GET", "/v1/trust", null, true) !== 0
    }

    function synchronize() {
        if (!configured || busy || !_configure())
            return false
        errorText = ""
        statusText = trust !== null && trust.enrolled
                ? "Synchronizing marketplace key rotation and revocation state..."
                : "Enrolling this player in the packaged marketplace channel..."
        loadState = "loading"
        return api.request("trust_sync", "POST", "/v1/trust/synchronize", null, true) !== 0
    }

    function stage(artifact) {
        if (!marketplaceReady || busy || !_validArtifact(artifact) || !_configure())
            return false
        errorText = ""
        statusText = "Downloading and verifying the exact reviewed client package..."
        loadState = "loading"
        return api.request("package_stage", "POST", "/v1/client-packages/stage", {
            "sha256": artifact.sha256
        }, true) !== 0
    }

    function _configure() {
        const configuredApi = api.configure(helperEndpoint)
        if (!configuredApi.ok || helperCredential === "") {
            loadState = "error"
            errorText = "The local marketplace companion is unavailable."
            return false
        }
        api.installBearer(helperCredential)
        return true
    }

    function _handle(generation, operation, status, body, transportError) {
        if (transportError !== "") {
            loadState = "error"
            statusText = "Marketplace operation did not complete."
            errorText = transportError === "timeout"
                    ? "The marketplace channel timed out. Existing trust was preserved."
                    : "The marketplace channel is offline. Existing trust was preserved."
            return
        }
        const parsed = _parse(body)
        if (status < 200 || status >= 300 || !parsed.ok) {
            loadState = "error"
            statusText = "Marketplace operation was rejected."
            errorText = _helperError(parsed.ok ? _errorCode(parsed.document) : "")
            return
        }
        if (operation === "trust_status" || operation === "trust_sync") {
            if (!_validTrust(parsed.document)) {
                _protocolFailure("The marketplace trust status was malformed.")
                return
            }
            trust = parsed.document
            stagedPackage = null
            if (trust.state !== "current") {
                packages = []
                loadState = trust.state === "expired" ? "error" : "unavailable"
                statusText = trust.state === "expired"
                        ? "Marketplace trust expired; cartridges and package downloads are paused."
                        : trust.mode === "channel"
                          ? "Marketplace channel is packaged but not enrolled."
                          : "Marketplace trust is unavailable."
                errorText = trust.state === "expired"
                        ? "Synchronize a current root-signed channel document to continue." : ""
                return
            }
            statusText = trust.mode === "manual"
                    ? "Manual marketplace key is active."
                    : trust.channel_name + " // BUNDLE " + trust.bundle_version
            loadState = "ready"
            if (trust.mode === "channel") {
                api.request("package_status", "GET", "/v1/client-packages", null, true)
            }
            return
        }
        if (operation === "package_status") {
            if (!_validPackageStatus(parsed.document)) {
                _protocolFailure("The client package inventory was malformed.")
                return
            }
            packages = parsed.document.available
            loadState = "ready"
            statusText = packages.length === 0
                    ? "Marketplace trust is current; no newer reviewed client package is available."
                    : "A newer root-authenticated client package is available."
            errorText = ""
            return
        }
        if (operation === "package_stage") {
            if (!_validStagedPackage(parsed.document)) {
                _protocolFailure("The staged client package receipt was malformed.")
                return
            }
            stagedPackage = parsed.document
            loadState = "ready"
            statusText = "Reviewed client package verified and staged without executing an installer."
            errorText = ""
        }
    }

    function _validTrust(value) {
        if (!value || typeof value !== "object")
            return false
        const keys = ["format", "mode", "state", "enrolled", "keys"]
        ;["channel_id", "channel_name", "channel_origin", "bundle_version",
          "expires_at_unix"].forEach(function(key) {
            if (value[key] !== undefined)
                keys.push(key)
        })
        return api.exactKeys(value, keys)
                && value.format === "omarchygs.client-trust-status/v1"
                && ["none", "manual", "channel"].indexOf(value.mode) !== -1
                && ["unavailable", "unenrolled", "current", "expired"].indexOf(value.state) !== -1
                && typeof value.enrolled === "boolean"
                && Array.isArray(value.keys) && value.keys.length <= 16
                && value.keys.every(function(key) { return root._validTrustKey(key) })
                && (value.channel_id === undefined || _validIdentifier(value.channel_id))
                && (value.channel_name === undefined || _validText(value.channel_name, 128))
                && (value.channel_origin === undefined || _validText(value.channel_origin, 512))
                && (value.bundle_version === undefined
                    || (Number.isInteger(value.bundle_version) && value.bundle_version > 0))
                && (value.expires_at_unix === undefined
                    || (Number.isInteger(value.expires_at_unix) && value.expires_at_unix > 0))
    }

    function _validTrustKey(value) {
        if (!value || typeof value !== "object")
            return false
        const keys = ["key_sha256", "status", "first_snapshot_version"]
        if (value.last_snapshot_version !== undefined)
            keys.push("last_snapshot_version")
        return api.exactKeys(value, keys) && _validDigest(value.key_sha256)
                && ["active", "retired", "revoked"].indexOf(value.status) !== -1
                && Number.isInteger(value.first_snapshot_version)
                && value.first_snapshot_version > 0
                && (value.last_snapshot_version === undefined
                    || (Number.isInteger(value.last_snapshot_version)
                        && value.last_snapshot_version >= value.first_snapshot_version))
    }

    function _validPackageStatus(value) {
        return api.exactKeys(value, ["format", "platform", "architecture",
                                     "installed_package_version", "available"])
                && value.format === "omarchygs.client-package-status/v1"
                && _validIdentifier(value.platform) && _validIdentifier(value.architecture)
                && _validText(value.installed_package_version, 64)
                && Array.isArray(value.available) && value.available.length <= 32
                && value.available.every(function(artifact) {
                    return root._validArtifact(artifact)
                })
    }

    function _validArtifact(value) {
        return api.exactKeys(value, ["platform", "architecture", "package_version",
                                     "filename", "relative_path", "bytes", "sha256",
                                     "source_revision", "source_sha256",
                                     "build_provenance_sha256"])
                && _validIdentifier(value.platform) && _validIdentifier(value.architecture)
                && _validVersion(value.package_version) && _validPathSegment(value.filename)
                && _validRelativePath(value.relative_path)
                && value.relative_path.endsWith(value.filename)
                && Number.isInteger(value.bytes) && value.bytes > 0 && value.bytes <= 268435456
                && _validDigest(value.sha256)
                && /^[0-9a-f]{40}$|^[0-9a-f]{64}$/.test(value.source_revision)
                && _validDigest(value.source_sha256)
                && _validDigest(value.build_provenance_sha256)
    }

    function _validStagedPackage(value) {
        return api.exactKeys(value, ["format", "package_version", "bytes", "sha256",
                                     "source_revision", "source_sha256",
                                     "build_provenance_sha256", "staged_path",
                                     "install_command"])
                && value.format === "omarchygs.staged-client-package/v1"
                && _validText(value.package_version, 64)
                && Number.isInteger(value.bytes) && value.bytes > 0 && value.bytes <= 268435456
                && _validDigest(value.sha256)
                && /^[0-9a-f]{40}$|^[0-9a-f]{64}$/.test(value.source_revision)
                && _validDigest(value.source_sha256)
                && _validDigest(value.build_provenance_sha256)
                && typeof value.staged_path === "string" && value.staged_path.startsWith("/")
                && _validText(value.staged_path, 1024)
                && _validText(value.install_command, 2048)
                && value.install_command === "sudo pacman -U -- "
                                             + _shellQuote(value.staged_path)
    }

    function _parse(body) {
        try {
            const value = JSON.parse(body)
            return value && typeof value === "object" && !Array.isArray(value)
                    ? {"ok": true, "document": value} : {"ok": false}
        } catch (error) {
            return {"ok": false}
        }
    }

    function _errorCode(document) {
        return document && api.exactKeys(document, ["error"])
                && api.exactKeys(document.error, ["code"])
                && typeof document.error.code === "string" ? document.error.code : ""
    }

    function _helperError(code) {
        const messages = {
            "companion_marketplace_untrusted": "Marketplace trust is absent, expired, revoked, or outside its signed range.",
            "companion_server_unavailable": "The independent marketplace channel is unavailable; prior trust was preserved.",
            "companion_server_rejected": "The root-signed channel or exact package bytes were rejected.",
            "companion_cache_failure": "The private marketplace trust store needs attention."
        }
        return messages[code] || "The marketplace operation failed closed."
    }

    function _protocolFailure(message) {
        loadState = "error"
        statusText = "Marketplace protocol error."
        errorText = message
    }

    function _validIdentifier(value) {
        return typeof value === "string" && /^[a-z][a-z0-9._-]{0,95}$/.test(value)
    }

    function _validVersion(value) {
        return typeof value === "string" && value.length >= 1 && value.length <= 64
                && /^[A-Za-z0-9._+-]+$/.test(value)
    }

    function _validPathSegment(value) {
        return typeof value === "string" && value.length >= 1 && value.length <= 192
                && value !== "." && value !== ".."
                && /^[a-z0-9._+-]+$/.test(value)
    }

    function _validRelativePath(value) {
        return typeof value === "string" && value.length >= 1 && value.length <= 512
                && !value.startsWith("/") && !value.endsWith("/")
                && value.indexOf("//") === -1 && value.indexOf("%") === -1
                && value.indexOf("?") === -1 && value.indexOf("#") === -1
                && value.indexOf("\\") === -1
                && value.split("/").every(function(segment) {
                    return root._validPathSegment(segment)
                })
    }

    function _shellQuote(value) {
        return "'" + value.replace(/'/g, "'\\''") + "'"
    }

    function _validDigest(value) {
        return typeof value === "string" && /^[0-9a-f]{64}$/.test(value)
    }

    function _validText(value, maximum) {
        return typeof value === "string" && value.length >= 1 && value.length <= maximum
                && value.trim() === value && !/[\u0000-\u001f\u007f-\u009f]/.test(value)
    }
}
