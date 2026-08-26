import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "GameCartridgeUi"
    when: windowShown

    property var applicationWindow: null
    property var fixtureConfig: ({})

    Component {
        id: mainComponent
        App.Main { visible: true }
    }

    function loadConfig() {
        const request = new XMLHttpRequest()
        request.open("GET", Qt.resolvedUrl("../../../../.dev/qml-onboarding/fixture-config.json"), false)
        request.send()
        return JSON.parse(request.responseText)
    }

    function resetFixture() {
        const request = new XMLHttpRequest()
        request.open("GET", fixtureConfig.server_url + "/__fixture__/reset-social", false)
        request.send()
        compare(request.status, 200)
    }

    function initTestCase() {
        fixtureConfig = loadConfig()
    }

    function init() {
        resetFixture()
        applicationWindow = createTemporaryObject(mainComponent, testCase)
        verify(applicationWindow !== null)
        applicationWindow.requestActivate()
        tryVerify(function() { return applicationWindow.active }, 3000)
        applicationWindow.onboardingController.showServerConfiguration()
        applicationWindow.onboardingController.connectToServer(fixtureConfig.server_url)
        tryCompare(applicationWindow.onboardingController, "state", "access", 5000)
        signInAndSelectActor()
    }

    function cleanup() {
        if (applicationWindow)
            applicationWindow.destroy()
        applicationWindow = null
    }

    function object(name) {
        const found = findChild(applicationWindow, name)
        verify(found !== null, "missing QML object " + name)
        return found
    }

    function enterText(field, value) {
        field.forceActiveFocus()
        tryVerify(function() { return field.activeFocus })
        field.selectAll()
        for (let index = 0; index < value.length; index++)
            keyClick(value.charAt(index))
        compare(field.text, value)
    }

    function activate(control) {
        control.forceActiveFocus()
        tryVerify(function() { return control.activeFocus })
        keyClick(Qt.Key_Return)
    }

    function clone(value) {
        return JSON.parse(JSON.stringify(value))
    }

    function signInAndSelectActor() {
        enterText(object("usernameField"), "social_user")
        const password = object("passwordField")
        enterText(password, "TEST-ONLY-social-passphrase")
        password.forceActiveFocus()
        keyClick(Qt.Key_Return)
        tryCompare(applicationWindow.onboardingController, "state", "personas", 5000)
        const personaList = object("personaList")
        tryVerify(function() { return personaList.itemAtIndex(0) !== null })
        activate(personaList.itemAtIndex(0))
        tryCompare(applicationWindow.onboardingController, "state", "home")
    }

    function findChallenge(direction, status) {
        for (let index = 0; index < applicationWindow.gameController.challenges.length; index++) {
            const challenge = applicationWindow.gameController.challenges[index]
            if (challenge.direction === direction && challenge.status === status)
                return challenge
        }
        return null
    }

    function test_keyboard_catalog_session_and_solo_command() {
        applicationWindow.width = 640
        applicationWindow.height = 420
        const finalHomeButton = object("homeChangeServerButton")
        tryVerify(function() {
            const position = finalHomeButton.mapToItem(applicationWindow.contentItem, 0, 0)
            return position.x >= 0
                    && position.x + finalHomeButton.width <= applicationWindow.contentItem.width
        }, 1000, "home action grid must remain inside the 640px content width")
        activate(object("homeGamesButton"))
        tryCompare(applicationWindow.onboardingController, "state", "games")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        compare(applicationWindow.gameController.catalog.length, 2)
        compare(applicationWindow.gameController.soloGames().length, 1)
        compare(applicationWindow.gameController.challengeGames().length, 1)
        compare(applicationWindow.gameController.sessions.length, 1)

        verify(applicationWindow.gameController.openSession(
                   applicationWindow.gameController.sessions[0]))
        tryCompare(applicationWindow.onboardingController, "state", "gameplay")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        verify(applicationWindow.gameController.presentation.supported)
        compare(applicationWindow.gameController.presentation.status, "YOUR COMMAND")
        const strike = object("gameStrikeButton")
        verify(strike.Accessible.name.indexOf("two damage") !== -1)
        activate(strike)
        tryVerify(function() {
            return !applicationWindow.gameController.busy
                    && applicationWindow.gameController.selectedSession.revision === 1
        }, 5000, applicationWindow.gameController.statusText + " // "
                 + applicationWindow.gameController.errorText)
        compare(applicationWindow.gameController.presentation.opponent_core, 6)

        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "games")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
    }

    function test_hostile_envelopes_preserve_safe_state_and_provider_is_inert() {
        applicationWindow.width = 920
        applicationWindow.height = 600
        activate(object("homeGamesButton"))
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        const controller = applicationWindow.gameController
        const manifest = clone(controller.catalog[0])
        manifest.max_human_players = 9
        verify(!controller._validManifest(manifest))

        verify(controller.openSession(controller.sessions[0]))
        tryCompare(controller, "loadState", "ready", 5000)
        const safeSession = clone(controller.selectedSession)
        const wrongParticipants = clone(safeSession)
        wrongParticipants.participants.push({
            "seat": 1,
            "persona": clone(wrongParticipants.participants[0].persona)
        })
        verify(!controller._validSession(wrongParticipants))

        const malformedState = clone(safeSession)
        malformedState.state.untrusted_extra = true
        verify(!controller._validSession(malformedState))

        const hostileBody = clone(safeSession)
        hostileBody.participants = wrongParticipants.participants
        controller._expectedGeneration = 901
        controller._expectedOperation = "player_game_detail"
        controller._handleFinished(901, "player_game_detail", 200,
                                   JSON.stringify(hostileBody), "")
        compare(controller.loadState, "error")
        compare(controller.selectedSession.id, safeSession.id)
        compare(controller.selectedSession.revision, safeSession.revision)

        const providerSession = clone(safeSession)
        providerSession.game_key = "door_legends"
        providerSession.authority = "registered_provider"
        providerSession.provider_release_id = "55555555-5555-4555-8555-555555555555"
        providerSession.availability = "unavailable"
        providerSession.state = {}
        verify(controller._validSession(providerSession))
        controller.selectedSession = providerSession
        controller._derivePresentation()
        verify(!controller.presentation.supported)
        verify(!controller.presentation.can_act)

        verify(applicationWindow.onboardingController.showPlayerScreen("home"))
        tryCompare(applicationWindow.onboardingController, "state", "home")
        activate(object("homeChallengesButton"))
        tryCompare(controller, "loadState", "ready", 5000)
        const selfChallenge = clone(controller.challenges[0])
        selfChallenge.challenger = clone(selfChallenge.challenged)
        verify(!controller._validChallenge(selfChallenge))
    }

    function test_transport_uncertainty_retains_exact_mutation_identity() {
        activate(object("homeGamesButton"))
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        const controller = applicationWindow.gameController
        verify(controller.openSession(controller.sessions[0]))
        tryCompare(controller, "loadState", "ready", 5000)
        const pending = {
            "operation": "player_game_command",
            "method": "POST",
            "path": "/v1/personas/" + controller.actor.id + "/game-sessions/"
                    + controller.selectedSession.id + "/commands",
            "document": {
                "idempotency_key": "77777777-7777-4777-8777-777777777777",
                "expected_revision": controller.selectedSession.revision,
                "command": {"kind": "play", "action": "strike"}
            }
        }
        controller._pendingMutation = pending
        controller._expectedGeneration = 902
        controller._expectedOperation = pending.operation
        controller._handleFinished(902, pending.operation, 0, "", "timeout")
        verify(controller.hasRetryableMutation)
        compare(JSON.stringify(controller._pendingMutation), JSON.stringify(pending))
        compare(controller.loadState, "error")
    }

    function test_revision_conflict_refetches_authoritative_session() {
        activate(object("homeGamesButton"))
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        const controller = applicationWindow.gameController
        verify(controller.openSession(controller.sessions[0]))
        tryCompare(controller, "loadState", "ready", 5000)
        activate(object("gameStrikeButton"))
        tryVerify(function() {
            return !controller.busy && controller.selectedSession.revision === 1
        }, 5000)
        controller.selectedSession = Object.assign({}, controller.selectedSession,
                                                   {"revision": 0})
        controller._derivePresentation()
        verify(controller.submitAction("charge"))
        tryVerify(function() {
            return !controller.busy && controller.selectedSession.revision === 1
                    && controller.loadState === "ready"
        }, 5000, controller.statusText + " // " + controller.errorText)
    }

    function test_valid_invalid_session_response_clears_player_authority() {
        activate(object("homeGamesButton"))
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        const controller = applicationWindow.gameController
        controller._expectedGeneration = 903
        controller._expectedOperation = "player_games_sessions"
        controller._handleFinished(903, "player_games_sessions", 401,
                                   JSON.stringify({
                                       "error": {
                                           "code": "invalid_session",
                                           "message": "device session is invalid"
                                       }
                                   }), "")
        tryCompare(applicationWindow.onboardingController, "state", "access")
        verify(!applicationWindow.onboardingController.hasSession)
        compare(controller.actor, null)
        compare(controller.selectedSession, null)
        compare(controller.sessions.length, 0)
    }

    function test_challenge_create_cancel_and_decline() {
        activate(object("homeChallengesButton"))
        tryCompare(applicationWindow.onboardingController, "state", "challenges")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        compare(applicationWindow.gameController.connections.length, 1)
        compare(applicationWindow.gameController.challenges.length, 1)

        verify(applicationWindow.gameController.createChallenge(
                   applicationWindow.gameController.connections[0],
                   applicationWindow.gameController.challengeGames()[0]))
        tryVerify(function() {
            return !applicationWindow.gameController.busy
                    && findChallenge("outgoing", "pending") !== null
        }, 5000)
        verify(applicationWindow.gameController.cancelChallenge(
                   findChallenge("outgoing", "pending")))
        tryVerify(function() {
            return !applicationWindow.gameController.busy
                    && findChallenge("outgoing", "cancelled") !== null
        }, 5000)

        verify(applicationWindow.gameController.declineChallenge(
                   findChallenge("incoming", "pending")))
        tryVerify(function() {
            return !applicationWindow.gameController.busy
                    && findChallenge("incoming", "declined") !== null
        }, 5000)
    }

    function test_accept_opens_versus_and_enforces_active_seat() {
        activate(object("homeChallengesButton"))
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        verify(applicationWindow.gameController.acceptChallenge(
                   findChallenge("incoming", "pending")))
        tryCompare(applicationWindow.onboardingController, "state", "gameplay", 5000)
        tryVerify(function() {
            return !applicationWindow.gameController.busy
                    && applicationWindow.gameController.selectedSession !== null
        }, 5000, applicationWindow.gameController.statusText + " // "
                 + applicationWindow.gameController.errorText)
        compare(applicationWindow.gameController.selectedSession.game_version, 2)
        verify(applicationWindow.gameController.presentation.can_act)
        activate(object("gameGuardButton"))
        tryVerify(function() {
            return !applicationWindow.gameController.busy
                    && applicationWindow.gameController.selectedSession.revision === 2
        }, 5000)
        verify(!applicationWindow.gameController.presentation.can_act)
        verify(!applicationWindow.gameController.submitAction("charge"))
    }
}
