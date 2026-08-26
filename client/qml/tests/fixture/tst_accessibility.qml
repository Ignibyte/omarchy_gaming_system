import QtQuick
import QtTest
import "../.." as App
import "../../components" as Components

TestCase {
    id: testCase
    name: "AccessibilityUi"
    when: windowShown

    property var applicationWindow: null
    property var fixtureConfig: ({})

    Component {
        id: mainComponent
        App.Main { visible: true }
    }

    Component {
        id: themeComponent
        Components.OgsTheme { }
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
        verify(fixtureConfig.server_url !== "")
    }

    function init() {
        resetFixture()
        applicationWindow = createTemporaryObject(mainComponent, testCase)
        verify(applicationWindow !== null)
        applicationWindow.width = 920
        applicationWindow.height = 600
        applicationWindow.requestActivate()
        tryVerify(function() { return applicationWindow.active }, 3000)
        applicationWindow.onboardingController.showServerConfiguration()
        applicationWindow.onboardingController.connectToServer(fixtureConfig.server_url)
        tryCompare(applicationWindow.onboardingController, "state", "access", 5000)
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
        applicationWindow.requestActivate()
        field.forceActiveFocus()
        tryVerify(function() { return field.activeFocus })
        field.selectAll()
        for (let index = 0; index < value.length; index++)
            keyClick(value.charAt(index))
        compare(field.text, value)
    }

    function activate(control) {
        applicationWindow.requestActivate()
        control.forceActiveFocus()
        tryVerify(function() { return control.activeFocus })
        keyClick(Qt.Key_Return)
    }

    function signIn(username, password) {
        enterText(object("usernameField"), username)
        const passwordField = object("passwordField")
        enterText(passwordField, password)
        passwordField.forceActiveFocus()
        keyClick(Qt.Key_Return)
    }

    function signInAndSelectActor() {
        signIn("social_user", "TEST-ONLY-social-passphrase")
        tryCompare(applicationWindow.onboardingController, "state", "personas", 5000)
        const personaList = object("personaList")
        tryVerify(function() { return personaList.itemAtIndex(0) !== null })
        activate(personaList.itemAtIndex(0))
        tryCompare(applicationWindow.onboardingController, "state", "home")
    }

    function assertHorizontalBounds(item, context) {
        const position = item.mapToItem(applicationWindow.contentItem, 0, 0)
        verify(position.x >= -1, context + " begins outside the window at " + position.x)
        verify(position.x + item.width <= applicationWindow.contentItem.width + 1,
               context + " exceeds the window width at " + (position.x + item.width))
    }

    function assertShellExit() {
        const exitButton = object("shellExitButton")
        verify(exitButton.visible, "shell exit control must remain visible")
        verify(exitButton.enabled, "shell exit control must remain enabled")
        compare(exitButton.Accessible.role, Accessible.Button)
        compare(exitButton.Accessible.name, "Close Omarchy Gaming System")
        verify(exitButton.Accessible.description.indexOf("without signing out") !== -1)
        assertHorizontalBounds(exitButton, "shell exit control")
    }

    function assertScreen(key, initialFocusName, suppliedInitialFocus) {
        const heading = waitForObject(key + "Heading")
        const status = object(key + "StatusBanner")
        const navigation = object(key + "NavigationHint")
        const initialFocus = suppliedInitialFocus || waitForObject(initialFocusName)

        assertShellExit()

        verify(heading.text.length > 0, key + " requires a visible heading")
        compare(heading.Accessible.role, Accessible.Heading)
        compare(heading.Accessible.name, heading.text)
        verify(status.visible, key + " requires a visible state message")
        compare(status.Accessible.role, Accessible.StatusBar)
        verify(status.Accessible.name.indexOf(": ") > 0,
               key + " state must retain a non-color semantic prefix")
        verify(navigation.visible, key + " requires keyboard navigation context")
        verify(navigation.Accessible.name.indexOf("Navigation: ") === 0)
        verify(initialFocus.enabled, key + " initial focus target must be enabled")
        tryVerify(function() { return initialFocus.activeFocus }, 1000,
                  key + " must restore deterministic initial focus; active item is "
                  + (applicationWindow.activeFocusItem
                     ? applicationWindow.activeFocusItem.objectName : "none")
                  + (key === "personas"
                     ? "; createMode=" + object("screenLoader").item.createMode
                       + "; list count=" + object("personaList").count
                       + "; current=" + object("personaList").currentIndex
                     : ""))
        assertHorizontalBounds(heading, key + " heading")
        assertHorizontalBounds(status, key + " state message")
        assertHorizontalBounds(initialFocus, key + " initial control")
        const headingPosition = heading.mapToItem(applicationWindow.contentItem, 0, 0)
        const statusPosition = status.mapToItem(applicationWindow.contentItem, 0, 0)
        verify(headingPosition.y + heading.height <= statusPosition.y + 1,
               key + " heading and state message must not overlap")
    }

    function assertTabRoundTrip(first) {
        const original = first
        tryVerify(function() { return original.activeFocus })
        keyClick(Qt.Key_Tab)
        const next = applicationWindow.activeFocusItem
        verify(next !== null && next !== original && next.enabled,
               "Tab must reach another enabled control")
        keyClick(Qt.Key_Tab, Qt.ShiftModifier)
        tryVerify(function() { return original.activeFocus }, 1000,
                  "reverse traversal must return to the prior control")
    }

    function linearChannel(channel) {
        return channel <= 0.04045 ? channel / 12.92
                                  : Math.pow((channel + 0.055) / 1.055, 2.4)
    }

    function luminance(color) {
        return 0.2126 * linearChannel(color.r)
             + 0.7152 * linearChannel(color.g)
             + 0.0722 * linearChannel(color.b)
    }

    function contrast(foreground, background) {
        const lighter = Math.max(luminance(foreground), luminance(background))
        const darker = Math.min(luminance(foreground), luminance(background))
        return (lighter + 0.05) / (darker + 0.05)
    }

    function requireContrast(foreground, background, minimum, label) {
        const ratio = contrast(foreground, background)
        verify(ratio >= minimum, label + " contrast " + ratio.toFixed(2)
               + " is below " + minimum.toFixed(1))
    }

    function test_theme_contrast_contract() {
        const theme = createTemporaryObject(themeComponent, testCase)
        verify(theme !== null)
        const textSurfaces = [theme.background, theme.surface, theme.surfaceRaised]
        for (let index = 0; index < textSurfaces.length; index++) {
            requireContrast(theme.textPrimary, textSurfaces[index], 4.5,
                            "primary text on surface " + index)
            requireContrast(theme.textSecondary, textSurfaces[index], 4.5,
                            "secondary text on surface " + index)
            requireContrast(theme.textMuted, textSurfaces[index], 4.5,
                            "muted text on surface " + index)
            requireContrast(theme.focus, textSurfaces[index], 3.0,
                            "focus indicator on surface " + index)
        }
        requireContrast(theme.accent, theme.background, 3.0, "success indicator")
        requireContrast(theme.warning, theme.background, 3.0, "warning indicator")
        requireContrast(theme.danger, theme.background, 3.0, "error indicator")
        requireContrast(theme.border, theme.surfaceRaised, 3.0, "control boundary")
    }

    function test_exit_button_keyboard_closes_without_logout() {
        signInAndSelectActor()
        const actorId = applicationWindow.onboardingController.selectedPersona.id
        const exitButton = object("shellExitButton")
        activate(exitButton)
        tryCompare(applicationWindow, "visible", false)
        verify(applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona.id, actorId)
    }

    function test_exit_button_pointer_closes_window() {
        const exitButton = object("shellExitButton")
        applicationWindow.requestActivate()
        tryVerify(function() { return applicationWindow.active })
        mouseClick(exitButton, exitButton.width / 2, exitButton.height / 2,
                   Qt.LeftButton)
        tryCompare(applicationWindow, "visible", false)
    }

    function test_public_flow_semantics_focus_and_compact_layout() {
        applicationWindow.width = 640
        applicationWindow.height = 420
        tryCompare(applicationWindow, "width", 640)
        tryCompare(applicationWindow, "height", 420)

        assertScreen("access", "usernameField")
        assertTabRoundTrip(object("usernameField"))

        applicationWindow.onboardingController.showServerConfiguration()
        tryCompare(applicationWindow.onboardingController, "state", "connection")
        assertScreen("connection", "serverUrlField")

        applicationWindow.onboardingController.connectToServer(fixtureConfig.server_url)
        tryCompare(applicationWindow.onboardingController, "state", "access", 5000)
        assertScreen("access", "usernameField")

        signIn("mfa_user", "TEST-ONLY-registration-passphrase")
        tryCompare(applicationWindow.onboardingController, "state", "mfa", 5000)
        assertScreen("mfa", "mfaFactorField")
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "access")
        verify(!applicationWindow.onboardingController.hasMfaChallenge)

        signIn("social_user", "TEST-ONLY-social-passphrase")
        tryCompare(applicationWindow.onboardingController, "state", "personas", 5000)
        const personaList = object("personaList")
        tryVerify(function() { return personaList.itemAtIndex(0) !== null })
        assertScreen("personas", "", personaList.itemAtIndex(0))
        assertTabRoundTrip(personaList.itemAtIndex(0))
    }

    function test_authenticated_keyboard_flow_escape_and_compact_layout() {
        signInAndSelectActor()
        applicationWindow.width = 640
        applicationWindow.height = 420
        const actorId = applicationWindow.onboardingController.selectedPersona.id

        assertScreen("home", "homeGamesButton")
        assertTabRoundTrip(object("homeGamesButton"))

        activate(object("homeSocialButton"))
        tryCompare(applicationWindow.onboardingController, "state", "social")
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)
        assertScreen("social", "socialHandleField")
        assertTabRoundTrip(object("socialHandleField"))
        verify(object("reportHandleField").Accessible.name.length > 0)
        verify(object("reportCategoryBox").Accessible.name.length > 0)
        verify(object("reportDetailField").Accessible.name.length > 0)
        verify(object("reportSubmitButton").Accessible.name.length > 0)
        assertHorizontalBounds(object("reportHandleField"), "report subject field")
        assertHorizontalBounds(object("reportCategoryBox"), "report category")
        assertHorizontalBounds(object("reportDetailField"), "report details")
        assertHorizontalBounds(object("reportSubmitButton"), "report submit control")
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "home")
        verify(applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona.id, actorId)

        activate(object("homeInboxButton"))
        tryCompare(applicationWindow.onboardingController, "state", "inbox")
        tryCompare(applicationWindow.socialController, "loadState", "ready", 5000)
        assertScreen("inbox", "inboxRefreshButton")
        assertTabRoundTrip(object("inboxRefreshButton"))
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "home")
        verify(applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona.id, actorId)

        activate(object("homeGamesButton"))
        tryCompare(applicationWindow.onboardingController, "state", "games")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        assertScreen("games", "gamesRefreshButton")
        assertTabRoundTrip(object("gamesRefreshButton"))

        activate(object("gamesChallengesButton"))
        tryCompare(applicationWindow.onboardingController, "state", "challenges")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        assertScreen("challenges", "challengesRefreshButton")
        assertTabRoundTrip(object("challengesRefreshButton"))
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "games")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        verify(applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona.id, actorId)

        verify(applicationWindow.gameController.sessions.length > 0)
        verify(applicationWindow.gameController.openSession(
                   applicationWindow.gameController.sessions[0]))
        tryCompare(applicationWindow.onboardingController, "state", "gameplay")
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        assertScreen("gameplay", "gameStrikeButton")
        assertTabRoundTrip(object("gameStrikeButton"))
        keyClick(Qt.Key_Escape)
        tryCompare(applicationWindow.onboardingController, "state", "games")
        verify(applicationWindow.onboardingController.hasSession)
        compare(applicationWindow.onboardingController.selectedPersona.id, actorId)

        applicationWindow.width = 920
        applicationWindow.height = 600
        tryCompare(applicationWindow, "width", 920)
        tryCompare(applicationWindow, "height", 600)
        tryCompare(applicationWindow.gameController, "loadState", "ready", 5000)
        assertScreen("games", "gamesRefreshButton")
    }
}
