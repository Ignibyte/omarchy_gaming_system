import QtQuick
import QtQuick.Controls
import "screens" as Screens

ApplicationWindow {
    id: root

    width: 920
    height: 600
    minimumWidth: 640
    minimumHeight: 420
    visible: true
    title: "Omarchy Gaming System"
    color: "#070b12"

    readonly property alias onboardingController: onboarding
    readonly property alias socialController: social
    property bool smokeTest: Qt.application.arguments.indexOf("--smoke-test") !== -1

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
        }
    }

    SocialController {
        id: social
        sessionController: onboarding
        actor: onboarding.selectedPersona
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

    Component.onCompleted: {
        const initialUrl = argumentValue("--server-url=", "http://127.0.0.1:8080")
        onboarding.initialize(initialUrl)
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
        border.color: "#182538"
        border.width: 1

        Rectangle {
            id: statusRail
            width: parent.width
            height: 6
            color: onboarding.errorText !== "" || social.errorText !== "" ? "#ff6b7a"
                  : onboarding.state === "home" || onboarding.state === "social"
                    || onboarding.state === "inbox" ? "#5ee6a8" : "#f4c95d"
        }

        Text {
            id: brand
            anchors.top: statusRail.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 16
            height: 34
            text: "OMARCHY // GAMES"
            textFormat: Text.PlainText
            color: "#8aa4c0"
            font.family: "monospace"
            font.pixelSize: 15
            font.letterSpacing: 3
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }

        Loader {
            id: screenLoader
            objectName: "screenLoader"
            anchors.top: brand.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: footer.top
            sourceComponent: root.screenComponent(onboarding.state)
            onLoaded: Qt.callLater(function() {
                if (screenLoader.item && screenLoader.item.focusInitial)
                    screenLoader.item.focusInitial()
            })
        }

        Text {
            id: footer
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: 20
            anchors.rightMargin: 20
            height: 28
            text: onboarding.state.toUpperCase() + " // REST RECOVERY LINK"
            textFormat: Text.PlainText
            color: "#546b82"
            font.family: "monospace"
            font.pixelSize: 11
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
    }
}
