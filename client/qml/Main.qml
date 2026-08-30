import QtQuick
import QtQuick.Controls
import "screens" as Screens
import "components" as Components

ApplicationWindow {
    id: root

    width: 920
    height: 600
    minimumWidth: 640
    minimumHeight: 420
    visible: true
    title: "Omarchy Gaming System"
    color: theme.background

    readonly property alias onboardingController: onboarding
    readonly property alias socialController: social
    readonly property alias gameController: games
    readonly property alias cartridgeController: cartridges
    readonly property alias marketplaceController: marketplace
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") !== -1
    readonly property bool hasShellError: onboarding.errorText !== ""
                                          || social.errorText !== ""
                                          || games.errorText !== ""
                                          || cartridges.errorText !== ""
                                          || marketplace.errorText !== ""
    readonly property bool playerReady: ["home", "social", "inbox", "games",
                                         "challenges", "gameplay"].indexOf(onboarding.state) !== -1
    readonly property string shellStateLabel: hasShellError ? "ERROR"
                                                   : playerReady ? "PLAYER READY" : "SETUP"
    readonly property color shellStateColor: hasShellError ? theme.danger
                                                   : playerReady ? theme.accent : theme.warning
    readonly property var customModuleDisclosure: onboarding.currentServer
            && onboarding.currentServer.operator_custom_modules !== undefined
            ? onboarding.currentServer.operator_custom_modules : null

    Components.OgsTheme { id: theme }

    function argumentValue(prefix, fallback) {
        const argumentsList = Qt.application.arguments
        for (let index = 0; index < argumentsList.length; index++) {
            if (argumentsList[index].startsWith(prefix))
                return argumentsList[index].slice(prefix.length)
        }
        return fallback
    }

    function screenComponent(state) {
        switch (state) {
        case "access": return accessComponent
        case "mfa": return mfaComponent
        case "personas": return personaComponent
        case "home": return homeComponent
        case "social": return socialComponent
        case "inbox": return inboxComponent
        case "games": return gamesComponent
        case "challenges": return challengesComponent
        case "gameplay": return gameplayComponent
        default: return connectionComponent
        }
    }

    OnboardingController {
        id: onboarding
        onStateChanged: {
            if (root.smokeTest && state === "access" && connectionState === "ready")
                Qt.callLater(function() { Qt.exit(0) })
            else if (state === "social")
                Qt.callLater(function() { social.refreshSocial() })
            else if (state === "inbox")
                Qt.callLater(function() { social.refreshInbox() })
            else if (state === "games")
                Qt.callLater(function() {
                    marketplace.refresh()
                    games.refreshGames()
                    cartridges.refresh()
                })
            else if (state === "challenges")
                Qt.callLater(function() { games.refreshChallenges() })
        }
    }

    SocialController {
        id: social
        sessionController: onboarding
        actor: onboarding.selectedPersona
    }

    GameController {
        id: games
        sessionController: onboarding
        actor: onboarding.selectedPersona
        helperEndpoint: root.argumentValue("--companion-endpoint=", "")
        helperCredential: root.argumentValue("--companion-credential=", "")
        marketplaceTrusted: marketplace.marketplaceReady
        operatorCustomTrusted: cartridges.operatorCustomTrusted
    }

    MarketplaceController {
        id: marketplace
        helperEndpoint: root.argumentValue("--companion-endpoint=", "")
        helperCredential: root.argumentValue("--companion-credential=", "")
        configured: root.argumentValue(
                        "--companion-marketplace-trusted=", "false") === "true"
    }

    CartridgeController {
        id: cartridges
        sessionController: onboarding
        actor: onboarding.selectedPersona
        helperEndpoint: root.argumentValue("--companion-endpoint=", "")
        helperCredential: root.argumentValue("--companion-credential=", "")
        marketplaceTrusted: marketplace.marketplaceReady
    }

    Component { id: connectionComponent; Screens.ConnectionScreen { controller: onboarding } }
    Component { id: accessComponent; Screens.AccessScreen { controller: onboarding } }
    Component { id: mfaComponent; Screens.MfaScreen { controller: onboarding } }
    Component { id: personaComponent; Screens.PersonaScreen { controller: onboarding } }
    Component { id: homeComponent; Screens.HomeScreen { controller: onboarding } }
    Component {
        id: socialComponent
        Screens.SocialScreen { controller: social; sessionController: onboarding }
    }
    Component {
        id: inboxComponent
        Screens.InboxScreen { controller: social; sessionController: onboarding }
    }
    Component {
        id: gamesComponent
        Screens.GamesScreen {
            controller: games
            cartridgeController: cartridges
            marketplaceController: marketplace
            sessionController: onboarding
        }
    }
    Component {
        id: challengesComponent
        Screens.ChallengesScreen { controller: games; sessionController: onboarding }
    }
    Component {
        id: gameplayComponent
        Screens.GameplayScreen { controller: games; sessionController: onboarding }
    }

    Component.onCompleted: {
        const initialUrl = argumentValue("--server-url=", "http://127.0.0.1:8080")
        onboarding.initialize(initialUrl)
    }

    Connections {
        target: marketplace
        function onMarketplaceReadyChanged() {
            if (marketplace.marketplaceReady && onboarding.state === "games")
                Qt.callLater(function() { cartridges.refresh() })
        }
    }

    Timer {
        interval: 15000
        repeat: false
        running: root.smokeTest
        onTriggered: Qt.exit(1)
    }

    Rectangle {
        anchors.fill: parent
        color: "transparent"
        border.color: theme.borderMuted
        border.width: theme.borderWidth

        Rectangle {
            id: statusRail
            width: parent.width
            objectName: "shellStatusRail"
            height: 24
            color: root.shellStateColor
            Accessible.role: Accessible.StatusBar
            Accessible.name: "Application state: " + root.shellStateLabel

            Text {
                anchors.fill: parent
                text: root.shellStateLabel + " // " + onboarding.state.toUpperCase()
                textFormat: Text.PlainText
                color: theme.background
                font.family: theme.fontFamily
                font.bold: true
                font.pixelSize: theme.captionSize
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
        }

        Item {
            id: brandBar
            anchors.top: statusRail.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: theme.controlHeight + theme.spaceMd

            Text {
                id: brand
                anchors.fill: parent
                anchors.leftMargin: theme.spaceLg
                anchors.rightMargin: exitButton.width + theme.space2Xl
                text: "OMARCHY // GAMES"
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.sectionSize
                font.letterSpacing: 3
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            Components.OgsButton {
                id: exitButton
                objectName: "shellExitButton"
                anchors.right: parent.right
                anchors.rightMargin: theme.spaceLg
                anchors.verticalCenter: parent.verticalCenter
                width: 88
                text: "EXIT"
                Accessible.role: Accessible.Button
                accessibleName: "Close Omarchy Gaming System"
                accessibleDescription: "Close this client without signing out or revoking the device session"
                onClicked: root.close()
            }
        }

        Loader {
            id: screenLoader
            objectName: "screenLoader"
            anchors.top: customModuleWarning.visible
                         ? customModuleWarning.bottom : brandBar.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: footer.top
            sourceComponent: root.screenComponent(onboarding.state)
            onLoaded: Qt.callLater(function() {
                if (screenLoader.item && screenLoader.item.focusInitial)
                    screenLoader.item.focusInitial()
            })
        }

        Rectangle {
            id: customModuleWarning
            objectName: "operatorCustomModuleWarning"
            anchors.top: brandBar.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: visible ? 52 : 0
            visible: root.customModuleDisclosure !== null
            color: theme.warning
            Accessible.role: Accessible.AlertMessage
            Accessible.name: visible
                             ? root.customModuleDisclosure.warning + " "
                               + root.customModuleDisclosure.support_boundary : ""

            Text {
                anchors.fill: parent
                anchors.leftMargin: theme.spaceLg
                anchors.rightMargin: theme.spaceLg
                text: customModuleWarning.visible
                      ? root.customModuleDisclosure.warning + " "
                        + root.customModuleDisclosure.support_boundary : ""
                textFormat: Text.PlainText
                color: theme.background
                font.family: theme.fontFamily
                font.bold: true
                font.pixelSize: theme.captionSize
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                wrapMode: Text.WordWrap
                maximumLineCount: 3
                elide: Text.ElideRight
            }
        }

        Text {
            id: footer
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: 20
            anchors.rightMargin: 20
            height: 28
            text: onboarding.state.toUpperCase() + " // TAB: MOVE // ENTER: ACTIVATE // ESC: BACK // REST RECOVERY"
            textFormat: Text.PlainText
            color: theme.textMuted
            font.family: theme.fontFamily
            font.pixelSize: theme.captionSize
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            Accessible.role: Accessible.Footer
            Accessible.name: "Keyboard navigation: Tab to move, Enter to activate, Escape to go back"
        }
    }
}
