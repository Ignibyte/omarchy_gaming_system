import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "OnboardingTransport"
    property var fixtureConfig: ({})

    Component { id: controllerComponent; App.OnboardingController {} }
    Component { id: apiComponent; App.ApiClient {} }
    Component { id: profileStoreComponent; App.ServerProfiles {} }

    function capabilities() {
        return [
            "accounts.invite-registration.v1",
            "auth.device-sessions.v1",
            "auth.totp.v1",
            "games.challenges.v1",
            "games.sessions.v1",
            "identity.personas.v1",
            "social.connections.v1",
            "social.private-inbox.v1",
            "social.reporting.v1",
            "sync.cursor.v1",
            "sync.websocket-hints.v1"
        ]
    }

    function loadConfig() {
        const request = new XMLHttpRequest()
        request.open("GET", Qt.resolvedUrl("../../../../.dev/qml-onboarding/fixture-config.json"), false)
        request.send()
        return JSON.parse(request.responseText)
    }

    function initTestCase() {
        fixtureConfig = loadConfig()
        verify(fixtureConfig.server_url !== "")
    }

    function controllerAt(url) {
        const controller = createTemporaryObject(controllerComponent, testCase)
        verify(controller !== null)
        controller.initialize(url)
        return controller
    }

    function test_endpoint_admission() {
        const api = createTemporaryObject(apiComponent, testCase)
        verify(api !== null)
        compare(api.normalizeEndpoint("http://127.0.0.1:8080/").url,
                "http://127.0.0.1:8080")
        compare(api.normalizeEndpoint("http://localhost").ok, true)
        compare(api.normalizeEndpoint("http://[::1]:8080").ok, true)
        compare(api.normalizeEndpoint("https://games.example.net:443").ok, true)
        compare(api.normalizeEndpoint("http://games.example.net").ok, false)
        compare(api.normalizeEndpoint("https://user@games.example.net").ok, false)
        compare(api.normalizeEndpoint("https://games.example.net/path").ok, false)
        compare(api.normalizeEndpoint("https://games.example.net?query=x").ok, false)
        compare(api.normalizeEndpoint("https://games.example.net#fragment").ok, false)
        compare(api.normalizeEndpoint("https://games.example.net:65536").ok, false)
    }

    function test_discovery_protocol_failures() {
        let controller = controllerAt(fixtureConfig.malformed_url)
        tryCompare(controller, "connectionState", "protocol_error", 5000)
        compare(controller.hasSession, false)
        controller.destroy()

        controller = controllerAt(fixtureConfig.wrong_identity_url)
        tryCompare(controller, "connectionState", "protocol_error", 5000)
        compare(controller.hasSession, false)
        controller.destroy()
    }

    function test_discovery_timeout_and_size_bound() {
        let controller = createTemporaryObject(controllerComponent, testCase)
        controller.requestTimeoutMilliseconds = 100
        controller.initialize(fixtureConfig.slow_url)
        tryCompare(controller, "connectionState", "offline", 5000)
        verify(controller.errorText.indexOf("timed out") !== -1)
        wait(2300)
        controller.destroy()

        controller = createTemporaryObject(controllerComponent, testCase)
        controller.maximumResponseBytes = 1024
        controller.initialize(fixtureConfig.oversized_url)
        tryCompare(controller, "connectionState", "offline", 5000)
        verify(controller.errorText.indexOf("exceeded") !== -1)
        controller.destroy()
    }

    function test_discovery_negotiates_unknown_and_incompatible_capabilities() {
        let controller = controllerAt(fixtureConfig.server_two_url)
        tryCompare(controller, "state", "access", 5000)
        compare(controller.connectionState, "ready")
        verify(controller.currentServer.capabilities.indexOf("future.arcade-mode.v1") !== -1)
        controller.destroy()

        controller = controllerAt(fixtureConfig.incompatible_url)
        tryCompare(controller, "connectionState", "incompatible", 5000)
        compare(controller.state, "connection")
        verify(!controller.hasSession)
        controller.destroy()
    }

    function test_operator_custom_module_disclosure_is_identity_bound_and_private() {
        let controller = controllerAt(fixtureConfig.custom_modules_url)
        tryCompare(controller, "state", "access", 5000)
        compare(controller.connectionState, "ready")
        verify(controller.currentServer.capabilities.indexOf(
                   "server.operator-custom-modules.v1") !== -1)
        const disclosure = controller.currentServer.operator_custom_modules
        compare(disclosure.server_id, controller.currentServer.server_id)
        compare(disclosure.active_count, 1)
        compare(disclosure.behavior_capabilities.length, 1)
        compare(disclosure.behavior_capabilities[0], "moderation_labels")
        compare(disclosure.warning,
                "This server runs operator-custom code not reviewed or supported by OmarchyGS.")
        const serialized = controller._profileStore.serializedProfiles()
        verify(serialized.indexOf("component_bytes") === -1)
        verify(serialized.indexOf("signing") === -1)
        verify(serialized.indexOf("module_id") === -1)
        controller.destroy()

        controller = controllerAt(fixtureConfig.custom_modules_hostile_url)
        tryCompare(controller, "connectionState", "protocol_error", 5000)
        compare(controller.hasSession, false)
        controller.destroy()

        controller = controllerAt(fixtureConfig.custom_modules_wrong_server_url)
        tryCompare(controller, "connectionState", "protocol_error", 5000)
        compare(controller.hasSession, false)
        controller.destroy()
    }

    function test_profiles_are_isolated_and_identity_replacement_fails_closed() {
        const controller = createTemporaryObject(controllerComponent, testCase)
        verify(controller !== null)
        controller._profileStore.clearProfiles()

        verify(controller.connectToServer(fixtureConfig.server_url, true))
        tryCompare(controller, "state", "access", 5000)
        compare(controller.serverProfiles.length, 1)
        verify(controller.signIn(
                   "normal_user", "TEST-ONLY-registration-passphrase", "Omarchy QML"))
        tryCompare(controller, "state", "personas", 5000)
        verify(controller.hasSession)

        verify(controller.connectToServer(fixtureConfig.server_two_url, true))
        compare(controller.hasSession, false)
        compare(controller.hasMfaChallenge, false)
        compare(controller.personas.length, 0)
        compare(controller.selectedPersona, null)
        compare(controller.suggestedUsername, "")
        tryCompare(controller, "state", "access", 5000)
        compare(controller.serverProfiles.length, 2)
        verify(controller._profileStore.serializedProfiles().indexOf("ogs1_") === -1)
        verify(controller._profileStore.serializedProfiles().indexOf("password") === -1)

        controller._profileStore.clearProfiles()
        verify(controller._profileStore.saveProfile({
            "origin": fixtureConfig.identity_changed_url,
            "server_id": "15151515-1515-4515-8515-151515151515",
            "server_name": "Pinned Fixture",
            "protocol_version": 1,
            "capabilities": capabilities()
        }))
        verify(controller.connectSavedProfile(0))
        tryCompare(controller, "connectionState", "identity_mismatch", 5000)
        compare(controller.state, "connection")
        verify(!controller.hasSession)
        compare(controller.serverProfiles.length, 1)
        compare(controller.serverProfiles[0].server_id,
                "15151515-1515-4515-8515-151515151515")
        controller._profileStore.clearProfiles()
        controller.destroy()
    }

    function test_hostile_profile_state_is_discarded_without_connection() {
        const store = createTemporaryObject(profileStoreComponent, testCase)
        verify(store !== null)
        store.clearProfiles()
        const base = {
            "origin": fixtureConfig.server_url,
            "server_id": "12121212-1212-4212-8212-121212121212",
            "server_name": "Fixture Community",
            "protocol_version": 1,
            "capabilities": capabilities()
        }

        const withCredential = Object.assign({}, base, {"token": "ogs1_" + "A".repeat(43)})
        store._settings.setValue("profiles", JSON.stringify([withCredential]))
        store._settings.sync()
        verify(!store.reload())
        compare(store.profiles.length, 0)
        compare(store.serializedProfiles(), "[]")

        store._settings.setValue("profiles", JSON.stringify([base, base]))
        store._settings.sync()
        verify(!store.reload())
        compare(store.profiles.length, 0)

        store._settings.setValue("profiles", "[" + "x".repeat(17000) + "]")
        store._settings.sync()
        verify(!store.reload())
        compare(store.profiles.length, 0)

        const unsupported = Object.assign({}, base, {"protocol_version": 2})
        store._settings.setValue("profiles", JSON.stringify([unsupported]))
        store._settings.sync()
        verify(!store.reload())
        compare(store.profiles.length, 0)
        store.clearProfiles()
        store.destroy()
    }

    function test_malformed_success_and_authenticated_shape_fail_closed() {
        let controller = controllerAt(fixtureConfig.server_url)
        tryCompare(controller, "state", "access", 5000)
        controller.signIn("malformed_login", "TEST-ONLY-registration-passphrase", "Omarchy QML")
        tryVerify(function() { return !controller.busy }, 5000)
        compare(controller.state, "access")
        verify(!controller.hasSession)
        verify(controller.statusText.indexOf("unexpected") !== -1)
        controller.destroy()

        controller = controllerAt(fixtureConfig.server_url)
        tryCompare(controller, "state", "access", 5000)
        controller.signIn("malformed_personas", "TEST-ONLY-registration-passphrase", "Omarchy QML")
        tryVerify(function() {
            return controller.state === "access" && !controller.busy
                    && controller.statusText.indexOf("not accepted") !== -1
        }, 5000)
        verify(!controller.hasSession)
        controller.destroy()
    }

    function test_registration_errors_are_safe_and_malformed_success_is_rejected() {
        const controller = controllerAt(fixtureConfig.server_url)
        tryCompare(controller, "state", "access", 5000)
        verify(controller.chooseAccessMode("register"))
        verify(controller.registerAccount(
                   "ogsi_" + "I".repeat(43),
                   "taken_user", "TEST-ONLY-registration-passphrase"))
        tryVerify(function() { return !controller.busy }, 5000)
        compare(controller.errorText, "That username is already registered.")
        compare(controller.suggestedUsername, "")

        verify(controller.registerAccount(
                   "ogsi_" + "I".repeat(43),
                   "malformed_register", "TEST-ONLY-registration-passphrase"))
        tryVerify(function() {
            return !controller.busy
                    && controller.statusText.indexOf("unexpected") !== -1
        }, 5000)
        verify(!controller.hasSession)
        compare(controller.suggestedUsername, "")

        verify(controller.registerAccount(
                   "ogsi_invalid_fixture",
                   "invalid_invite", "TEST-ONLY-registration-passphrase"))
        tryVerify(function() { return !controller.busy }, 5000)
        compare(controller.errorText,
                "That invitation is invalid, expired, revoked, or already used.")
    }

    function test_mfa_terminal_error_clears_challenge_authority() {
        const controller = controllerAt(fixtureConfig.server_url)
        tryCompare(controller, "state", "access", 5000)
        verify(controller.signIn(
                   "mfa_user", "TEST-ONLY-registration-passphrase", "Omarchy QML"))
        tryCompare(controller, "state", "mfa", 5000)
        verify(controller.hasMfaChallenge)
        verify(controller.completeMfa("EXPIRED"))
        tryVerify(function() {
            return !controller.busy && controller.state === "access"
        }, 5000)
        verify(!controller.hasMfaChallenge)
        verify(!controller.hasSession)
        compare(controller.errorText,
                "That MFA challenge expired or was already used.")
    }

    function test_persona_conflict_preserves_valid_session() {
        const controller = controllerAt(fixtureConfig.server_url)
        tryCompare(controller, "state", "access", 5000)
        verify(controller.signIn(
                   "normal_user", "TEST-ONLY-registration-passphrase", "Omarchy QML"))
        tryCompare(controller, "state", "personas", 5000)
        verify(controller.createPersona(
                   "taken_handle", "Taken", "Fixture", "Ready"))
        tryVerify(function() { return !controller.busy }, 5000)
        compare(controller.state, "personas")
        verify(controller.hasSession)
        compare(controller.selectedPersona, null)
        compare(controller.errorText, "That persona handle is already in use.")
    }

    function test_success_contract_enforces_server_bounds_and_future_session() {
        const controller = createTemporaryObject(controllerComponent, testCase)
        verify(controller !== null)
        verify(!controller._validAccount({
            "id": "11111111-1111-4111-8111-111111111111", "username": ""
        }))

        const persona = {
            "id": "33333333-3333-4333-8333-333333333333",
            "handle": "bounded",
            "display_name": "Bounded Player",
            "bio": "x".repeat(1000),
            "status_message": "y".repeat(160),
            "created_at": "2026-08-25T20:00:00.000Z",
            "updated_at": "2026-08-25T20:00:00.000Z"
        }
        verify(controller._validPersona(persona))
        persona.bio += "x"
        verify(!controller._validPersona(persona))

        verify(!controller._validSessionCreation({
            "token": "ogs1_" + "A".repeat(43),
            "session": {
                "id": "22222222-2222-4222-8222-222222222222",
                "device_name": "Omarchy QML",
                "created_at": "2000-01-01T00:00:00.000Z",
                "last_used_at": "2000-01-01T00:00:00.000Z",
                "expires_at": "2000-01-02T00:00:00.000Z",
                "revoked_at": null,
                "current": true
            }
        }))
    }

    function test_invalid_session_clears_all_authority() {
        const controller = controllerAt(fixtureConfig.server_url)
        tryCompare(controller, "state", "access", 5000)
        controller.signIn("unauthorized_user", "TEST-ONLY-registration-passphrase", "Omarchy QML")
        tryVerify(function() {
            return controller.state === "access" && !controller.busy
                    && controller.statusText.indexOf("no longer valid") !== -1
        }, 5000)
        verify(!controller.hasSession)
        verify(!controller.hasMfaChallenge)
        compare(controller.personas.length, 0)
        compare(controller.selectedPersona, null)
    }

    function test_local_mfa_expiry_clears_challenge() {
        const controller = createTemporaryObject(controllerComponent, testCase)
        controller.state = "mfa"
        controller._mfaChallengeToken = "ogm1_" + "C".repeat(43)
        controller._mfaExpiresAt = "2000-01-01T00:00:00.000Z"
        controller._expireMfaIfNeeded()
        compare(controller.state, "access")
        verify(!controller.hasMfaChallenge)
        verify(controller.errorText.indexOf("new challenge") !== -1)
    }

    function test_superseded_xhr_cannot_complete() {
        const api = createTemporaryObject(apiComponent, testCase)
        const spy = createTemporaryObject(signalSpyComponent, testCase,
                                          {"target": api, "signalName": "finished"})
        verify(api.configure(fixtureConfig.slow_url).ok)
        api.timeoutMilliseconds = 2000
        const slowGeneration = api.request(
                    "slow", "GET", "/.well-known/omarchygs", null, false)
        verify(slowGeneration > 0)
        verify(api.configure(fixtureConfig.server_url).ok)
        const fastGeneration = api.request(
                    "fast", "GET", "/.well-known/omarchygs", null, false)
        verify(fastGeneration > slowGeneration)
        tryCompare(spy, "count", 1, 5000)
        compare(spy.signalArguments[0][0], fastGeneration)
        compare(spy.signalArguments[0][1], "fast")
        wait(700)
        compare(spy.count, 1)
    }

    Component { id: signalSpyComponent; SignalSpy {} }
}
