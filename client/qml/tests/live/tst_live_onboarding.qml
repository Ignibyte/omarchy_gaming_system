import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "LiveOnboarding"
    property var liveConfig: ({})

    Component { id: controllerComponent; App.OnboardingController {} }
    Component { id: socialComponent; App.SocialController {} }

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
        const personaHandle = liveConfig.persona_handle || ""
        const factor = liveConfig.factor || ""
        const peerHandle = liveConfig.peer_handle || ""
        const messageBody = liveConfig.message_body || ""
        verify(serverUrl !== "")
        verify(username !== "")
        verify(password !== "")

        const controller = createTemporaryObject(controllerComponent, testCase)
        verify(controller !== null)
        controller.initialize(serverUrl)
        tryCompare(controller, "state", "access", 10000)

        if (scenario === "register") {
            verify(personaHandle !== "")
            controller.chooseAccessMode("register")
            verify(controller.registerAccount(username, password))
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
