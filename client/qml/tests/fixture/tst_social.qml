import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "SocialInboxUi"
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

    function waitForObject(name) {
        tryVerify(function() { return findChild(applicationWindow, name) !== null }, 3000,
                  "missing delayed QML object " + name)
        return findChild(applicationWindow, name)
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
        compare(applicationWindow.onboardingController.selectedPersona.handle, "social_actor")
    }

    function test_keyboard_social_inventory_and_actions() {
        applicationWindow.width = 640
        applicationWindow.height = 420
        activate(object("homeSocialButton"))
        tryCompare(applicationWindow.onboardingController, "state", "social")
        tryVerify(function() {
            return applicationWindow.socialController.loadState === "ready"
        }, 5000, applicationWindow.socialController.statusText + " // "
                 + applicationWindow.socialController.errorText)
        compare(applicationWindow.socialController.incomingRequests.length, 1)
        compare(applicationWindow.socialController.outgoingRequests.length, 1)
        compare(applicationWindow.socialController.connections.length, 1)
        compare(applicationWindow.socialController.blocks.length, 1)

        const handle = object("socialHandleField")
        verify(handle.Accessible.name.length > 0)
        enterText(handle, "x")
        activate(object("socialRequestButton"))
        verify(applicationWindow.socialController.errorText.indexOf("3–24") !== -1)

        enterText(handle, "social_actor")
        activate(object("socialRequestButton"))
        verify(applicationWindow.socialController.errorText.indexOf("another") !== -1)

        const incomingList = object("incomingRequestList")
        tryVerify(function() { return incomingList.itemAtIndex(0) !== null }, 3000)
        const acceptButton = findChild(incomingList.itemAtIndex(0), "socialPrimaryButton")
        verify(acceptButton !== null)
        verify(acceptButton.Accessible.name.indexOf("ACCEPT") !== -1)
        activate(acceptButton)
        tryVerify(function() {
            return applicationWindow.socialController.loadState === "ready"
        }, 5000, applicationWindow.socialController.statusText + " // "
                 + applicationWindow.socialController.errorText)
        compare(applicationWindow.socialController.incomingRequests.length, 0)
        compare(applicationWindow.socialController.connections.length, 2)

        verify(applicationWindow.socialController.removeRelationship(
                   applicationWindow.socialController.outgoingRequests[0].persona))
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)
        compare(applicationWindow.socialController.outgoingRequests.length, 0)

        verify(applicationWindow.socialController.unblockPersona(
                   applicationWindow.socialController.blocks[0].persona))
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)
        compare(applicationWindow.socialController.blocks.length, 0)

        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "home")
        verify(object("homeSocialButton").activeFocus)
    }

    function test_keyboard_inbox_history_send_and_read() {
        activate(object("homeInboxButton"))
        tryCompare(applicationWindow.onboardingController, "state", "inbox")
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)
        compare(applicationWindow.socialController.conversations.length, 1)
        compare(applicationWindow.socialController.conversations[0].unread_count, 2)

        const conversationList = object("conversationList")
        tryVerify(function() { return conversationList.itemAtIndex(0) !== null }, 3000)
        const conversationButton = conversationList.itemAtIndex(0)
        verify(conversationButton.Accessible.name.indexOf("2 unread") !== -1)
        activate(conversationButton)
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)
        tryVerify(function() {
            return !applicationWindow.socialController.busy
                    && applicationWindow.socialController.selectedConversation.unread_count === 0
        }, 5000)
        compare(applicationWindow.socialController.messages.length, 2)
        compare(applicationWindow.socialController.messageText(
                    applicationWindow.socialController.messages[0]),
                "@social_friend: Fixture hello <b>as plain text</b>")

        activate(object("loadOlderMessagesButton"))
        tryVerify(function() { return !applicationWindow.socialController.busy }, 5000)
        compare(applicationWindow.socialController.messages.length, 3)
        compare(applicationWindow.socialController.messages[0].sequence, 1)
        compare(applicationWindow.socialController.nextBefore, null)

        const composer = object("messageComposer")
        enterText(composer, "Reply from keyboard")
        activate(object("sendMessageButton"))
        compare(composer.text, "")
        tryVerify(function() { return !applicationWindow.socialController.busy }, 5000)
        compare(applicationWindow.socialController.messages.length, 4)
        compare(applicationWindow.socialController.messages[3].body, "Reply from keyboard")

        keyClick(Qt.Key_Escape)
        compare(applicationWindow.socialController.selectedConversation, null)
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "home")
    }

    function test_z_hostile_schema_size_and_invalid_session_fail_safe() {
        activate(object("homeSocialButton"))
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)

        enterText(object("socialHandleField"), "malformed_peer")
        activate(object("socialRequestButton"))
        tryCompare(applicationWindow.socialController, "loadState", "error", 5000)
        verify(applicationWindow.onboardingController.hasSession)

        enterText(object("socialHandleField"), "oversized_peer")
        activate(object("socialRequestButton"))
        tryVerify(function() {
            return applicationWindow.socialController.errorText.indexOf("limit") !== -1
        }, 5000)
        verify(applicationWindow.onboardingController.hasSession)

        enterText(object("socialHandleField"), "session_lost")
        activate(object("socialRequestButton"))
        tryCompare(applicationWindow.onboardingController, "state", "access", 5000)
        verify(!applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona, null)
        compare(applicationWindow.socialController.incomingRequests.length, 0)
    }
}
