import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "LiveOnboarding"
    property var liveConfig: ({})

    Component { id: controllerComponent; App.OnboardingController {} }
    Component { id: socialComponent; App.SocialController {} }
    Component { id: gameComponent; App.GameController {} }

    function loadConfig() {
        const request = new XMLHttpRequest()
        request.open("GET", Qt.resolvedUrl("../../../../.dev/qml-onboarding/live-config.json"), false)
        request.send()
        return JSON.parse(request.responseText)
    }

    function initTestCase() {
        liveConfig = loadConfig()
        verify(liveConfig.server_url !== "")
    }

    function test_real_migrated_api() {
        const serverUrl = liveConfig.server_url
        const scenario = liveConfig.scenario
        const username = liveConfig.username
        const password = liveConfig.password
        const inviteCode = liveConfig.invite_code || ""
        const personaHandle = liveConfig.persona_handle || ""
        const factor = liveConfig.factor || ""
        const peerHandle = liveConfig.peer_handle || ""
        const messageBody = liveConfig.message_body || ""
        const peerUsername = liveConfig.peer_username || ""
        const peerPassword = liveConfig.peer_password || ""
        verify(serverUrl !== "")
        verify(username !== "")
        verify(password !== "")

        const controller = createTemporaryObject(controllerComponent, testCase)
        verify(controller !== null)
        controller.initialize(serverUrl)
        tryCompare(controller, "state", "access", 10000)

        if (scenario === "register") {
            verify(personaHandle !== "")
            verify(inviteCode !== "")
            controller.chooseAccessMode("register")
            verify(controller.registerAccount(inviteCode, username, password))
            tryVerify(function() {
                return !controller.busy && controller.accessMode === "sign_in"
                        && controller.suggestedUsername === username
            }, 10000)
            verify(controller.signIn(username, password, "QML live smoke"))
            tryCompare(controller, "state", "personas", 10000)
            compare(controller.personas.length, 0)
            verify(controller.createPersona(personaHandle, "QML Live Player",
                                             "Created by the real QML smoke", "Ready"))
            tryCompare(controller, "state", "home", 10000)
            compare(controller.selectedPersona.handle, personaHandle)
        } else if (scenario === "mfa") {
            verify(factor !== "")
            verify(controller.signIn(username, password, "QML MFA smoke"))
            tryCompare(controller, "state", "mfa", 10000)
            verify(controller.hasMfaChallenge)
            verify(controller.completeMfa(factor))
            tryCompare(controller, "state", "personas", 10000)
            verify(controller.personas.length > 0)
            verify(controller.selectPersona(controller.personas[0]))
            compare(controller.state, "home")
            verify(!controller.hasMfaChallenge)
        } else if (scenario === "social") {
            verify(personaHandle !== "")
            verify(peerHandle !== "")
            verify(messageBody !== "")
            verify(controller.signIn(username, password, "QML social smoke"))
            tryCompare(controller, "state", "personas", 10000)
            let selected = null
            for (let index = 0; index < controller.personas.length; index++) {
                if (controller.personas[index].handle === personaHandle) {
                    selected = controller.personas[index]
                    break
                }
            }
            verify(selected !== null)
            verify(controller.selectPersona(selected))

            const social = createTemporaryObject(socialComponent, testCase, {
                "sessionController": controller,
                "actor": controller.selectedPersona
            })
            verify(social !== null)
            verify(controller.showPlayerScreen("social"))
            verify(social.refreshSocial())
            tryVerify(function() { return !social.busy && social.loadState === "ready" }, 10000)
            verify(social.connections.some(function(connection) {
                return connection.persona.handle === peerHandle
            }))
            verify(social.reportPersonaByHandle(
                       peerHandle, "other", "QML live operator report"))
            tryVerify(function() {
                return !social.busy && social.loadState === "ready"
                        && social.statusText === "Report submitted for operator review."
            }, 10000, social.statusText + " // " + social.errorText)

            verify(controller.showPlayerScreen("inbox"))
            verify(social.refreshInbox())
            tryVerify(function() { return !social.busy && social.loadState === "ready" }, 10000)
            let conversation = null
            for (let index = 0; index < social.conversations.length; index++) {
                if (social.conversations[index].other_persona.handle === peerHandle) {
                    conversation = social.conversations[index]
                    break
                }
            }
            verify(conversation !== null)
            verify(social.openConversation(conversation))
            tryVerify(function() { return !social.busy && social.loadState === "ready" }, 10000)
            verify(social.messages.length >= 2)
            verify(social.sendMessage(messageBody))
            tryVerify(function() { return !social.busy && social.loadState === "ready" }, 10000)
            compare(social.messages[social.messages.length - 1].body, messageBody)
            compare(social.messages[social.messages.length - 1].sender.id, selected.id)
            social.destroy()
        } else if (scenario === "games") {
            verify(personaHandle !== "")
            verify(peerHandle !== "")
            verify(peerUsername !== "")
            verify(peerPassword !== "")
            verify(controller.signIn(username, password, "QML game challenger"))
            tryCompare(controller, "state", "personas", 10000)
            let challengerPersona = null
            for (let challengerIndex = 0; challengerIndex < controller.personas.length;
                 challengerIndex++) {
                if (controller.personas[challengerIndex].handle === personaHandle) {
                    challengerPersona = controller.personas[challengerIndex]
                    break
                }
            }
            verify(challengerPersona !== null)
            verify(controller.selectPersona(challengerPersona))

            const peerController = createTemporaryObject(controllerComponent, testCase)
            verify(peerController !== null)
            peerController.initialize(serverUrl)
            tryCompare(peerController, "state", "access", 10000)
            verify(peerController.signIn(peerUsername, peerPassword, "QML game challenged"))
            tryCompare(peerController, "state", "personas", 10000)
            let challengedPersona = null
            for (let peerIndex = 0; peerIndex < peerController.personas.length; peerIndex++) {
                if (peerController.personas[peerIndex].handle === peerHandle) {
                    challengedPersona = peerController.personas[peerIndex]
                    break
                }
            }
            verify(challengedPersona !== null)
            verify(peerController.selectPersona(challengedPersona))

            const challengerGame = createTemporaryObject(gameComponent, testCase, {
                "sessionController": controller,
                "actor": challengerPersona
            })
            const challengedGame = createTemporaryObject(gameComponent, testCase, {
                "sessionController": peerController,
                "actor": challengedPersona
            })
            verify(challengerGame !== null)
            verify(challengedGame !== null)

            verify(controller.showPlayerScreen("challenges"))
            verify(challengerGame.refreshChallenges())
            tryVerify(function() {
                return !challengerGame.busy && challengerGame.loadState === "ready"
            }, 10000, challengerGame.statusText + " // " + challengerGame.errorText)
            let connection = null
            for (let connectionIndex = 0; connectionIndex < challengerGame.connections.length;
                 connectionIndex++) {
                if (challengerGame.connections[connectionIndex].persona.id === challengedPersona.id) {
                    connection = challengerGame.connections[connectionIndex]
                    break
                }
            }
            compare(challengerGame.challengeGames().length, 1)
            verify(connection !== null)
            verify(challengerGame.createChallenge(
                       connection, challengerGame.challengeGames()[0]))
            tryVerify(function() {
                return !challengerGame.busy && challengerGame.loadState === "ready"
            }, 10000, challengerGame.statusText + " // " + challengerGame.errorText)

            verify(peerController.showPlayerScreen("challenges"))
            verify(challengedGame.refreshChallenges())
            tryVerify(function() {
                return !challengedGame.busy && challengedGame.loadState === "ready"
            }, 10000, challengedGame.statusText + " // " + challengedGame.errorText)
            let incoming = null
            for (let challengeIndex = 0; challengeIndex < challengedGame.challenges.length;
                 challengeIndex++) {
                const candidate = challengedGame.challenges[challengeIndex]
                if (candidate.direction === "incoming" && candidate.status === "pending"
                        && candidate.challenger.id === challengerPersona.id) {
                    incoming = candidate
                    break
                }
            }
            verify(incoming !== null)
            verify(challengedGame.acceptChallenge(incoming))
            tryVerify(function() {
                return !challengedGame.busy && challengedGame.selectedSession !== null
            }, 10000, challengedGame.statusText + " // " + challengedGame.errorText)
            const gameSessionId = challengedGame.selectedSession.id
            compare(challengedGame.selectedSession.game_version, 2)

            verify(controller.showPlayerScreen("gameplay"))
            verify(challengerGame.openSessionById(gameSessionId))
            tryVerify(function() {
                return !challengerGame.busy && challengerGame.selectedSession !== null
            }, 10000, challengerGame.statusText + " // " + challengerGame.errorText)

            let terminalSession = null
            for (let turn = 0; turn < 24; turn++) {
                const authority = challengerGame.selectedSession.state.active_seat === 0
                        ? challengerGame : challengedGame
                verify(authority.openSessionById(gameSessionId))
                tryVerify(function() {
                    return !authority.busy && authority.selectedSession !== null
                }, 10000, authority.statusText + " // " + authority.errorText)
                if (authority.selectedSession.status === "completed") {
                    terminalSession = authority.selectedSession
                    break
                }
                verify(authority.presentation.can_act)
                const energy = authority.presentation.actor_energy
                verify(authority.submitAction(energy === 0 ? "charge" : "strike"))
                tryVerify(function() {
                    return !authority.busy && authority.loadState === "ready"
                }, 10000, authority.statusText + " // " + authority.errorText)
                if (authority.selectedSession.status === "completed") {
                    terminalSession = authority.selectedSession
                    break
                }
                const other = authority === challengerGame ? challengedGame : challengerGame
                verify(other.openSessionById(gameSessionId))
                tryVerify(function() { return !other.busy && other.loadState === "ready" }, 10000)
            }
            verify(terminalSession !== null)
            compare(terminalSession.status, "completed")
            verify(terminalSession.state.outcome !== null)

            const recoveredGame = createTemporaryObject(gameComponent, testCase, {
                "sessionController": controller,
                "actor": challengerPersona
            })
            verify(recoveredGame !== null)
            verify(recoveredGame.openSessionById(gameSessionId))
            tryVerify(function() {
                return !recoveredGame.busy && recoveredGame.loadState === "ready"
            }, 10000, recoveredGame.statusText + " // " + recoveredGame.errorText)
            compare(recoveredGame.selectedSession.status, "completed")
            compare(recoveredGame.selectedSession.revision, terminalSession.revision)
            compare(JSON.stringify(recoveredGame.selectedSession.state),
                    JSON.stringify(terminalSession.state))
            recoveredGame.destroy()
            challengerGame.destroy()
            challengedGame.destroy()
            peerController.logout()
            peerController.destroy()
        } else {
            fail("unsupported live scenario")
        }

        verify(controller.hasSession)
        verify(controller.selectedPersona !== null)
        controller.logout()
        compare(controller.state, "access")
        verify(!controller.hasSession)
        verify(!controller.hasMfaChallenge)
        compare(controller.selectedPersona, null)
    }
}
