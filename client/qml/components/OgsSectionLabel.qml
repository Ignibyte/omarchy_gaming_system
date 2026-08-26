import QtQuick

Text {
    id: root

    property string tone: "warning"

    textFormat: Text.PlainText
    color: theme.toneColor(tone)
    font.family: theme.fontFamily
    font.bold: true
    font.pixelSize: theme.sectionSize
    font.letterSpacing: 1
    wrapMode: Text.Wrap
    Accessible.role: Accessible.Heading
    Accessible.name: text

    OgsTheme { id: theme }
}
