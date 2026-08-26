import QtQuick
import QtQuick.Layouts

Item {
    id: root

    property string screenKey: "screen"
    property string title: ""
    property string titleTone: "success"
    property string statusText: ""
    property string statusTone: "info"
    property string errorText: ""
    property string navigationHint: ""

    implicitHeight: headerColumn.implicitHeight

    OgsTheme { id: theme }

    ColumnLayout {
        id: headerColumn
        anchors.fill: parent
        spacing: theme.spaceSm

        Text {
            objectName: root.screenKey + "Heading"
            Layout.fillWidth: true
            text: root.title
            textFormat: Text.PlainText
            color: theme.toneColor(root.titleTone)
            font.family: theme.fontFamily
            font.bold: true
            font.pixelSize: theme.titleSize
            wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter
            Accessible.role: Accessible.Heading
            Accessible.name: root.title
        }

        OgsStatusBanner {
            objectName: root.screenKey + "StatusBanner"
            Layout.fillWidth: true
            message: root.statusText
            tone: root.statusTone
            accessibleDescription: "Current " + root.title.toLowerCase() + " state"
        }

        OgsStatusBanner {
            objectName: root.screenKey + "ErrorBanner"
            Layout.fillWidth: true
            message: root.errorText
            tone: "error"
            accessibleDescription: "Action required on " + root.title.toLowerCase()
        }

        Text {
            objectName: root.screenKey + "NavigationHint"
            Layout.fillWidth: true
            visible: root.navigationHint !== ""
            text: "NAV // " + root.navigationHint
            textFormat: Text.PlainText
            color: theme.textMuted
            font.family: theme.fontFamily
            font.pixelSize: theme.captionSize
            wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter
            Accessible.name: "Navigation: " + root.navigationHint
        }
    }
}
