import QtQuick

QtObject {
    id: root

    required property var sessionController
    property var actor: null
    property string helperEndpoint: ""
    property string helperCredential: ""
    property bool marketplaceTrusted: false
    property var catalog: []
    property var mounts: []
    property string statusText: ""
    property string errorText: ""
    property string loadState: "idle"
    readonly property bool busy: _serverGeneration !== 0 || _helperGeneration !== 0
    readonly property bool helperAvailable: helperEndpoint !== "" && helperCredential !== ""
    readonly property bool acquisitionSupported: _authority !== null
                                                 && _authority.acquisition_supported === true

    property int _serverGeneration: 0
    property string _serverOperation: ""
    property int _helperGeneration: 0
    property string _helperOperation: ""
    property var _authority: null

    property ApiClient _serverApi: ApiClient {
        id: serverApi
        maximumResponseBytes: 262144
        onFinished: function(generation, operation, status, body, transportError) {
            root._handleServer(generation, operation, status, body, transportError)
        }
    }

    property ApiClient _helperApi: ApiClient {
        id: helperApi
        maximumResponseBytes: 262144
        onFinished: function(generation, operation, status, body, transportError) {
            root._handleHelper(generation, operation, status, body, transportError)
        }
    }

    onActorChanged: reset()
    onHelperEndpointChanged: reset()
    onHelperCredentialChanged: reset()
    onMarketplaceTrustedChanged: reset()

    function reset() {
        serverApi.cancel()
        helperApi.cancel()
        _serverGeneration = 0
        _serverOperation = ""
        _helperGeneration = 0
        _helperOperation = ""
        _authority = null
        catalog = []
        mounts = []
        statusText = ""
        errorText = ""
        loadState = "idle"
    }

    function refresh() {
        if (busy || actor === null)
            return false
        const authority = sessionController.trustedCartridgeAuthority()
        if (authority === null) {
            catalog = []
            mounts = []
            loadState = "unavailable"
            statusText = "This server does not publish a cartridge catalog."
            errorText = ""
            return false
        }
        _authority = authority
        const configured = serverApi.configure(authority.origin)
        if (!configured.ok) {
            _protocolFailure("The selected server origin is invalid.")
            return false
        }
        serverApi.installBearer(authority.device_bearer)
        if (helperAvailable) {
            const helperConfigured = helperApi.configure(helperEndpoint)
            if (!helperConfigured.ok) {
                _protocolFailure("The local cartridge companion is invalid.")
                return false
            }
            helperApi.installBearer(helperCredential)
        }
        errorText = ""
        statusText = "Loading signed server cartridges..."
        loadState = "loading"
        _serverOperation = "cartridge_catalog"
        _serverGeneration = serverApi.request(
                    _serverOperation, "GET", "/v1/cartridges", null, true)
        return _serverGeneration !== 0
    }

    function install(release) {
        if (!_readyForMutation(release) || !helperAvailable || !marketplaceTrusted
                || !acquisitionSupported || isMountedExact(release))
            return false
        errorText = ""
        statusText = "Installing exact signed cartridge..."
        loadState = "loading"
        _helperOperation = "cartridge_install"
        _helperGeneration = helperApi.request(_helperOperation, "POST", "/v1/acquisitions", {
            "server_origin": _authority.origin,
            "server_id": _authority.server_id,
            "device_bearer": _authority.device_bearer,
            "game_key": release.game_key,
            "archive_sha256": release.archive_sha256,
            "admission_revision": release.server_admission.revision
        }, true)
        return _helperGeneration !== 0
    }

    function remove(release) {
        const mount = mountForExact(release)
        if (!_readyForMutation(release) || !helperAvailable || mount === null)
            return false
        errorText = ""
        statusText = "Removing this server profile mount..."
        loadState = "loading"
        _helperOperation = "cartridge_remove"
        _helperGeneration = helperApi.request(_helperOperation, "POST", "/v1/removals", {
            "server_id": _authority.server_id,
            "game_key": mount.game_key,
            "archive_sha256": mount.archive_sha256,
            "admission_revision": mount.admission_revision
        }, true)
        return _helperGeneration !== 0
    }

    function mountFor(gameKey) {
        for (let index = 0; index < mounts.length; index++) {
            if (mounts[index].game_key === gameKey)
                return mounts[index]
        }
        return null
    }

    function isMountedExact(release) {
        return mountForExact(release) !== null
    }

    function mountForExact(release) {
        if (!_validRelease(release))
            return null
        for (let index = 0; index < mounts.length; index++) {
            const mount = mounts[index]
            if (mount.game_key === release.game_key
                    && mount.archive_sha256 === release.archive_sha256
                    && mount.admission_revision === release.server_admission.revision)
                return mount
        }
        return null
    }

    function actionLabel(release) {
        if (isMountedExact(release))
            return "MOUNTED"
        return "INSTALL"
    }

    function _readyForMutation(release) {
        return !busy && _authority !== null && _validRelease(release)
    }

    function _handleServer(generation, operation, status, body, transportError) {
        if (generation !== _serverGeneration || operation !== _serverOperation)
            return
        _serverGeneration = 0
        _serverOperation = ""
        if (transportError !== "") {
            _transportFailure(transportError)
            return
        }
        const parsed = _parse(body)
        if (status === 401 && parsed.ok && _errorCode(parsed.document) === "invalid_session") {
            reset()
            sessionController.invalidatePlayerSession("This device session is no longer valid.")
            return
        }
        if (status !== 200 || !parsed.ok || !_validCatalog(parsed.document)) {
            _protocolFailure("The signed cartridge catalog was not accepted.")
            return
        }
        catalog = parsed.document.cartridges
        if (!helperAvailable) {
            mounts = []
            loadState = "unavailable"
            statusText = acquisitionSupported
                    ? "Cartridges are available; install the native companion to mount them."
                    : "This server publishes cartridge metadata but does not offer downloads."
            return
        }
        if (!marketplaceTrusted) {
            mounts = []
            loadState = "unavailable"
            statusText = "Cartridges are available, but this client has no independently trusted marketplace key."
            return
        }
        statusText = "Loading local cartridge mounts..."
        _helperOperation = "cartridge_mounts"
        _helperGeneration = helperApi.request(
                    _helperOperation, "GET", "/v1/mounts/" + _authority.server_id,
                    null, true)
    }

    function _handleHelper(generation, operation, status, body, transportError) {
        if (generation !== _helperGeneration || operation !== _helperOperation)
            return
        _helperGeneration = 0
        _helperOperation = ""
        if (transportError !== "") {
            _transportFailure(transportError)
            return
        }
        const parsed = _parse(body)
        if (status < 200 || status >= 300 || !parsed.ok) {
            loadState = "error"
            statusText = "The cartridge operation did not complete."
            errorText = _helperError(parsed.ok ? _errorCode(parsed.document) : "")
            return
        }
        if (operation === "cartridge_mounts") {
            if (!_validMountList(parsed.document)) {
                _protocolFailure("The local mount inventory was not accepted.")
                return
            }
            mounts = parsed.document.mounts
            loadState = "ready"
            statusText = catalog.length === 0
                    ? "No signed cartridges are available on this server."
                    : acquisitionSupported ? "Signed cartridge library ready."
                    : "Cartridge metadata is ready; this server does not offer downloads."
            errorText = ""
            return
        }
        if (operation === "cartridge_install") {
            if (!serverApi.exactKeys(parsed.document, ["mount"])
                    || !_validMount(parsed.document.mount)) {
                _protocolFailure("The installed mount receipt was not accepted.")
                return
            }
            statusText = "Exact signed cartridge mounted."
        } else if (operation === "cartridge_remove") {
            if (!serverApi.exactKeys(parsed.document, ["removed"])
                    || parsed.document.removed !== true) {
                _protocolFailure("The removal receipt was not accepted.")
                return
            }
            statusText = "Server profile mount removed; authoritative game state is unchanged."
        }
        loadState = "ready"
        errorText = ""
        Qt.callLater(function() { root.refresh() })
    }

    function _validCatalog(document) {
        return serverApi.exactKeys(document, ["cartridges"])
                && Array.isArray(document.cartridges)
                && document.cartridges.length <= 128
                && document.cartridges.every(function(release) {
                    return root._validRelease(release)
                })
    }

    function _validRelease(release) {
        if (!release || typeof release !== "object")
            return false
        const keys = ["game_key", "publisher_id", "rules_version",
                      "cartridge_version", "display_name", "archive_sha256",
                      "signed_identity_sha256", "marketplace", "server_admission"]
        if (release.warning !== undefined)
            keys.push("warning")
        return serverApi.exactKeys(release, keys)
                && _validIdentifier(release.game_key)
                && _validIdentifier(release.publisher_id)
                && Number.isInteger(release.rules_version) && release.rules_version > 0
                && Number.isInteger(release.cartridge_version) && release.cartridge_version > 0
                && _validText(release.display_name, 128)
                && _validDigest(release.archive_sha256)
                && _validDigest(release.signed_identity_sha256)
                && _validMarketplace(release.marketplace)
                && serverApi.exactKeys(release.server_admission, ["revision"])
                && Number.isInteger(release.server_admission.revision)
                && release.server_admission.revision > 0
                && (release.warning === undefined || _validText(release.warning, 512))
    }

    function _validMarketplace(value) {
        return serverApi.exactKeys(value, ["provenance_class", "marketplace_id",
                                          "marketplace_name", "reviewed_by",
                                          "review_summary", "policy_version",
                                          "lifecycle_status"])
                && value.provenance_class === "marketplace_vetted"
                && _validIdentifier(value.marketplace_id)
                && _validText(value.marketplace_name, 128)
                && _validIdentifier(value.reviewed_by)
                && _validText(value.review_summary, 512)
                && Number.isInteger(value.policy_version) && value.policy_version > 0
                && ["active", "deprecated"].indexOf(value.lifecycle_status) !== -1
    }

    function _validMountList(document) {
        return serverApi.exactKeys(document, ["mounts"])
                && Array.isArray(document.mounts) && document.mounts.length <= 128
                && document.mounts.every(function(mount) { return root._validMount(mount) })
    }

    function _validMount(mount) {
        if (!mount || typeof mount !== "object")
            return false
        const keys = ["format", "server_id", "server_origin", "game_key",
                      "publisher_id", "rules_version", "cartridge_version",
                      "display_name", "archive_sha256", "signed_identity_sha256",
                      "marketplace_key_sha256",
                      "marketplace_id", "marketplace_name", "reviewed_by",
                      "review_summary", "snapshot_version", "policy_version",
                      "lifecycle_status", "admission_revision"]
        if (mount.warning !== undefined)
            keys.push("warning")
        return serverApi.exactKeys(mount, keys)
                && mount.format === "omarchygs.client-cartridge-mount/v1"
                && mount.server_id === _authority.server_id
                && mount.server_origin === _authority.origin
                && _validIdentifier(mount.game_key) && _validIdentifier(mount.publisher_id)
                && Number.isInteger(mount.rules_version) && mount.rules_version > 0
                && Number.isInteger(mount.cartridge_version) && mount.cartridge_version > 0
                && _validText(mount.display_name, 128)
                && _validDigest(mount.archive_sha256)
                && _validDigest(mount.signed_identity_sha256)
                && _validDigest(mount.marketplace_key_sha256)
                && _validIdentifier(mount.marketplace_id)
                && _validText(mount.marketplace_name, 128)
                && _validIdentifier(mount.reviewed_by)
                && _validText(mount.review_summary, 512)
                && Number.isInteger(mount.snapshot_version) && mount.snapshot_version > 0
                && Number.isInteger(mount.policy_version) && mount.policy_version > 0
                && ["active", "deprecated"].indexOf(mount.lifecycle_status) !== -1
                && Number.isInteger(mount.admission_revision) && mount.admission_revision > 0
                && (mount.warning === undefined || _validText(mount.warning, 512))
    }

    function _parse(body) {
        if (typeof body !== "string" || body.length === 0)
            return {"ok": false}
        try {
            const document = JSON.parse(body)
            return document && typeof document === "object" && !Array.isArray(document)
                    ? {"ok": true, "document": document} : {"ok": false}
        } catch (error) {
            return {"ok": false}
        }
    }

    function _errorCode(document) {
        return document && serverApi.exactKeys(document, ["error"])
                && serverApi.exactKeys(document.error, ["code"])
                && typeof document.error.code === "string" ? document.error.code : ""
    }

    function _helperError(code) {
        const messages = {
            "companion_admission_changed": "The server changed this cartridge admission. Refresh and try again.",
            "companion_server_unavailable": "The selected server could not complete the download.",
            "companion_server_rejected": "The signed cartridge evidence was rejected.",
            "companion_marketplace_untrusted": "Configure an independently trusted marketplace public key before installing cartridges.",
            "companion_cache_failure": "The private local cartridge cache needs attention."
        }
        return messages[code] || "The local cartridge companion rejected the operation."
    }

    function _transportFailure(error) {
        loadState = "error"
        statusText = "The cartridge operation did not complete."
        errorText = error === "timeout" ? "The cartridge request timed out."
                  : error === "response_too_large" ? "The cartridge response exceeded the client limit."
                  : "The cartridge service could not be reached."
    }

    function _protocolFailure(message) {
        loadState = "error"
        statusText = "Cartridge protocol error."
        errorText = message
    }

    function _validIdentifier(value) {
        return typeof value === "string" && /^[a-z][a-z0-9._-]{0,95}$/.test(value)
    }

    function _validDigest(value) {
        return typeof value === "string" && /^[0-9a-f]{64}$/.test(value)
    }

    function _validText(value, maximum) {
        return typeof value === "string" && value.length >= 1 && value.length <= maximum
                && value.trim() === value && !/[\u0000-\u001f\u007f-\u009f]/.test(value)
    }
}
