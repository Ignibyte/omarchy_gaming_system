import QtQuick

QtObject {
    id: root

    required property var sessionController
    property var actor: null
    property string helperEndpoint: ""
    property string helperCredential: ""
    property bool marketplaceTrusted: false
    property string statusText: ""
    property string errorText: ""
    property string loadState: "idle"
    property var catalog: []
    property var connections: []
    property var challenges: []
    property var nextChallengeBefore: null
    property var sessions: []
    property var selectedSession: null
    property var cartridgeRenderPlan: null
    property string cartridgeAssetRoot: ""
    property string cartridgeRenderState: "idle"
    property string cartridgeScreenId: ""
    property string cartridgeEntryScreenId: ""
    property var cartridgeNavigation: []
    property var cartridgeHistory: []
    readonly property bool cartridgeInstallAvailable: cartridgeRenderState === "missing"
                                                        && _sessionAcquisitionSupported()
    readonly property bool cartridgeCanGoBack: cartridgeRenderState === "ready"
                                                && cartridgeHistory.length > 0
    readonly property bool cartridgeCanGoEntry: cartridgeRenderState === "ready"
                                                 && cartridgeEntryScreenId !== ""
                                                 && cartridgeScreenId !== cartridgeEntryScreenId
    property var presentation: ({
        "supported": false,
        "title": "",
        "turn_label": "",
        "actor_label": "",
        "opponent_label": "",
        "actor_core": 0,
        "actor_energy": 0,
        "actor_guard": 0,
        "opponent_core": 0,
        "opponent_energy": 0,
        "opponent_guard": 0,
        "can_act": false,
        "can_strike": false,
        "can_guard": false,
        "can_charge": false,
        "status": ""
    })
    readonly property bool busy: _expectedGeneration !== 0 || _helperGeneration !== 0
    readonly property bool hasRetryableMutation: _pendingMutation !== null

    property int _expectedGeneration: 0
    property string _expectedOperation: ""
    property bool _appendChallenges: false
    property var _pendingMutation: null
    property int _helperGeneration: 0
    property string _helperOperation: ""
    property string _cartridgeScope: ""
    property string _requestedCartridgeScreen: ""
    property bool _entryFallbackAttempted: false
    property bool _legacyEntryRender: false

    property ApiClient _helperApi: ApiClient {
        id: helperApi
        maximumResponseBytes: 2 * 1024 * 1024
        onFinished: function(generation, operation, status, body, transportError) {
            root._handleHelperFinished(generation, operation, status, body, transportError)
        }
    }

    property Connections _requestConnection: Connections {
        target: root.sessionController
        function onPlayerRequestFinished(generation, operation, status, body, transportError) {
            root._handleFinished(generation, operation, status, body, transportError)
        }
    }

    onActorChanged: reset()
    onHelperEndpointChanged: reset()
    onHelperCredentialChanged: reset()
    onMarketplaceTrustedChanged: reset()

    function reset() {
        if (_expectedGeneration !== 0)
            sessionController.cancelPlayerRequest()
        helperApi.cancel()
        _expectedGeneration = 0
        _expectedOperation = ""
        _appendChallenges = false
        _pendingMutation = null
        _helperGeneration = 0
        _helperOperation = ""
        statusText = ""
        errorText = ""
        loadState = "idle"
        catalog = []
        connections = []
        challenges = []
        nextChallengeBefore = null
        sessions = []
        _clearSelectedSession()
    }

    function refreshGames() {
        if (!_ready())
            return false
        errorText = ""
        statusText = "Loading game cartridges..."
        loadState = "loading"
        return _request("player_games_catalog", "GET", "/v1/games", null, false)
    }

    function refreshChallenges() {
        if (!_ready())
            return false
        _appendChallenges = false
        errorText = ""
        statusText = "Loading challenge cartridges..."
        loadState = "loading"
        return _request("player_challenges_catalog", "GET", "/v1/games", null, false)
    }

    function loadOlderChallenges() {
        if (!_ready() || nextChallengeBefore === null || !_validUuid(nextChallengeBefore))
            return false
        _appendChallenges = true
        errorText = ""
        statusText = "Loading older challenges..."
        return _request("player_challenges_list", "GET", _actorPath()
                        + "/game-challenges?limit=100&before=" + nextChallengeBefore)
    }

    function startSolo(game) {
        if (!_ready() || !_validManifest(game)
                || game.min_human_players !== 1 || game.max_human_players !== 1)
            return false
        return _startMutation("player_games_start", "POST", _actorPath() + "/game-sessions", {
            "idempotency_key": _newUuid(),
            "game_key": game.key,
            "game_version": game.version
        }, "Starting " + game.display_name + "...")
    }

    function createChallenge(connection, game) {
        if (!_ready() || !_validConnection(connection) || !_validManifest(game)
                || game.authority !== "platform_compiled"
                || game.min_human_players !== 2 || game.max_human_players !== 2)
            return false
        return _startMutation("player_challenges_create", "POST",
                              _actorPath() + "/game-challenges", {
            "idempotency_key": _newUuid(),
            "challenged_persona_id": connection.persona.id,
            "game_key": game.key,
            "game_version": game.version
        }, "Sending game challenge...")
    }

    function acceptChallenge(challenge) {
        return _challengeMutation("player_challenges_accept", "PUT", challenge,
                                  "accept", "Accepting challenge...")
    }

    function declineChallenge(challenge) {
        return _challengeMutation("player_challenges_decline", "PUT", challenge,
                                  "decline", "Declining challenge...")
    }

    function cancelChallenge(challenge) {
        if (!_ready() || !_validChallenge(challenge) || challenge.status !== "pending"
                || challenge.direction !== "outgoing")
            return false
        return _startMutation("player_challenges_cancel", "DELETE",
                              _actorPath() + "/game-challenges/" + challenge.id,
                              null, "Canceling challenge...")
    }

    function openChallengeSession(challenge) {
        if (!_validChallenge(challenge) || challenge.status !== "accepted"
                || !_validUuid(challenge.game_session_id))
            return false
        sessionController.showPlayerScreen("gameplay")
        return openSessionById(challenge.game_session_id)
    }

    function openSession(session) {
        if (!_validSession(session))
            return false
        sessionController.showPlayerScreen("gameplay")
        return openSessionById(session.id)
    }

    function openSessionById(sessionId) {
        if (!_ready() || !_validUuid(sessionId))
            return false
        errorText = ""
        statusText = "Loading authoritative game state..."
        loadState = "loading"
        return _request("player_game_detail", "GET", _actorPath()
                        + "/game-sessions/" + sessionId)
    }

    function closeSession() {
        if (_expectedGeneration !== 0)
            sessionController.cancelPlayerRequest()
        if (_helperGeneration !== 0)
            helperApi.cancel()
        _expectedGeneration = 0
        _expectedOperation = ""
        _helperGeneration = 0
        _helperOperation = ""
        _pendingMutation = null
        _clearSelectedSession()
        errorText = ""
        loadState = "idle"
        return true
    }

    function submitAction(action) {
        const selectedAction = String(action)
        if (!_ready() || !_validSession(selectedSession) || !presentation.supported
                || !presentation.can_act
                || ["strike", "guard", "charge"].indexOf(selectedAction) === -1)
            return false
        if ((selectedAction === "strike" && !presentation.can_strike)
                || (selectedAction === "guard" && !presentation.can_guard))
            return false
        return _startMutation("player_game_command", "POST", _actorPath()
                              + "/game-sessions/" + selectedSession.id + "/commands", {
            "idempotency_key": _newUuid(),
            "expected_revision": selectedSession.revision,
            "command": {"kind": "play", "action": selectedAction}
        }, "Sending " + selectedAction.toUpperCase() + " command...")
    }

    function activateCartridgeAction(action, payload) {
        const selectedAction = String(action)
        if (selectedAction.startsWith("navigate."))
            return navigateCartridge(selectedAction, payload)
        return submitCartridgeAction(selectedAction, payload)
    }

    function navigateCartridge(action, payload) {
        if (busy || cartridgeRenderState !== "ready" || cartridgeScreenId === ""
                || !payload || typeof payload !== "object" || Array.isArray(payload)
                || Object.keys(payload).length !== 0)
            return false
        let target = ""
        for (let index = 0; index < cartridgeNavigation.length; index++) {
            if (cartridgeNavigation[index].action === action) {
                target = cartridgeNavigation[index].target_screen
                break
            }
        }
        if (target === "")
            return false
        let nextHistory = cartridgeHistory.concat([cartridgeScreenId])
        if (nextHistory.length > 16)
            nextHistory = nextHistory.slice(nextHistory.length - 16)
        if (!_requestCartridgeRender(target, false))
            return false
        cartridgeHistory = nextHistory
        return true
    }

    function backCartridgeScreen() {
        if (!cartridgeCanGoBack || busy)
            return false
        const target = cartridgeHistory[cartridgeHistory.length - 1]
        if (!_requestCartridgeRender(target, false))
            return false
        cartridgeHistory = cartridgeHistory.slice(0, cartridgeHistory.length - 1)
        return true
    }

    function enterCartridgeScreen() {
        if (!cartridgeCanGoEntry || busy)
            return false
        if (!_requestCartridgeRender(cartridgeEntryScreenId, false))
            return false
        cartridgeHistory = []
        return true
    }

    function installPinnedCartridge() {
        const session = selectedSession
        const binding = session === null ? null : session.presentation
        const authority = sessionController.trustedCartridgeAuthority()
        if (busy || cartridgeRenderState !== "missing" || authority === null
                || !_validSession(session) || !_validSessionPresentation(binding, session)
                || !_sessionAcquisitionSupported() || helperEndpoint === ""
                || helperCredential === "" || !marketplaceTrusted)
            return false
        const configured = helperApi.configure(helperEndpoint)
        if (!configured.ok)
            return false
        helperApi.installBearer(helperCredential)
        statusText = "Installing the session's exact signed cartridge..."
        loadState = "loading"
        cartridgeRenderState = "installing"
        _helperOperation = "cartridge_session_install"
        _helperGeneration = helperApi.request(_helperOperation, "POST", "/v1/session-acquisitions", {
            "server_origin": authority.origin,
            "server_id": authority.server_id,
            "device_bearer": authority.device_bearer,
            "persona_id": actor.id,
            "game_session_id": session.id
        }, true)
        if (_helperGeneration === 0) {
            _helperOperation = ""
            cartridgeRenderState = "missing"
            loadState = "ready"
            return false
        }
        return true
    }

    function submitCartridgeAction(action, payload) {
        const selectedAction = String(action)
        const session = selectedSession
        const binding = session === null ? null : session.presentation
        if (!_ready() || !_validSession(session) || !_validSessionPresentation(binding, session)
                || cartridgeRenderPlan === null || cartridgeRenderState !== "ready"
                || session.status !== "active" || binding.active_session_policy !== "continue"
                || !/^[a-z][a-z0-9._-]{0,95}$/.test(selectedAction)
                || selectedAction.startsWith("navigate.")
                || (cartridgeScreenId === "" && !_legacyEntryRender)
                || !payload || typeof payload !== "object" || Array.isArray(payload))
            return false
        const document = {
            "idempotency_key": _newUuid(),
            "expected_revision": session.revision,
            "archive_sha256": binding.archive_sha256,
            "action": selectedAction,
            "payload": payload
        }
        if (!_legacyEntryRender)
            document.screen_id = cartridgeScreenId
        if (JSON.stringify(document).length > 32768)
            return false
        return _startMutation("player_cartridge_action", "POST", _actorPath()
                              + "/game-sessions/" + session.id + "/cartridge-actions",
                              document, "Sending signed cartridge action...")
    }

    function retryPendingMutation() {
        if (!_ready() || _pendingMutation === null)
            return false
        const pending = _pendingMutation
        errorText = ""
        statusText = "Retrying the same operation..."
        return _request(pending.operation, pending.method, pending.path, pending.document)
    }

    function challengeGames() {
        return catalog.filter(function(game) {
            return game.authority === "platform_compiled"
                    && game.min_human_players === 2 && game.max_human_players === 2
        })
    }

    function soloGames() {
        return catalog.filter(function(game) {
            return game.min_human_players === 1 && game.max_human_players === 1
        })
    }

    function gameName(gameKey, gameVersion) {
        for (let index = 0; index < catalog.length; index++) {
            const game = catalog[index]
            if (game.key === gameKey && game.version === gameVersion)
                return game.display_name
        }
        return gameKey + " v" + gameVersion
    }

    function otherChallengePersona(challenge) {
        if (!_validChallenge(challenge))
            return null
        return challenge.direction === "incoming" ? challenge.challenger : challenge.challenged
    }

    function _challengeMutation(operation, method, challenge, transition, message) {
        if (!_ready() || !_validChallenge(challenge) || challenge.status !== "pending"
                || challenge.direction !== "incoming")
            return false
        return _startMutation(operation, method, _actorPath() + "/game-challenges/"
                              + challenge.id + "/" + transition, null, message)
    }

    function _startMutation(operation, method, path, document, message) {
        if (busy)
            return false
        _pendingMutation = {
            "operation": operation,
            "method": method,
            "path": path,
            "document": document
        }
        errorText = ""
        statusText = message
        return _request(operation, method, path, document)
    }

    function _request(operation, method, path, document, authenticated) {
        if (busy)
            return false
        const generation = sessionController.playerRequest(
                    operation, method, path, document, authenticated)
        if (generation === 0) {
            errorText = "The player request could not start. Try again."
            return false
        }
        _expectedGeneration = generation
        _expectedOperation = operation
        loadState = "loading"
        return true
    }

    function _handleFinished(generation, operation, status, body, transportError) {
        if (generation !== _expectedGeneration || operation !== _expectedOperation)
            return
        _expectedGeneration = 0
        _expectedOperation = ""
        if (transportError !== "") {
            loadState = "error"
            errorText = transportError === "timeout" ? "The request timed out."
                      : transportError === "response_too_large" ? "The response exceeded the client limit."
                      : "The server could not complete the request."
            statusText = _pendingMutation === null
                    ? "Request not completed." : "Outcome unknown; retry uses the same operation ID."
            return
        }

        const parsed = _parseDocument(body)
        if (!parsed.ok) {
            _pendingMutation = null
            _protocolFailure()
            return
        }
        const document = parsed.document
        if (_invalidSession(status, document)) {
            reset()
            sessionController.invalidatePlayerSession("This device session is no longer valid.")
            return
        }
        if (status < 200 || status >= 300) {
            _pendingMutation = null
            _actionFailure(operation, document)
            return
        }

        _pendingMutation = null
        if (operation === "player_games_catalog" || operation === "player_challenges_catalog") {
            if (!_validCatalog(document)) { _protocolFailure(); return }
            catalog = document.games
            if (operation === "player_games_catalog") {
                statusText = "Loading game sessions..."
                _request("player_games_sessions", "GET", _actorPath() + "/game-sessions?limit=100")
            } else {
                statusText = "Loading accepted connections..."
                _request("player_challenges_connections", "GET", _actorPath() + "/connections")
            }
        } else if (operation === "player_games_sessions") {
            if (!_exactKeys(document, ["sessions"])
                    || !_validBoundedArray(document.sessions, _validSession)) { _protocolFailure(); return }
            sessions = document.sessions
            loadState = "ready"
            statusText = sessions.length === 0 ? "No game sessions yet." : "Game sessions are current."
            errorText = ""
        } else if (operation === "player_challenges_connections") {
            if (!_exactKeys(document, ["connections"])
                    || !_validBoundedArray(document.connections, _validConnection)) { _protocolFailure(); return }
            connections = document.connections
            statusText = "Loading game challenges..."
            _request("player_challenges_list", "GET", _actorPath() + "/game-challenges?limit=100")
        } else if (operation === "player_challenges_list") {
            if (!_validChallengePage(document)) { _protocolFailure(); return }
            if (_appendChallenges) {
                const existingIds = ({})
                for (let existingIndex = 0; existingIndex < challenges.length; existingIndex++)
                    existingIds[challenges[existingIndex].id] = true
                for (let pageIndex = 0; pageIndex < document.challenges.length; pageIndex++) {
                    if (existingIds[document.challenges[pageIndex].id]) {
                        _protocolFailure()
                        return
                    }
                }
                if (challenges.length > 0 && document.challenges.length > 0
                        && Date.parse(challenges[challenges.length - 1].created_at)
                           < Date.parse(document.challenges[0].created_at)) {
                    _protocolFailure()
                    return
                }
            }
            challenges = _appendChallenges ? challenges.concat(document.challenges) : document.challenges
            nextChallengeBefore = document.next_before
            _appendChallenges = false
            loadState = "ready"
            statusText = challenges.length === 0 ? "No game challenges yet." : "Game challenges are current."
            errorText = ""
        } else if (operation === "player_games_start") {
            if ((status !== 200 && status !== 201 && status !== 202)
                    || !_validSession(document)) { _protocolFailure(); return }
            selectedSession = document
            _derivePresentation()
            sessionController.showPlayerScreen("gameplay")
            if (!_requestCartridgeRender()) {
                loadState = "ready"
                statusText = status === 201 ? "Game cartridge started."
                           : status === 202 ? "Game provider is provisioning the session."
                           : "Existing start recovered."
            }
        } else if (operation.startsWith("player_challenges_")
                   && ["player_challenges_create", "player_challenges_accept",
                       "player_challenges_decline", "player_challenges_cancel"].indexOf(operation) !== -1) {
            if (!_validChallenge(document)) { _protocolFailure(); return }
            if (operation === "player_challenges_accept" && _validUuid(document.game_session_id)) {
                sessionController.showPlayerScreen("gameplay")
                openSessionById(document.game_session_id)
            } else {
                statusText = "Challenge state updated."
                Qt.callLater(function() { root.refreshChallenges() })
            }
        } else if (operation === "player_game_detail") {
            if (!_validSession(document)) { _protocolFailure(); return }
            selectedSession = document
            _derivePresentation()
            if (!_requestCartridgeRender()) {
                loadState = "ready"
                statusText = document.status === "completed"
                        ? "Final authoritative result loaded." : "Authoritative turn state loaded."
                errorText = ""
            }
        } else if (operation === "player_game_command"
                   || operation === "player_cartridge_action") {
            if (operation === "player_cartridge_action") {
                if (!_validCartridgeActionResponse(document, selectedSession)) {
                    _protocolFailure(); return
                }
            } else if (!_validCommandResponse(document, selectedSession)) {
                _protocolFailure(); return
            }
            selectedSession = Object.assign({}, selectedSession, {
                "revision": document.revision,
                "status": document.status,
                "state": document.state,
                "authority": document.authority,
                "provider_release_id": document.provider_release_id,
                "availability": document.availability
            })
            _derivePresentation()
            statusText = "Command committed; confirming session..."
            openSessionById(document.game_session_id)
        }
    }

    function _requestCartridgeRender(requestedScreen, entryFallback) {
        cartridgeRenderPlan = null
        cartridgeAssetRoot = ""
        cartridgeRenderState = "idle"
        cartridgeNavigation = []
        const session = selectedSession
        if (!_validSession(session) || session.presentation === null)
            return false
        const binding = session.presentation
        const scope = session.id + ":" + binding.archive_sha256 + ":"
                + binding.admission_revision
        if (_cartridgeScope !== scope) {
            _cartridgeScope = scope
            cartridgeScreenId = ""
            cartridgeEntryScreenId = ""
            cartridgeHistory = []
            _legacyEntryRender = false
        }
        if (binding.active_session_policy !== "continue") {
            cartridgeRenderState = binding.active_session_policy
            return false
        }
        const authority = sessionController.trustedCartridgeAuthority()
        if (authority === null || helperEndpoint === "" || helperCredential === ""
                || !marketplaceTrusted || session.state === null) {
            cartridgeRenderState = "unavailable"
            return false
        }
        const configured = helperApi.configure(helperEndpoint)
        if (!configured.ok) {
            cartridgeRenderState = "error"
            return false
        }
        helperApi.installBearer(helperCredential)
        let screen = typeof requestedScreen === "string" ? requestedScreen : cartridgeScreenId
        if (screen !== "" && !/^[a-z][a-z0-9._-]{0,95}$/.test(screen))
            screen = ""
        statusText = "Compiling trusted cartridge presentation..."
        loadState = "loading"
        cartridgeRenderState = "loading"
        _requestedCartridgeScreen = screen
        _entryFallbackAttempted = entryFallback === true
        const request = {
            "server_origin": authority.origin,
            "server_id": authority.server_id,
            "game_key": binding.game_key,
            "archive_sha256": binding.archive_sha256,
            "admission_revision": binding.admission_revision,
            "lifecycle_status": binding.lifecycle_status,
            "active_session_policy": binding.active_session_policy,
            "view": session.state,
            "preferences": {
                "scale": 1.0,
                "high_contrast": false,
                "reduced_motion": false,
                "muted_audio": false
            }
        }
        if (screen !== "")
            request.screen_id = screen
        _helperOperation = "cartridge_render"
        _helperGeneration = helperApi.request(_helperOperation, "POST", "/v1/render-plans",
                                              request, true)
        if (_helperGeneration === 0) {
            _helperOperation = ""
            cartridgeRenderState = "error"
            return false
        }
        return true
    }

    function _handleHelperFinished(generation, operation, status, body, transportError) {
        if (generation !== _helperGeneration || operation !== _helperOperation)
            return
        _helperGeneration = 0
        _helperOperation = ""
        if (transportError !== "") {
            cartridgeRenderState = operation === "cartridge_session_install" ? "missing" : "error"
            loadState = "ready"
            statusText = operation === "cartridge_session_install"
                    ? "The pinned cartridge was not installed; authoritative state is unchanged."
                    : "Authoritative state loaded; trusted cartridge rendering is unavailable."
            errorText = transportError === "timeout" ? "The cartridge companion timed out."
                      : "The cartridge companion could not complete the request."
            return
        }
        const parsed = _parseDocument(body)
        if (operation === "cartridge_session_install") {
            if (status !== 200 || !parsed.ok
                    || !_validSessionMountResponse(parsed.document)) {
                cartridgeRenderState = "missing"
                loadState = "ready"
                statusText = "The pinned cartridge was not installed; authoritative state is unchanged."
                errorText = parsed.ok ? _helperError(_errorCode(parsed.document))
                                      : "The companion response was not accepted."
                return
            }
            statusText = "Exact session cartridge installed; compiling its signed entry screen..."
            errorText = ""
            if (!_requestCartridgeRender("", false)) {
                cartridgeRenderState = "error"
                loadState = "ready"
            }
            return
        }
        if (status === 404 && parsed.ok
                && _errorCode(parsed.document) === "companion_mount_missing") {
            cartridgeRenderState = "missing"
            loadState = "ready"
            statusText = "Authoritative state loaded; this session's exact cartridge is not installed."
            errorText = ""
            return
        }
        if (status !== 200 || !parsed.ok || !_validRenderResponse(parsed.document)) {
            if (_requestedCartridgeScreen !== "" && !_entryFallbackAttempted
                    && _requestCartridgeRender("", true)) {
                cartridgeHistory = []
                statusText = "That signed screen is no longer available; returning to entry..."
                return
            }
            cartridgeRenderState = "error"
            loadState = "ready"
            statusText = "Authoritative state loaded; the cartridge render plan was rejected."
            errorText = "The exact signed screen could not be compiled."
            return
        }
        cartridgeRenderPlan = parsed.document.plan
        cartridgeAssetRoot = parsed.document.asset_base_url
        if (parsed.document.format === "omarchygs.session-cartridge-render/v2") {
            cartridgeScreenId = parsed.document.screen_id
            cartridgeEntryScreenId = parsed.document.entry_screen_id
            cartridgeNavigation = parsed.document.navigation
            _legacyEntryRender = false
        } else {
            cartridgeScreenId = ""
            cartridgeEntryScreenId = ""
            cartridgeNavigation = []
            cartridgeHistory = []
            _legacyEntryRender = true
        }
        cartridgeRenderState = "ready"
        loadState = "ready"
        statusText = "Trusted cartridge presentation ready."
        errorText = ""
    }

    function _validRenderResponse(document) {
        if (!document || typeof document !== "object")
            return false
        const legacy = _exactKeys(document, ["plan", "asset_base_url"])
        const current = _exactKeys(document, ["format", "screen_id", "entry_screen_id",
                                               "navigation", "plan", "asset_base_url"])
        let navigationActions = null
        if (!legacy && !current)
            return false
        if (!document.plan || typeof document.plan !== "object"
                || typeof document.asset_base_url !== "string")
            return false
        if (current) {
            if (document.format !== "omarchygs.session-cartridge-render/v2"
                    || !/^[a-z][a-z0-9._-]{0,95}$/.test(document.screen_id)
                    || !/^[a-z][a-z0-9._-]{0,95}$/.test(document.entry_screen_id)
                    || (_requestedCartridgeScreen !== ""
                        && document.screen_id !== _requestedCartridgeScreen)
                    || (_requestedCartridgeScreen === ""
                        && document.screen_id !== document.entry_screen_id)
                    || !Array.isArray(document.navigation)
                    || document.navigation.length > 512)
                return false
            navigationActions = ({})
            for (let index = 0; index < document.navigation.length; index++) {
                const navigation = document.navigation[index]
                if (!_exactKeys(navigation, ["action", "target_screen"])
                        || !/^[a-z][a-z0-9._-]{0,95}$/.test(navigation.target_screen)
                        || navigation.action !== "navigate." + navigation.target_screen
                        || navigationActions[navigation.action])
                    return false
                navigationActions[navigation.action] = true
            }
        }
        const binding = selectedSession.presentation
        const origin = document.plan.origin
        if (!origin || typeof origin !== "object"
                || document.plan.format !== "omarchygs.render-plan/v1"
                || document.plan.state !== "ready"
                || origin.publisher_id !== binding.publisher_id
                || origin.game_key !== binding.game_key
                || origin.cartridge_version !== binding.cartridge_version
                || origin.archive_sha256 !== binding.archive_sha256)
            return false
        if (current) {
            if (!Array.isArray(document.plan.nodes))
                return false
            const emitted = ({})
            for (let nodeIndex = 0; nodeIndex < document.plan.nodes.length; nodeIndex++) {
                const node = document.plan.nodes[nodeIndex]
                if (node && typeof node.action === "string"
                        && node.action.startsWith("navigate.")) {
                    if (node.kind !== "button" || emitted[node.action]
                            || !Object.prototype.hasOwnProperty.call(navigationActions, node.action))
                        return false
                    emitted[node.action] = true
                }
            }
            const actionNames = Object.keys(navigationActions)
            for (let actionIndex = 0; actionIndex < actionNames.length; actionIndex++) {
                if (!emitted[actionNames[actionIndex]])
                    return false
            }
        }
        const configured = helperApi.baseUrl + "/v1/render-assets/"
        const capability = document.asset_base_url.startsWith(configured)
                ? document.asset_base_url.slice(configured.length) : ""
        return /^[A-Za-z0-9_-]{43}$/.test(capability)
    }

    function _sessionAcquisitionSupported() {
        const authority = sessionController.trustedCartridgeAuthority()
        return authority !== null && authority.session_acquisition_supported === true
                && helperEndpoint !== "" && helperCredential !== "" && marketplaceTrusted
    }

    function _validSessionMountResponse(document) {
        if (!_exactKeys(document, ["mount"]) || !document.mount
                || typeof document.mount !== "object")
            return false
        const mount = document.mount
        const binding = selectedSession === null ? null : selectedSession.presentation
        const authority = sessionController.trustedCartridgeAuthority()
        if (binding === null || authority === null)
            return false
        const keys = ["format", "server_id", "server_origin", "game_key", "publisher_id",
                      "rules_version", "cartridge_version", "display_name", "archive_sha256",
                      "signed_identity_sha256", "marketplace_key_sha256", "marketplace_id",
                      "marketplace_name", "reviewed_by", "review_summary", "snapshot_version",
                      "policy_version", "lifecycle_status", "admission_revision"]
        if (mount.warning !== undefined)
            keys.push("warning")
        const warningMatches = mount.lifecycle_status === "deprecated"
                ? _boundedString(mount.warning, 512, 1) : mount.warning === undefined
        return _exactKeys(mount, keys)
                && mount.format === "omarchygs.client-cartridge-mount/v1"
                && mount.server_id === authority.server_id
                && mount.server_origin === authority.origin
                && mount.game_key === binding.game_key
                && mount.publisher_id === binding.publisher_id
                && mount.rules_version === binding.rules_version
                && mount.cartridge_version === binding.cartridge_version
                && _boundedString(mount.display_name, 128, 1)
                && mount.archive_sha256 === binding.archive_sha256
                && mount.signed_identity_sha256 === binding.signed_identity_sha256
                && /^[0-9a-f]{64}$/.test(mount.marketplace_key_sha256)
                && /^[a-z][a-z0-9._-]{0,95}$/.test(mount.marketplace_id)
                && _boundedString(mount.marketplace_name, 128, 1)
                && /^[a-z][a-z0-9._-]{0,95}$/.test(mount.reviewed_by)
                && _boundedString(mount.review_summary, 512, 1)
                && Number.isSafeInteger(mount.snapshot_version) && mount.snapshot_version > 0
                && Number.isSafeInteger(mount.policy_version) && mount.policy_version > 0
                && ["active", "deprecated"].indexOf(mount.lifecycle_status) !== -1
                && mount.admission_revision === binding.admission_revision
                && warningMatches
    }

    function _helperError(code) {
        const messages = {
            "companion_admission_changed": "The session pin changed before installation completed. Refresh and retry.",
            "companion_mount_missing": "The session's exact cartridge is not installed.",
            "companion_server_unavailable": "The selected server could not complete the cartridge download.",
            "companion_server_rejected": "The signed historical cartridge evidence was rejected.",
            "companion_marketplace_untrusted": "Configure an independently trusted marketplace key before installing.",
            "companion_cache_failure": "The private cartridge cache needs attention."
        }
        return messages[code] || "The local cartridge companion rejected the operation."
    }

    function _actionFailure(operation, document) {
        const code = _errorCode(document)
        if (code === "") { _protocolFailure(); return }
        if ((operation === "player_game_command" || operation === "player_cartridge_action")
                && code === "game_revision_conflict"
                && selectedSession !== null) {
            statusText = "Turn changed; refreshing authoritative state..."
            openSessionById(selectedSession.id)
            return
        }
        if (operation === "player_cartridge_action"
                && code === "session_cartridge_unavailable") {
            cartridgeRenderPlan = null
            cartridgeAssetRoot = ""
            cartridgeRenderState = "denied"
        }
        const messages = {
            "persona_not_found": "The selected persona is unavailable.",
            "game_session_not_found": "That game session is unavailable.",
            "game_unavailable": "That exact game cartridge is unavailable.",
            "session_cartridge_unavailable": "That signed session cartridge is unavailable or no longer allowed.",
            "invalid_game_participants": "That game does not support this player count.",
            "too_many_active_game_sessions": "Finish an active solo game before starting another.",
            "challenge_target_unavailable": "That connection cannot receive this challenge.",
            "game_challenge_not_found": "That challenge is unavailable.",
            "game_challenge_expired": "That challenge expired.",
            "game_challenge_transition_unavailable": "That challenge was already resolved.",
            "duplicate_pending_game_challenge": "That exact challenge is already pending.",
            "game_challenge_limit_reached": "The pending challenge limit was reached.",
            "game_command_rejected": "That action is not legal for the current turn.",
            "invalid_game_command": "The game command was not accepted.",
            "game_idempotency_conflict": "The operation identity conflicted with an earlier command.",
            "game_challenge_idempotency_conflict": "The operation identity conflicted with an earlier challenge.",
            "invalid_pagination": "That page is unavailable.",
            "internal_error": "The server could not complete the request."
        }
        errorText = messages[code] || "The server rejected the request."
        statusText = "Request not accepted."
        loadState = "error"
    }

    function _protocolFailure() {
        _appendChallenges = false
        loadState = "error"
        statusText = "The server response was not accepted."
        errorText = "No game authority was changed; retry after checking the server."
    }

    function _derivePresentation() {
        const session = selectedSession
        if (!_validSession(session) || session.authority !== "platform_compiled"
                || session.game_key !== "signal_siege") {
            presentation = Object.assign({}, presentation, {
                "supported": false, "can_act": false, "can_strike": false,
                "can_guard": false, "can_charge": false
            })
            return
        }
        let actorSeat = -1
        for (let index = 0; index < session.participants.length; index++) {
            if (session.participants[index].persona.id === actor.id)
                actorSeat = session.participants[index].seat
        }
        if (actorSeat < 0) {
            presentation = Object.assign({}, presentation, {
                "supported": false, "can_act": false, "can_strike": false,
                "can_guard": false, "can_charge": false
            })
            return
        }
        if (session.game_version === 1) {
            const state = session.state
            const active = session.status === "active" && state.phase === "awaiting_human"
            presentation = {
                "supported": true,
                "title": "SIGNAL SIEGE // SOLO",
                "turn_label": "ROUND " + state.round + " / " + state.max_rounds,
                "actor_label": actor.display_name,
                "opponent_label": "SIEGE BOT",
                "actor_core": state.human.core,
                "actor_energy": state.human.energy,
                "actor_guard": 0,
                "opponent_core": state.bot.core,
                "opponent_energy": state.bot.energy,
                "opponent_guard": 0,
                "can_act": active,
                "can_strike": active && state.human.energy > 0,
                "can_guard": active && state.human.energy > 0,
                "can_charge": active,
                "status": _gameplayStatus(session, actorSeat)
            }
            return
        }
        if (session.game_version === 2) {
            const versus = session.state
            const opponentSeat = 1 - actorSeat
            const active = session.status === "active" && versus.active_seat === actorSeat
            presentation = {
                "supported": true,
                "title": "SIGNAL SIEGE // VERSUS",
                "turn_label": "TURN " + versus.turn + " / " + versus.max_turns,
                "actor_label": session.participants[actorSeat].persona.display_name,
                "opponent_label": session.participants[opponentSeat].persona.display_name,
                "actor_core": versus.players[actorSeat].core,
                "actor_energy": versus.players[actorSeat].energy,
                "actor_guard": versus.players[actorSeat].guard,
                "opponent_core": versus.players[opponentSeat].core,
                "opponent_energy": versus.players[opponentSeat].energy,
                "opponent_guard": versus.players[opponentSeat].guard,
                "can_act": active,
                "can_strike": active && versus.players[actorSeat].energy > 0,
                "can_guard": active && versus.players[actorSeat].energy > 0,
                "can_charge": active,
                "status": _gameplayStatus(session, actorSeat)
            }
            return
        }
        presentation = Object.assign({}, presentation, {
            "supported": false, "can_act": false, "can_strike": false,
            "can_guard": false, "can_charge": false
        })
    }

    function _gameplayStatus(session, actorSeat) {
        if (session.status === "completed") {
            const outcome = session.state.outcome
            if (session.game_version === 1)
                return outcome.winner === "human" ? "VICTORY // " + outcome.reason.toUpperCase()
                     : outcome.winner === "bot" ? "DEFEAT // " + outcome.reason.toUpperCase()
                     : "DRAW // " + outcome.reason.toUpperCase()
            const actorWinner = actorSeat === 0 ? "seat_0" : "seat_1"
            return outcome.winner === actorWinner ? "VICTORY // " + outcome.reason.toUpperCase()
                 : outcome.winner === "draw" ? "DRAW // " + outcome.reason.toUpperCase()
                 : "DEFEAT // " + outcome.reason.toUpperCase()
        }
        if (session.game_version === 1)
            return "YOUR COMMAND"
        return session.state.active_seat === actorSeat ? "YOUR TURN" : "WAITING FOR OPPONENT"
    }

    function _clearSelectedSession() {
        selectedSession = null
        cartridgeRenderPlan = null
        cartridgeAssetRoot = ""
        cartridgeRenderState = "idle"
        cartridgeScreenId = ""
        cartridgeEntryScreenId = ""
        cartridgeNavigation = []
        cartridgeHistory = []
        _cartridgeScope = ""
        _requestedCartridgeScreen = ""
        _entryFallbackAttempted = false
        _legacyEntryRender = false
        presentation = {
            "supported": false,
            "title": "",
            "turn_label": "",
            "actor_label": "",
            "opponent_label": "",
            "actor_core": 0,
            "actor_energy": 0,
            "actor_guard": 0,
            "opponent_core": 0,
            "opponent_energy": 0,
            "opponent_guard": 0,
            "can_act": false,
            "can_strike": false,
            "can_guard": false,
            "can_charge": false,
            "status": ""
        }
    }

    function _validCatalog(value) {
        if (!_exactKeys(value, ["games"]) || !_validBoundedArray(value.games, _validManifest))
            return false
        for (let index = 1; index < value.games.length; index++) {
            const previous = value.games[index - 1]
            const current = value.games[index]
            if (previous.key > current.key
                    || (previous.key === current.key && previous.version >= current.version))
                return false
        }
        return true
    }

    function _validManifest(value) {
        return _exactKeys(value, ["key", "version", "display_name", "min_human_players",
                                  "max_human_players", "authority", "provider_release_id"])
                && _boundedString(value.key, 32, 3) && /^[a-z0-9][a-z0-9_-]*$/.test(value.key)
                && Number.isSafeInteger(value.version) && value.version > 0
                && _boundedString(value.display_name, 64, 1)
                && Number.isSafeInteger(value.min_human_players) && value.min_human_players > 0
                && Number.isSafeInteger(value.max_human_players)
                && value.max_human_players >= value.min_human_players
                && value.max_human_players <= 8
                && (value.authority === "platform_compiled" || value.authority === "registered_provider")
                && (value.provider_release_id === null || _validUuid(value.provider_release_id))
                && (value.authority === "registered_provider") === (value.provider_release_id !== null)
    }

    function _validChallengePage(value) {
        if (!_exactKeys(value, ["challenges", "next_before"])
                || !_validBoundedArray(value.challenges, _validChallenge)
                || !(value.next_before === null || _validUuid(value.next_before)))
            return false
        const seen = ({})
        for (let index = 0; index < value.challenges.length; index++) {
            const challenge = value.challenges[index]
            if (seen[challenge.id])
                return false
            seen[challenge.id] = true
            if (index > 0 && Date.parse(value.challenges[index - 1].created_at)
                    < Date.parse(challenge.created_at))
                return false
        }
        return true
    }

    function _validChallenge(value) {
        if (!_exactKeys(value, ["id", "game_key", "game_version", "direction", "status",
                                "challenger", "challenged", "game_session_id", "expires_at",
                                "resolved_at", "created_at", "updated_at"])
                || !_validUuid(value.id) || !_boundedString(value.game_key, 64, 1)
                || !Number.isSafeInteger(value.game_version) || value.game_version < 1
                || (value.direction !== "incoming" && value.direction !== "outgoing")
                || ["pending", "accepted", "declined", "cancelled", "expired"].indexOf(value.status) === -1
                || !_validPersona(value.challenger) || !_validPersona(value.challenged)
                || !_validTimestamp(value.expires_at) || !_validTimestamp(value.created_at)
                || !_validTimestamp(value.updated_at)
                || !(value.resolved_at === null || _validTimestamp(value.resolved_at)))
            return false
        const accepted = value.status === "accepted"
        return value.challenger.id !== value.challenged.id
                && (accepted ? _validUuid(value.game_session_id) : value.game_session_id === null)
                && (value.status === "pending" ? value.resolved_at === null : value.resolved_at !== null)
                && (value.direction === "incoming"
                    ? value.challenged.id === actor.id && value.challenger.id !== actor.id
                    : value.challenger.id === actor.id && value.challenged.id !== actor.id)
    }

    function _validSession(value) {
        if (!_exactKeys(value, ["id", "game_key", "game_version", "revision", "status", "state",
                                "authority", "provider_release_id", "availability", "result",
                                "presentation", "participants", "completed_at", "created_at", "updated_at"])
                || !_validUuid(value.id) || !_boundedString(value.game_key, 64, 1)
                || !Number.isSafeInteger(value.game_version) || value.game_version < 1
                || !Number.isSafeInteger(value.revision) || value.revision < 0
                || (value.status !== "active" && value.status !== "completed")
                || !Array.isArray(value.participants) || value.participants.length < 1
                || value.participants.length > 8 || !_validTimestamp(value.created_at)
                || !_validTimestamp(value.updated_at)
                || !(value.completed_at === null || _validTimestamp(value.completed_at))
                || (value.status === "completed") !== (value.completed_at !== null))
            return false
        let actorFound = false
        const seenPersonas = ({})
        for (let index = 0; index < value.participants.length; index++) {
            const participant = value.participants[index]
            if (!_exactKeys(participant, ["seat", "persona"]) || participant.seat !== index
                    || !_validPersona(participant.persona))
                return false
            if (seenPersonas[participant.persona.id])
                return false
            seenPersonas[participant.persona.id] = true
            if (participant.persona.id === actor.id)
                actorFound = true
        }
        if (!actorFound)
            return false
        if (!(value.presentation === null
                || _validSessionPresentation(value.presentation, value)))
            return false
        if (value.authority === "platform_compiled") {
            if (value.game_key === "signal_siege"
                    && ((value.game_version === 1 && value.participants.length !== 1)
                        || (value.game_version === 2 && value.participants.length !== 2)))
                return false
            return value.provider_release_id === null && value.availability === null
                    && value.result === null && value.state !== null
                    && _validCompiledState(value.game_key, value.game_version, value.state, value.status)
        }
        return value.authority === "registered_provider" && _validUuid(value.provider_release_id)
                && (value.state === null || (typeof value.state === "object" && !Array.isArray(value.state)))
                && (value.availability === "provisioning" || value.availability === "ready"
                    || value.availability === "reconciling" || value.availability === "unavailable"
                    || value.availability === "suspended" || value.availability === "completed"
                    || value.availability === "retired")
                && (value.result === null || _validProviderResult(value.result))
    }

    function _validSessionPresentation(binding, session) {
        if (!binding || typeof binding !== "object")
            return false
        const keys = ["format", "publisher_id", "game_key", "rules_version",
                      "cartridge_version", "archive_sha256", "signed_identity_sha256",
                      "admission_revision", "lifecycle_status", "active_session_policy"]
        if (binding.warning !== undefined)
            keys.push("warning")
        const policies = {
            "active": "continue", "deprecated": "continue", "suspended": "suspend",
            "revoked": "terminate", "retired": "continue"
        }
        return _exactKeys(binding, keys)
                && binding.format === "omarchygs.session-cartridge/v1"
                && /^[a-z][a-z0-9._-]{0,95}$/.test(binding.publisher_id)
                && binding.game_key === session.game_key
                && binding.rules_version === session.game_version
                && Number.isSafeInteger(binding.cartridge_version)
                && binding.cartridge_version > 0
                && /^[0-9a-f]{64}$/.test(binding.archive_sha256)
                && /^[0-9a-f]{64}$/.test(binding.signed_identity_sha256)
                && Number.isSafeInteger(binding.admission_revision)
                && binding.admission_revision > 0
                && policies[binding.lifecycle_status] === binding.active_session_policy
                && (binding.lifecycle_status === "deprecated"
                    ? _boundedString(binding.warning, 512, 1)
                    : binding.warning === undefined)
    }

    function _validCommandResponse(value, session) {
        return _exactKeys(value, ["game_session_id", "revision", "status", "state", "authority",
                                  "provider_release_id", "availability"])
                && _validSession(session) && value.game_session_id === session.id
                && Number.isSafeInteger(value.revision) && value.revision === session.revision + 1
                && (value.status === "active" || value.status === "completed")
                && value.authority === "platform_compiled" && value.provider_release_id === null
                && value.availability === null
                && _validCompiledState(session.game_key, session.game_version, value.state, value.status)
    }

    function _validCartridgeActionResponse(value, session) {
        if (!_exactKeys(value, ["game_session_id", "revision", "status", "state", "authority",
                                "provider_release_id", "availability", "archive_sha256"])
                || !_validSession(session) || session.presentation === null
                || value.game_session_id !== session.id
                || value.archive_sha256 !== session.presentation.archive_sha256
                || !Number.isSafeInteger(value.revision)
                || value.revision !== session.revision + 1
                || (value.status !== "active" && value.status !== "completed")
                || value.authority !== session.authority
                || !value.state || typeof value.state !== "object" || Array.isArray(value.state))
            return false
        if (value.authority === "platform_compiled")
            return value.provider_release_id === null && value.availability === null
                    && _validCompiledState(session.game_key, session.game_version,
                                           value.state, value.status)
        return _validUuid(value.provider_release_id)
                && (value.availability === "provisioning" || value.availability === "ready"
                    || value.availability === "reconciling" || value.availability === "unavailable"
                    || value.availability === "suspended" || value.availability === "completed"
                    || value.availability === "retired")
    }

    function _validCompiledState(gameKey, gameVersion, state, status) {
        if (gameKey !== "signal_siege")
            return typeof state === "object" && state !== null && !Array.isArray(state)
        if (gameVersion === 1)
            return _validSoloState(state, status)
        if (gameVersion === 2)
            return _validVersusState(state, status)
        return false
    }

    function _validSoloState(state, status) {
        if (!_exactKeys(state, ["schema_version", "rules_version", "round", "max_rounds", "phase",
                                "human", "bot", "last_round", "outcome"])
                || state.schema_version !== 1 || state.rules_version !== 1
                || !Number.isSafeInteger(state.round) || state.round < 0 || state.round > 12
                || state.max_rounds !== 12 || !_validCombatant(state.human)
                || !_validCombatant(state.bot))
            return false
        if (state.last_round !== null
                && (!_exactKeys(state.last_round, ["round", "human_action", "bot_action",
                                                   "damage_to_human", "damage_to_bot"])
                    || state.last_round.round !== state.round
                    || !_validAction(state.last_round.human_action)
                    || !_validAction(state.last_round.bot_action)
                    || !_boundedInteger(state.last_round.damage_to_human, 0, 2)
                    || !_boundedInteger(state.last_round.damage_to_bot, 0, 2)))
            return false
        if (state.round === 0 && (state.last_round !== null
                || state.human.core !== 8 || state.human.energy !== 2
                || state.bot.core !== 8 || state.bot.energy !== 2))
            return false
        if (state.round > 0 && state.last_round === null)
            return false
        if (status === "active")
            return state.phase === "awaiting_human" && state.outcome === null
                    && state.round < 12 && state.human.core > 0 && state.bot.core > 0
        return state.phase === "completed" && _validSoloOutcome(state.outcome)
                && state.outcome.human_core === state.human.core
                && state.outcome.bot_core === state.bot.core
                && state.outcome.human_energy === state.human.energy
                && state.outcome.bot_energy === state.bot.energy
                && state.outcome.rounds_played === state.round
                && state.outcome.reason === ((state.human.core === 0 || state.bot.core === 0)
                                             ? "core_destroyed" : "round_limit")
                && state.outcome.winner === _soloWinner(state)
                && ((state.human.core === 0 || state.bot.core === 0) || state.round === 12)
    }

    function _validVersusState(state, status) {
        if (!_exactKeys(state, ["schema_version", "rules_version", "turn", "max_turns", "phase",
                                "active_seat", "players", "last_turn", "outcome"])
                || state.schema_version !== 1 || state.rules_version !== 2
                || !_boundedInteger(state.turn, 0, 24) || state.max_turns !== 24
                || !Array.isArray(state.players) || state.players.length !== 2)
            return false
        for (let index = 0; index < 2; index++) {
            const player = state.players[index]
            if (!_exactKeys(player, ["seat", "core", "energy", "guard"])
                    || player.seat !== index || !_boundedInteger(player.core, 0, 8)
                    || !_boundedInteger(player.energy, 0, 4)
                    || (player.guard !== 0 && player.guard !== 2))
                return false
        }
        if (state.last_turn !== null
                && (!_exactKeys(state.last_turn, ["turn", "actor_seat", "action",
                                                  "damage_to_opponent", "blocked_damage"])
                    || state.last_turn.turn !== state.turn
                    || !_boundedInteger(state.last_turn.actor_seat, 0, 1)
                    || !_validAction(state.last_turn.action)
                    || !_boundedInteger(state.last_turn.damage_to_opponent, 0, 2)
                    || !_boundedInteger(state.last_turn.blocked_damage, 0, 2)))
            return false
        if (state.turn === 0 && (state.last_turn !== null
                || state.players[0].core !== 8 || state.players[0].energy !== 2
                || state.players[0].guard !== 0 || state.players[1].core !== 8
                || state.players[1].energy !== 2 || state.players[1].guard !== 0))
            return false
        if (state.turn > 0 && (state.last_turn === null
                || state.last_turn.actor_seat !== (state.turn - 1) % 2
                || !_validVersusTurnEvidence(state)))
            return false
        if (status === "active")
            return state.phase === "awaiting_action" && state.outcome === null
                    && state.active_seat === state.turn % 2 && state.turn < 24
                    && state.players[0].core > 0 && state.players[1].core > 0
        return state.phase === "completed" && state.active_seat === null
                && _validVersusOutcome(state.outcome)
                && state.outcome.seat_0_core === state.players[0].core
                && state.outcome.seat_1_core === state.players[1].core
                && state.outcome.seat_0_energy === state.players[0].energy
                && state.outcome.seat_1_energy === state.players[1].energy
                && state.outcome.turns_played === state.turn
                && state.outcome.reason === ((state.players[0].core === 0
                                               || state.players[1].core === 0)
                                              ? "core_destroyed" : "round_limit")
                && state.outcome.winner === _versusWinner(state)
                && ((state.players[0].core === 0 || state.players[1].core === 0)
                    || state.turn === 24)
    }

    function _validCombatant(value) {
        return _exactKeys(value, ["core", "energy"])
                && _boundedInteger(value.core, 0, 8) && _boundedInteger(value.energy, 0, 4)
    }

    function _validSoloOutcome(value) {
        return _exactKeys(value, ["winner", "reason", "human_core", "bot_core", "human_energy",
                                  "bot_energy", "rounds_played"])
                && ["human", "bot", "draw"].indexOf(value.winner) !== -1
                && _validOutcomeReason(value.reason)
                && _boundedInteger(value.human_core, 0, 8) && _boundedInteger(value.bot_core, 0, 8)
                && _boundedInteger(value.human_energy, 0, 4) && _boundedInteger(value.bot_energy, 0, 4)
                && _boundedInteger(value.rounds_played, 1, 12)
    }

    function _validVersusOutcome(value) {
        return _exactKeys(value, ["winner", "reason", "seat_0_core", "seat_1_core",
                                  "seat_0_energy", "seat_1_energy", "turns_played"])
                && ["seat_0", "seat_1", "draw"].indexOf(value.winner) !== -1
                && _validOutcomeReason(value.reason)
                && _boundedInteger(value.seat_0_core, 0, 8) && _boundedInteger(value.seat_1_core, 0, 8)
                && _boundedInteger(value.seat_0_energy, 0, 4) && _boundedInteger(value.seat_1_energy, 0, 4)
                && _boundedInteger(value.turns_played, 1, 24)
    }

    function _validProviderResult(value) {
        return _exactKeys(value, ["outcome", "public_summary", "provider_revision", "projected_at"])
                && _boundedString(value.outcome, 64, 1)
                && typeof value.public_summary === "object" && value.public_summary !== null
                && !Array.isArray(value.public_summary)
                && Number.isSafeInteger(value.provider_revision) && value.provider_revision >= 0
                && _validTimestamp(value.projected_at)
    }

    function _validConnection(value) {
        return _exactKeys(value, ["persona", "connected_at"])
                && _validPersona(value.persona) && value.persona.id !== actor.id
                && _validTimestamp(value.connected_at)
    }

    function _validPersona(value) {
        return _exactKeys(value, ["id", "handle", "display_name", "bio", "status_message",
                                  "created_at", "updated_at"])
                && _validUuid(value.id) && typeof value.handle === "string"
                && /^[a-z0-9][a-z0-9_-]{2,23}$/.test(value.handle)
                && typeof value.display_name === "string"
                && Array.from(value.display_name).length >= 1
                && Array.from(value.display_name).length <= 64
                && !/[\u0000-\u001f\u007f]/.test(value.display_name)
                && typeof value.bio === "string" && Array.from(value.bio).length <= 1000
                && !/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(value.bio)
                && typeof value.status_message === "string"
                && Array.from(value.status_message).length <= 160
                && !/[\u0000-\u001f\u007f]/.test(value.status_message)
                && _validTimestamp(value.created_at) && _validTimestamp(value.updated_at)
    }

    function _validVersusTurnEvidence(state) {
        const record = state.last_turn
        const actor = state.players[record.actor_seat]
        const opponent = state.players[1 - record.actor_seat]
        if (record.action === "strike")
            return record.damage_to_opponent + record.blocked_damage === 2
                    && (record.blocked_damage === 0 || record.blocked_damage === 2)
                    && opponent.guard === 0
        if (record.action === "guard")
            return record.damage_to_opponent === 0 && record.blocked_damage === 0
                    && actor.guard === 2
        return record.damage_to_opponent === 0 && record.blocked_damage === 0
                && actor.guard === 0
    }

    function _soloWinner(state) {
        if (state.human.core > state.bot.core)
            return "human"
        if (state.human.core < state.bot.core)
            return "bot"
        if (state.human.energy > state.bot.energy)
            return "human"
        if (state.human.energy < state.bot.energy)
            return "bot"
        return "draw"
    }

    function _versusWinner(state) {
        if (state.players[0].core > state.players[1].core)
            return "seat_0"
        if (state.players[0].core < state.players[1].core)
            return "seat_1"
        if (state.players[0].energy > state.players[1].energy)
            return "seat_0"
        if (state.players[0].energy < state.players[1].energy)
            return "seat_1"
        return "draw"
    }

    function _validBoundedArray(value, validator) {
        if (!Array.isArray(value) || value.length > 100)
            return false
        for (let index = 0; index < value.length; index++)
            if (!validator.call(root, value[index]))
                return false
        return true
    }

    function _parseDocument(body) {
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

    function _errorCode(value) {
        return _exactKeys(value, ["error"]) && _exactKeys(value.error, ["code", "message"])
                && _boundedString(value.error.code, 64, 1)
                && _boundedString(value.error.message, 512, 1) ? value.error.code : ""
    }

    function _invalidSession(status, value) {
        return status === 401 && _errorCode(value) === "invalid_session"
    }

    function _exactKeys(value, expected) {
        if (!value || typeof value !== "object" || Array.isArray(value))
            return false
        return JSON.stringify(Object.keys(value).sort()) === JSON.stringify(expected.slice().sort())
    }

    function _validUuid(value) {
        return typeof value === "string"
                && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value)
    }

    function _newUuid() {
        let timestamp = Date.now()
        return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function(character) {
            const random = (timestamp + Math.random() * 16) % 16 | 0
            timestamp = Math.floor(timestamp / 16)
            return (character === "x" ? random : (random & 0x3) | 0x8).toString(16)
        })
    }

    function _validTimestamp(value) {
        return typeof value === "string" && value.endsWith("Z") && Number.isFinite(Date.parse(value))
    }

    function _boundedString(value, maximum, minimum) {
        return typeof value === "string" && Array.from(value).length >= (minimum || 0)
                && Array.from(value).length <= maximum
                && !/[\u0000-\u001f\u007f]/.test(value)
    }

    function _boundedInteger(value, minimum, maximum) {
        return Number.isSafeInteger(value) && value >= minimum && value <= maximum
    }

    function _validAction(value) {
        return value === "strike" || value === "guard" || value === "charge"
    }

    function _validOutcomeReason(value) {
        return value === "core_destroyed" || value === "round_limit"
    }

    function _ready() {
        return !busy && _validPersona(actor) && sessionController.hasSession
    }

    function _actorPath() {
        return "/v1/personas/" + actor.id
    }
}
