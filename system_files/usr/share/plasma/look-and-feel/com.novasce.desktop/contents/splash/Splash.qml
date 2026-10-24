/*
    Novasce OS · abertura (KSplash). Canvas v0.5, prancha 2.
    Teal #00666C, cata-vento branco PARADO e uma linha que anda com o `stage`.
    Nada gira. Com animações desligadas no KDE, a linha pula direto.
*/
import QtQuick
import org.kde.kirigami as Kirigami

Rectangle {
    id: root
    color: "#00666C"

    property int stage

    // O KSplash avisa os estágios de 1 a 6; a linha acompanha.
    readonly property real progresso: Math.max(0, Math.min(1, stage / 6))

    Image {
        id: logo
        readonly property real lado: root.height * 0.128
        anchors.horizontalCenter: parent.horizontalCenter
        y: root.height * 0.45 - height / 2
        source: "images/catavento.svgz"
        sourceSize.width: lado
        sourceSize.height: lado
        asynchronous: true
    }

    Rectangle {
        id: trilho
        width: root.width * 0.125
        height: 3
        radius: 1.5
        color: Qt.rgba(245 / 255, 245 / 255, 247 / 255, 0.25)
        anchors.horizontalCenter: parent.horizontalCenter
        y: root.height * 0.583

        Rectangle {
            height: parent.height
            radius: parent.radius
            color: "#F5F5F7"
            width: parent.width * root.progresso
            Behavior on width {
                enabled: Kirigami.Units.longDuration > 1
                NumberAnimation {
                    duration: 240
                    easing.type: Easing.BezierSpline
                    easing.bezierCurve: [0.2, 0.7, 0.2, 1, 1, 1]
                }
            }
        }
    }

    Text {
        text: "Novasce OS"
        color: Qt.rgba(245 / 255, 245 / 255, 247 / 255, 0.7)
        font.family: "Inter"
        font.pixelSize: root.height * 0.021
        font.letterSpacing: font.pixelSize * 0.04
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.height * 0.061
    }
}
