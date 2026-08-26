import QtQuick

QtObject {
    readonly property color background: "#070b12"
    readonly property color surface: "#0c1825"
    readonly property color surfaceRaised: "#132537"
    readonly property color surfaceHover: "#1d3348"
    readonly property color surfacePressed: "#294966"
    readonly property color border: "#6685a3"
    readonly property color borderMuted: "#4f6b86"
    readonly property color textPrimary: "#eef7ff"
    readonly property color textSecondary: "#b7c9da"
    readonly property color textMuted: "#91a7bc"
    readonly property color accent: "#5ee6a8"
    readonly property color warning: "#f4c95d"
    readonly property color danger: "#ff8b98"
    readonly property color focus: "#ffffff"
    readonly property color highContrastBackground: "#000000"
    readonly property color highContrastForeground: "#ffffff"

    readonly property string fontFamily: "monospace"
    readonly property int titleSize: 28
    readonly property int heroSize: 30
    readonly property int sectionSize: 15
    readonly property int bodySize: 14
    readonly property int controlSize: 14
    readonly property int labelSize: 12
    readonly property int captionSize: 12

    readonly property int spaceXs: 4
    readonly property int spaceSm: 8
    readonly property int spaceMd: 12
    readonly property int spaceLg: 16
    readonly property int spaceXl: 24
    readonly property int space2Xl: 28
    readonly property int radius: 4
    readonly property int borderWidth: 1
    readonly property int focusWidth: 3
    readonly property int controlHeight: 44
    readonly property int textAreaHeight: 104

    function toneColor(tone) {
        switch (tone) {
        case "success": return accent
        case "warning": return warning
        case "error": return danger
        case "working": return warning
        default: return textMuted
        }
    }

    function tonePrefix(tone) {
        switch (tone) {
        case "success": return "READY"
        case "warning": return "WARNING"
        case "error": return "ERROR"
        case "working": return "WORKING"
        default: return "STATUS"
        }
    }
}
