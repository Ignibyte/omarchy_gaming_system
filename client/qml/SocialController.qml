import QtQuick

QtObject {
    id: root

    signal reportSubmitted()

    required property var sessionController
    property var actor: null
    property string statusText: ""
    property string errorText: ""
    property string loadState: "idle"
    property var incomingRequests: []
    property var outgoingRequests: []
    property var connections: []
    property var blocks: []
    property var conversations: []
    property var selectedConversation: null
    property var messages: []
    property var nextBefore: null
    property var foundPersona: null
    readonly property bool busy: _expectedGeneration !== 0

    property int _expectedGeneration: 0
    property string _expectedOperation: ""
    property string _pendingHandle: ""
    property var _pendingReport: null
    property bool _historyAppendOlder: false

    property Connections _requestConnection: Connections {
        target: root.sessionController
        function onPlayerRequestFinished(generation, operation, status, body, transportError) {
            root._handleFinished(generation, operation, status, body, transportError)
        }
    }

    onActorChanged: reset()

    function reset() {
        if (_expectedGeneration !== 0)
            sessionController.cancelPlayerRequest()
        _expectedGeneration = 0
        _expectedOperation = ""
        statusText = ""
        errorText = ""
        loadState = "idle"
        incomingRequests = []
        outgoingRequests = []
        connections = []
        blocks = []
        conversations = []
        selectedConversation = null
        messages = []
        nextBefore = null
        foundPersona = null
        _pendingHandle = ""
        _pendingReport = null
        _historyAppendOlder = false
    }

    function refreshSocial() {
        if (!_ready())
            return false
        errorText = ""
        statusText = "Loading connection requests..."
        loadState = "loading"
        return _request("player_social_requests", "GET", _actorPath() + "/connection-requests")
    }

    function refreshInbox() {
        if (!_ready())
            return false
        errorText = ""
        statusText = "Loading private conversations..."
        loadState = "loading"
        return _request("player_inbox_list", "GET", _actorPath() + "/conversations?limit=100")
    }

    function requestConnectionByHandle(handle) {
        if (!_ready())
            return false
        const normalized = String(handle).trim().toLowerCase()
        if (!/^[a-z0-9][a-z0-9_-]{2,23}$/.test(normalized)) {
            errorText = "Enter an exact 3–24 character persona handle."
            return false
        }
        if (normalized === actor.handle.toLowerCase()) {
            errorText = "Choose another persona."
            return false
        }
        _pendingHandle = normalized
        foundPersona = null
        errorText = ""
        statusText = "Resolving exact persona handle..."
        return _request("player_social_lookup", "GET",
                        "/v1/personas/by-handle/" + encodeURIComponent(normalized), null, false)
    }

    function reportPersonaByHandle(handle, category, detail) {
        if (!_ready())
            return false
        const normalized = String(handle).trim().toLowerCase()
        const normalizedCategory = String(category)
        const normalizedDetail = String(detail).trim()
        if (!/^[a-z0-9][a-z0-9_-]{2,23}$/.test(normalized)) {
            errorText = "Enter an exact 3–24 character persona handle."
            return false
        }
        if (normalized === actor.handle.toLowerCase()) {
            errorText = "Choose another persona."
            return false
        }
        if (["harassment", "spam", "cheating", "other"].indexOf(normalizedCategory) === -1
                || _characterCount(normalizedDetail) < 1
                || _characterCount(normalizedDetail) > 1000
                || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(normalizedDetail)) {
            errorText = "Reports need a category and 1–1,000 supported detail characters."
            return false
        }
        if (!_pendingReport || _pendingReport.handle !== normalized
                || _pendingReport.category !== normalizedCategory
                || _pendingReport.detail !== normalizedDetail) {
            _pendingReport = {
                "handle": normalized,
                "category": normalizedCategory,
                "detail": normalizedDetail,
                "idempotency_key": _newUuid()
            }
        }
        _pendingHandle = normalized
        foundPersona = null
        errorText = ""
        statusText = "Resolving report subject..."
        return _request("player_report_lookup", "GET",
                        "/v1/personas/by-handle/" + encodeURIComponent(normalized), null, false)
    }

    function acceptRequest(persona) {
        return _targetMutation("player_social_accept", "PUT", "/connections/", persona,
                               "Accepting connection...")
    }

    function removeRelationship(persona) {
        return _targetMutation("player_social_remove", "DELETE", "/connections/", persona,
                               "Updating connection...")
    }

    function blockPersona(persona) {
        return _targetMutation("player_social_block", "PUT", "/blocks/", persona,
                               "Blocking persona...")
    }

    function unblockPersona(persona) {
        return _targetMutation("player_social_unblock", "DELETE", "/blocks/", persona,
                               "Unblocking persona...")
    }

    function openConversation(conversation) {
        if (!_validConversation(conversation) || busy)
            return false
        selectedConversation = conversation
        messages = []
        nextBefore = null
        _historyAppendOlder = false
        errorText = ""
        statusText = "Loading conversation history..."
        loadState = "loading"
        return _request("player_inbox_history", "GET", _conversationPath(conversation) + "/messages?limit=50")
    }

    function closeConversation() {
        if (busy)
            sessionController.cancelPlayerRequest()
        _expectedGeneration = 0
        _expectedOperation = ""
        selectedConversation = null
        messages = []
        nextBefore = null
        errorText = ""
        return true
    }

    function loadOlderMessages() {
        if (!_validConversation(selectedConversation) || busy
                || !Number.isInteger(nextBefore) || nextBefore < 1)
            return false
        _historyAppendOlder = true
        errorText = ""
        statusText = "Loading older messages..."
        return _request("player_inbox_history", "GET",
                        _conversationPath(selectedConversation) + "/messages?before="
                        + nextBefore + "&limit=50")
    }

    function sendMessage(body) {
        if (!_validConversation(selectedConversation) || busy)
            return false
        const text = String(body).trim()
        if (_characterCount(text) < 1 || _characterCount(text) > 4000
                || /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(text)) {
            errorText = "Messages must contain 1–4,000 characters without control codes."
            return false
        }
        errorText = ""
        statusText = "Sending private message..."
        return _request("player_inbox_send", "POST",
                        _conversationPath(selectedConversation) + "/messages", {"body": text})
    }

    function messageText(message) {
        if (message.type === "user")
            return "@" + message.sender.handle + ": " + message.body
        const system = message.system
        const actorText = "@" + system.actor.handle
        if (system.type === "connection_accepted")
            return actorText + " accepted the connection."
        if (system.type === "game_challenge_created")
            return actorText + " created a game challenge."
        if (system.type === "game_challenge_accepted")
            return actorText + " accepted a game challenge."
        if (system.type === "game_challenge_declined")
            return actorText + " declined a game challenge."
        return actorText + " canceled a game challenge."
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

    function _targetMutation(operation, method, segment, persona, message) {
        if (!_ready() || !_validPersona(persona) || persona.id === actor.id)
            return false
        errorText = ""
        statusText = message
        return _request(operation, method, _actorPath() + segment + persona.id)
    }

    function _handleFinished(generation, operation, status, body, transportError) {
        if (generation !== _expectedGeneration || operation !== _expectedOperation)
            return
        _expectedGeneration = 0
        _expectedOperation = ""
        if (transportError !== "") {
            _historyAppendOlder = false
            loadState = "error"
            errorText = transportError === "timeout" ? "The request timed out. Try again."
                      : transportError === "response_too_large" ? "The response exceeded the client limit."
                      : "The server could not complete the request."
            statusText = "Request not completed."
            return
        }

        if (status === 204 && (operation === "player_social_remove"
                              || operation === "player_social_unblock")) {
            statusText = "Social state updated."
            Qt.callLater(function() { root.refreshSocial() })
            return
        }

        const parsed = _parseDocument(body)
        if (!parsed.ok) {
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
            _actionFailure(document)
            return
        }

        if (operation === "player_social_requests") {
            if (!_validRequestInventory(document)) { _protocolFailure(); return }
            incomingRequests = document.incoming
            outgoingRequests = document.outgoing
            statusText = "Loading accepted connections..."
            _request("player_social_connections", "GET", _actorPath() + "/connections")
        } else if (operation === "player_social_connections") {
            if (!_exactKeys(document, ["connections"]) || !_validBoundedArray(document.connections, _validConnection)) { _protocolFailure(); return }
            connections = document.connections
            statusText = "Loading private blocks..."
            _request("player_social_blocks", "GET", _actorPath() + "/blocks")
        } else if (operation === "player_social_blocks") {
            if (!_exactKeys(document, ["blocks"]) || !_validBoundedArray(document.blocks, _validBlock)) { _protocolFailure(); return }
            blocks = document.blocks
            loadState = "ready"
            statusText = "Social state is current."
            errorText = ""
        } else if (operation === "player_social_lookup") {
            if (!_validPersona(document) || document.id === actor.id || document.handle !== _pendingHandle) { _protocolFailure(); return }
            foundPersona = document
            statusText = "Sending connection request..."
            _request("player_social_request", "PUT", _actorPath() + "/connection-requests/" + document.id)
        } else if (operation === "player_report_lookup") {
            if (!_pendingReport || !_validPersona(document) || document.id === actor.id
                    || document.handle !== _pendingReport.handle) { _protocolFailure(); return }
            foundPersona = document
            statusText = "Submitting persona report..."
            _request("player_report_create", "POST", _actorPath() + "/reports", {
                "idempotency_key": _pendingReport.idempotency_key,
                "subject_persona_id": document.id,
                "category": _pendingReport.category,
                "detail": _pendingReport.detail
            })
        } else if (operation === "player_report_create") {
            if ((status !== 200 && status !== 201) || !_pendingReport
                    || !_validReportReceipt(document)
                    || document.idempotency_key !== _pendingReport.idempotency_key) {
                _protocolFailure()
                return
            }
            _pendingReport = null
            _pendingHandle = ""
            foundPersona = null
            loadState = "ready"
            statusText = "Report submitted for operator review."
            errorText = ""
            reportSubmitted()
        } else if (operation === "player_social_request") {
            if ((status !== 200 && status !== 201) || !_validRequest(document)) { _protocolFailure(); return }
            statusText = "Connection request sent."
            Qt.callLater(function() { root.refreshSocial() })
        } else if (operation === "player_social_accept") {
            if (!_validConnection(document)) { _protocolFailure(); return }
            statusText = "Connection accepted."
            Qt.callLater(function() { root.refreshSocial() })
        } else if (operation === "player_social_block") {
            if ((status !== 200 && status !== 201) || !_validBlock(document)) { _protocolFailure(); return }
            statusText = "Persona blocked."
            Qt.callLater(function() { root.refreshSocial() })
        } else if (operation === "player_inbox_list") {
            if (!_exactKeys(document, ["conversations"]) || !_validBoundedArray(document.conversations, _validConversation)) { _protocolFailure(); return }
            conversations = document.conversations
            loadState = "ready"
            statusText = conversations.length === 0 ? "No private conversations yet." : "Private inbox is current."
            errorText = ""
        } else if (operation === "player_inbox_history") {
            if (!_validMessagePage(document)) { _protocolFailure(); return }
            if (_historyAppendOlder && messages.length > 0 && document.messages.length > 0
                    && document.messages[document.messages.length - 1].sequence
                       >= messages[0].sequence) {
                _protocolFailure()
                return
            }
            messages = _historyAppendOlder ? document.messages.concat(messages) : document.messages
            nextBefore = document.next_before
            _historyAppendOlder = false
            loadState = "ready"
            statusText = "Conversation history is current."
            if (selectedConversation && selectedConversation.unread_count > 0 && messages.length > 0)
                Qt.callLater(function() { root._markRead(messages[messages.length - 1].id) })
        } else if (operation === "player_inbox_send") {
            if (status !== 201 || !_validMessage(document) || document.type !== "user" || document.sender.id !== actor.id) { _protocolFailure(); return }
            if (messages.length > 0
                    && document.sequence <= messages[messages.length - 1].sequence) {
                _protocolFailure()
                return
            }
            messages = messages.concat([document])
            statusText = "Private message sent."
            errorText = ""
            Qt.callLater(function() { root.refreshInbox() })
        } else if (operation === "player_inbox_read") {
            if (!_validReadReceipt(document)) { _protocolFailure(); return }
            if (selectedConversation)
                selectedConversation = Object.assign({}, selectedConversation, {"unread_count": document.unread_count})
            loadState = "ready"
            statusText = "Conversation marked read."
            errorText = ""
        }
    }

    function _markRead(messageId) {
        if (!_validConversation(selectedConversation) || !_validUuid(messageId) || busy)
            return false
        return _request("player_inbox_read", "PUT",
                        _conversationPath(selectedConversation) + "/read/" + messageId)
    }

    function _actionFailure(document) {
        const code = _errorCode(document)
        if (code === "") { _protocolFailure(); return }
        const messagesByCode = {
            "persona_not_found": "The selected persona or target is unavailable.",
            "connection_unavailable": "That connection is unavailable.",
            "connection_request_not_found": "That incoming request is no longer available.",
            "connection_request_pending": "That persona already sent you a request.",
            "connection_already_exists": "Those personas are already connected.",
            "conversation_not_found": "That conversation is unavailable.",
            "conversation_unavailable": "That conversation cannot accept a new message.",
            "message_not_found": "That message is unavailable.",
            "invalid_message_body": "Messages must contain 1–4,000 supported characters.",
            "invalid_pagination": "That history page is unavailable.",
            "invalid_report": "Reports need a valid subject, category, and detail.",
            "report_idempotency_conflict": "That report operation conflicted with an earlier submission.",
            "report_limit_reached": "Resolve an existing report with the server operator before filing another.",
            "internal_error": "The server could not complete the request."
        }
        errorText = messagesByCode[code] || "The server rejected the request."
        statusText = "Request not accepted."
        loadState = "error"
    }

    function _protocolFailure() {
        _historyAppendOlder = false
        loadState = "error"
        statusText = "The server response was not accepted."
        errorText = "Keep the current state and retry after checking the server."
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

    function _validRequestInventory(value) {
        return _exactKeys(value, ["incoming", "outgoing"])
                && _validBoundedArray(value.incoming, _validRequest)
                && _validBoundedArray(value.outgoing, _validRequest)
    }

    function _validBoundedArray(value, validator) {
        if (!Array.isArray(value) || value.length > 100)
            return false
        for (let index = 0; index < value.length; index++)
            if (!validator.call(root, value[index]))
                return false
        return true
    }

    function _validRequest(value) {
        return _exactKeys(value, ["persona", "created_at"])
                && _validPersona(value.persona) && _validTimestamp(value.created_at)
    }

    function _validConnection(value) {
        return _exactKeys(value, ["persona", "connected_at"])
                && _validPersona(value.persona) && _validTimestamp(value.connected_at)
    }

    function _validBlock(value) {
        return _exactKeys(value, ["persona", "created_at"])
                && _validPersona(value.persona) && _validTimestamp(value.created_at)
    }

    function _validConversation(value) {
        return _exactKeys(value, ["id", "other_persona", "unread_count", "latest_message",
                                  "created_at", "updated_at"])
                && _validUuid(value.id) && _validPersona(value.other_persona)
                && Number.isSafeInteger(value.unread_count) && value.unread_count >= 0
                && (value.latest_message === null || _validMessage(value.latest_message))
                && _validTimestamp(value.created_at) && _validTimestamp(value.updated_at)
    }

    function _validMessagePage(value) {
        if (!_exactKeys(value, ["messages", "next_before"])
                || !_validBoundedArray(value.messages, _validMessage)
                || !(value.next_before === null || (Number.isSafeInteger(value.next_before) && value.next_before > 0)))
            return false
        for (let index = 1; index < value.messages.length; index++)
            if (value.messages[index - 1].sequence >= value.messages[index].sequence)
                return false
        return true
    }

    function _validMessage(value) {
        if (!value || typeof value !== "object")
            return false
        if (value.type === "user")
            return _exactKeys(value, ["type", "id", "sequence", "sender", "body", "created_at"])
                    && _validMessageBase(value) && _validPersona(value.sender)
                    && _boundedString(value.body, 4000, 1)
                    && value.body === value.body.trim()
                    && !/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(value.body)
        if (value.type !== "system"
                || !_exactKeys(value, ["type", "id", "sequence", "system", "created_at"])
                || !_validMessageBase(value) || !value.system || typeof value.system !== "object")
            return false
        const system = value.system
        if (system.type === "connection_accepted")
            return _exactKeys(system, ["type", "actor"]) && _validPersona(system.actor)
        const challengeKeys = ["type", "actor", "challenge_id"]
        if (system.type === "game_challenge_accepted")
            return _exactKeys(system, challengeKeys.concat(["game_session_id"]))
                    && _validPersona(system.actor) && _validUuid(system.challenge_id)
                    && _validUuid(system.game_session_id)
        return (system.type === "game_challenge_created"
                || system.type === "game_challenge_declined"
                || system.type === "game_challenge_cancelled")
                && _exactKeys(system, challengeKeys) && _validPersona(system.actor)
                && _validUuid(system.challenge_id)
    }

    function _validMessageBase(value) {
        return _validUuid(value.id) && Number.isSafeInteger(value.sequence) && value.sequence > 0
                && _validTimestamp(value.created_at)
    }

    function _validReadReceipt(value) {
        return _exactKeys(value, ["through_message_id", "unread_count"])
                && _validUuid(value.through_message_id)
                && Number.isSafeInteger(value.unread_count) && value.unread_count >= 0
    }

    function _validReportReceipt(value) {
        return _exactKeys(value, ["id", "idempotency_key", "status", "created_at"])
                && _validUuid(value.id) && _validUuid(value.idempotency_key)
                && value.status === "open" && _validTimestamp(value.created_at)
    }

    function _validPersona(value) {
        return _exactKeys(value, ["id", "handle", "display_name", "bio", "status_message",
                                  "created_at", "updated_at"])
                && _validUuid(value.id)
                && typeof value.handle === "string"
                && /^[a-z0-9][a-z0-9_-]{2,23}$/.test(value.handle)
                && _boundedString(value.display_name, 64, 1)
                && !/[\u0000-\u001f\u007f]/.test(value.display_name)
                && _boundedString(value.bio, 1000)
                && !/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/.test(value.bio)
                && _boundedString(value.status_message, 160)
                && !/[\u0000-\u001f\u007f]/.test(value.status_message)
                && _validTimestamp(value.created_at) && _validTimestamp(value.updated_at)
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
        return typeof value === "string" && _characterCount(value) >= (minimum || 0)
                && _characterCount(value) <= maximum
    }

    function _characterCount(value) {
        return Array.from(String(value)).length
    }

    function _ready() {
        return !busy && _validPersona(actor) && sessionController.hasSession
    }

    function _actorPath() {
        return "/v1/personas/" + actor.id
    }

    function _conversationPath(conversation) {
        return _actorPath() + "/conversations/" + conversation.id
    }
}
