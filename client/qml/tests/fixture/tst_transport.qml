import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "OnboardingTransport"
    property var fixtureConfig: ({})

    Component { id: controllerComponent; App.OnboardingController {} }
    Component { id: apiComponent; App.ApiClient {} }

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

    function test_health_protocol_failures() {
        let controller = controllerAt(fixtureConfig.malformed_url)
        tryCompare(controller, "connectionState", "protocol_error", 5000)
        compare(controller.hasSession, false)
        controller.destroy()

        controller = controllerAt(fixtureConfig.wrong_identity_url)
        tryCompare(controller, "connectionState", "protocol_error", 5000)
        compare(controller.hasSession, false)
        controller.destroy()
    }

    function test_health_timeout_and_size_bound() {
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
                   "taken_user", "TEST-ONLY-registration-passphrase"))
        tryVerify(function() { return !controller.busy }, 5000)
        compare(controller.errorText, "That username is already registered.")
        compare(controller.suggestedUsername, "")

        verify(controller.registerAccount(
                   "malformed_register", "TEST-ONLY-registration-passphrase"))
        tryVerify(function() {
            return !controller.busy
                    && controller.statusText.indexOf("unexpected") !== -1
        }, 5000)
        verify(!controller.hasSession)
        compare(controller.suggestedUsername, "")
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
        const slowGeneration = api.request("slow", "GET", "/health", null, false)
        verify(slowGeneration > 0)
        verify(api.configure(fixtureConfig.server_url).ok)
        const fastGeneration = api.request("fast", "GET", "/health", null, false)
        verify(fastGeneration > slowGeneration)
        tryCompare(spy, "count", 1, 5000)
        compare(spy.signalArguments[0][0], fastGeneration)
        compare(spy.signalArguments[0][1], "fast")
        wait(700)
        compare(spy.count, 1)
    }

    Component { id: signalSpyComponent; SignalSpy {} }
}
