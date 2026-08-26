import QtQuick
import QtTest
import "../.." as App

TestCase {
    id: testCase
    name: "OnboardingUi"
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

    function initTestCase() {
        fixtureConfig = loadConfig()
        verify(fixtureConfig.server_url !== "")
    }

    function init() {
        applicationWindow = createTemporaryObject(mainComponent, testCase)
        verify(applicationWindow !== null)
        applicationWindow.width = 920
        applicationWindow.height = 600
        applicationWindow.requestActivate()
        tryVerify(function() { return applicationWindow.active }, 3000)
        applicationWindow.onboardingController.connectToServer(fixtureConfig.server_url)
        tryCompare(applicationWindow.onboardingController, "state", "access", 5000)
        compare(applicationWindow.onboardingController.serverUrl, fixtureConfig.server_url)
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
        applicationWindow.requestActivate()
        tryVerify(function() { return applicationWindow.active })
        field.forceActiveFocus()
        tryVerify(function() { return field.activeFocus })
        field.selectAll()
        for (let index = 0; index < value.length; index++)
            keyClick(value.charAt(index))
        compare(field.text, value)
    }

    function activate(control) {
        applicationWindow.requestActivate()
        tryVerify(function() { return applicationWindow.active })
        control.forceActiveFocus()
        verify(control.activeFocus)
        keyClick(Qt.Key_Return)
    }

    function test_register_login_create_logout_with_keyboard() {
        const registerMode = object("registerModeButton")
        verify(registerMode.Accessible.name.length > 0)
        activate(registerMode)
        tryCompare(applicationWindow.onboardingController, "accessMode", "register")

        const inviteCode = object("inviteCodeField")
        let username = object("usernameField")
        let password = object("passwordField")
        tryVerify(function() { return inviteCode.activeFocus }, 1000,
                  "registration mode must finish its deferred focus handoff before typing")
        verify(inviteCode.Accessible.name.length > 0)
        compare(inviteCode.echoMode, TextInput.Password)
        verify(username.Accessible.name.length > 0)
        verify(password.Accessible.name.length > 0)
        compare(password.echoMode, TextInput.Password)
        enterText(inviteCode, "ogsi_" + "I".repeat(43))
        enterText(username, "New_User")
        enterText(password, "TEST-ONLY-registration-passphrase")
        password.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(inviteCode.text, "")
        compare(password.text, "")
        tryCompare(applicationWindow.onboardingController, "accessMode", "sign_in", 5000)
        tryCompare(username, "text", "new_user", 5000)

        password = object("passwordField")
        enterText(password, "TEST-ONLY-registration-passphrase")
        password.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(password.text, "")
        tryCompare(applicationWindow.onboardingController, "state", "personas", 5000)
        verify(applicationWindow.onboardingController.hasSession)

        const handle = object("personaHandleField")
        const displayName = object("personaDisplayNameField")
        const status = object("personaStatusField")
        const bio = object("personaBioField")
        bio.text = "x".repeat(1001)
        compare(bio.text.length, 1000)
        enterText(handle, "New_Hero")
        enterText(displayName, "New Hero")
        enterText(status, "Ready")
        enterText(bio, "Keyboard-created fixture persona")
        activate(object("createPersonaButton"))
        tryCompare(applicationWindow.onboardingController, "state", "home", 5000)
        compare(applicationWindow.onboardingController.selectedPersona.handle, "new_hero")

        activate(object("homeLogoutButton"))
        tryCompare(applicationWindow.onboardingController, "state", "access")
        verify(!applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona, null)
    }

    function test_mfa_retry_cancel_and_owned_selection() {
        let username = object("usernameField")
        let password = object("passwordField")
        enterText(username, "mfa_user")
        enterText(password, "TEST-ONLY-registration-passphrase")
        password.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(password.text, "")
        tryCompare(applicationWindow.onboardingController, "state", "mfa", 5000)
        verify(applicationWindow.onboardingController.hasMfaChallenge)

        let factor = object("mfaFactorField")
        compare(factor.echoMode, TextInput.Password)
        enterText(factor, "000000")
        factor.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(factor.text, "")
        tryVerify(function() {
            return applicationWindow.onboardingController.errorText.indexOf("not accepted") !== -1
        }, 5000)
        compare(applicationWindow.onboardingController.state, "mfa")
        verify(applicationWindow.onboardingController.hasMfaChallenge)

        factor = object("mfaFactorField")
        enterText(factor, "OGS-TEST-RECOVERY")
        factor.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(factor.text, "")
        tryCompare(applicationWindow.onboardingController, "state", "personas", 5000)
        verify(!applicationWindow.onboardingController.hasMfaChallenge)
        compare(applicationWindow.onboardingController.personas.length, 2)

        const personaList = object("personaList")
        tryVerify(function() { return personaList.itemAtIndex(0) !== null })
        const firstPersona = personaList.itemAtIndex(0)
        verify(firstPersona.Accessible.name.indexOf("MFA One") !== -1)
        activate(firstPersona)
        tryCompare(applicationWindow.onboardingController, "state", "home")
        compare(applicationWindow.onboardingController.selectedPersona.id,
                "33333333-3333-4333-8333-333333333333")
    }

    function test_escape_focus_and_minimum_layout() {
        applicationWindow.width = 640
        applicationWindow.height = 420
        tryCompare(applicationWindow, "width", 640)
        tryCompare(applicationWindow, "height", 420)

        const username = object("usernameField")
        const password = object("passwordField")
        username.forceActiveFocus()
        applicationWindow.requestActivate()
        verify(username.activeFocus)
        keyClick(Qt.Key_Tab)
        tryVerify(function() { return password.activeFocus })
        keyClick(Qt.Key_Tab, Qt.ShiftModifier)
        tryVerify(function() { return username.activeFocus })

        activate(object("registerModeButton"))
        tryCompare(applicationWindow.onboardingController, "accessMode", "register")
        const inviteCode = object("inviteCodeField")
        tryVerify(function() { return inviteCode.activeFocus })
        enterText(inviteCode, "ogsi_" + "I".repeat(43))
        enterText(password, "TEST-ONLY-registration-passphrase")
        inviteCode.forceActiveFocus()
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "accessMode", "sign_in")
        compare(inviteCode.text, "")
        compare(password.text, "")

        username.forceActiveFocus()
        applicationWindow.requestActivate()
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "connection")
        const endpoint = object("serverUrlField")
        verify(endpoint.activeFocus)
        verify(endpoint.Accessible.name.length > 0)
        enterText(endpoint, "http://example.com")
        endpoint.forceActiveFocus()
        applicationWindow.requestActivate()
        keyClick(Qt.Key_Return)
        tryCompare(applicationWindow.onboardingController, "connectionState", "configuration_error")
        verify(applicationWindow.onboardingController.errorText.indexOf("HTTPS") !== -1)
    }

    function test_mfa_escape_clears_challenge() {
        enterText(object("usernameField"), "mfa_user")
        const password = object("passwordField")
        enterText(password, "TEST-ONLY-registration-passphrase")
        password.forceActiveFocus()
        applicationWindow.requestActivate()
        keyClick(Qt.Key_Return)
        tryCompare(applicationWindow.onboardingController, "state", "mfa", 5000)
        const factor = object("mfaFactorField")
        enterText(factor, "123456")
        factor.forceActiveFocus()
        applicationWindow.requestActivate()
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "access")
        verify(!applicationWindow.onboardingController.hasMfaChallenge)
    }
}
