import QtQuick
import "nodes" as Nodes
import "../components" as Components

Rectangle {
    id: root

    required property string assetRoot
    property var renderPlan: null
    property var acceptedPlan: null
    property bool actionsEnabled: true
    property bool planAccepted: false
    property string validationError: "No render plan loaded"
    property int instantiatedNodeCount: nodeRepeater.count
    signal actionRequested(string action, var payload)

    onRenderPlanChanged: acceptPlan(renderPlan)

    Components.OgsTheme { id: theme }

    color: acceptedPlan && acceptedPlan.preferences.high_contrast
           ? theme.highContrastBackground : theme.background
    border.color: acceptedPlan && acceptedPlan.preferences.high_contrast
                  ? theme.highContrastForeground : theme.borderMuted
    border.width: theme.borderWidth

    readonly property var allowedKinds: [
        "terminal", "grid", "status", "button", "image", "meter",
        "sprite", "particle_field", "audio_cue", "platform_placeholder"
    ]
    readonly property var allowedStates: [
        "ready", "loading", "offline", "stale", "empty", "protocol_error",
        "unsupported_capability", "revoked"
    ]

    function exactKeys(value, allowed) {
        const keys = Object.keys(value).sort()
        const expected = allowed.slice().sort()
        return JSON.stringify(keys) === JSON.stringify(expected)
    }

    function isString(value, maximum) {
        return typeof value === "string" && value.length <= maximum
    }

    function validDigest(value) {
        return typeof value === "string" && /^[0-9a-f]{64}$/.test(value)
    }

    function validAssetToken(value) {
        return typeof value === "string" && /^[0-9a-f]{64}\.(png|wav)$/.test(value)
    }

    function validNode(node) {
        if (!node || typeof node !== "object" || allowedKinds.indexOf(node.kind) === -1)
            return false
        if (!isString(node.id, 128) || !isString(node.accessible_label, 256))
            return false
        switch (node.kind) {
        case "terminal":
        case "status":
            return exactKeys(node, ["kind", "id", "text", "accessible_label"])
                && isString(node.text, 65536)
        case "grid":
            return exactKeys(node, ["kind", "id", "rows", "columns", "cells", "action", "accessible_label"])
                && Number.isInteger(node.rows) && node.rows > 0 && node.rows <= 64
                && Number.isInteger(node.columns) && node.columns > 0 && node.columns <= 64
                && Array.isArray(node.cells) && node.cells.length === node.rows * node.columns
                && node.cells.every(function(cell) { return isString(cell, 65536) })
                && isString(node.action, 128)
        case "button":
            return exactKeys(node, ["kind", "id", "label", "action", "accessible_label"])
                && isString(node.label, 65536) && isString(node.action, 128)
        case "image":
            return exactKeys(node, ["kind", "id", "asset_token", "accessible_label"])
                && validAssetToken(node.asset_token) && node.asset_token.endsWith(".png")
        case "meter":
            return exactKeys(node, ["kind", "id", "value", "minimum", "maximum", "accessible_label"])
                && Number.isFinite(node.value) && Number.isInteger(node.minimum)
                && Number.isInteger(node.maximum) && node.minimum < node.maximum
                && node.value >= node.minimum && node.value <= node.maximum
        case "sprite":
            return exactKeys(node, ["kind", "id", "asset_token", "frame_width", "frame_height", "frame_count", "frames_per_second", "animated", "accessible_label"])
                && validAssetToken(node.asset_token) && node.asset_token.endsWith(".png")
                && Number.isInteger(node.frame_width) && node.frame_width > 0
                && Number.isInteger(node.frame_height) && node.frame_height > 0
                && Number.isInteger(node.frame_count) && node.frame_count > 0 && node.frame_count <= 1024
                && Number.isInteger(node.frames_per_second) && node.frames_per_second > 0 && node.frames_per_second <= 120
                && typeof node.animated === "boolean"
        case "particle_field":
            return exactKeys(node, ["kind", "id", "particle_count", "preset", "running", "accessible_label"])
                && Number.isInteger(node.particle_count) && node.particle_count >= 0 && node.particle_count <= 4096
                && ["stars", "sparks", "snow"].indexOf(node.preset) !== -1
                && typeof node.running === "boolean"
        case "audio_cue":
            return exactKeys(node, ["kind", "id", "asset_token", "looped", "muted", "accessible_label"])
                && validAssetToken(node.asset_token) && node.asset_token.endsWith(".wav")
                && typeof node.looped === "boolean" && typeof node.muted === "boolean"
        case "platform_placeholder":
            return exactKeys(node, ["kind", "id", "message", "accessible_label"])
                && isString(node.message, 512)
        default:
            return false
        }
    }

    function withinProfileBudgets(plan) {
        const limits = plan.profile === "core"
            ? {"nodes": 256, "grid_cells": 1024, "images": 32, "sprites": 0,
                "particles": 0, "audio_cues": 0, "animations": 32}
            : {"nodes": 512, "grid_cells": 4096, "images": 64, "sprites": 128,
                "particles": 2048, "audio_cues": 16, "animations": 128}
        const usage = {"nodes": plan.nodes.length, "grid_cells": 0, "images": 0,
            "sprites": 0, "particles": 0, "audio_cues": 0, "animations": 0}
        for (let index = 0; index < plan.nodes.length; index++) {
            const node = plan.nodes[index]
            switch (node.kind) {
            case "grid":
                usage.grid_cells += node.cells.length
                break
            case "image":
                usage.images++
                break
            case "sprite":
                usage.sprites++
                usage.animations += node.animated ? 1 : 0
                break
            case "particle_field":
                usage.particles += node.particle_count
                usage.animations += node.running ? 1 : 0
                break
            case "audio_cue":
                usage.audio_cues++
                break
            }
        }
        return usage.nodes <= limits.nodes
            && usage.grid_cells <= limits.grid_cells
            && usage.images <= limits.images
            && usage.sprites <= limits.sprites
            && usage.particles <= limits.particles
            && usage.audio_cues <= limits.audio_cues
            && usage.animations <= limits.animations
    }

    function validatePlan(plan) {
        if (!plan || typeof plan !== "object"
                || !exactKeys(plan, ["format", "profile", "state", "state_message", "origin", "title", "preferences", "nodes", "requested_actions_are_unconfirmed"]))
            return "invalid plan envelope"
        if (plan.format !== "omarchygs.render-plan/v1"
                || ["core", "rich2d"].indexOf(plan.profile) === -1
                || allowedStates.indexOf(plan.state) === -1
                || !isString(plan.state_message, 512) || !isString(plan.title, 128)
                || plan.requested_actions_are_unconfirmed !== true)
            return "invalid plan identity"
        if (!plan.origin || !exactKeys(plan.origin, ["publisher_id", "game_key", "cartridge_version", "archive_sha256"])
                || !isString(plan.origin.publisher_id, 128) || !isString(plan.origin.game_key, 128)
                || !Number.isInteger(plan.origin.cartridge_version) || plan.origin.cartridge_version < 1
                || !validDigest(plan.origin.archive_sha256))
            return "invalid cartridge origin"
        if (!plan.preferences || !exactKeys(plan.preferences, ["scale", "high_contrast", "reduced_motion", "muted_audio"])
                || !Number.isFinite(plan.preferences.scale) || plan.preferences.scale < 0.75 || plan.preferences.scale > 2.0
                || typeof plan.preferences.high_contrast !== "boolean"
                || typeof plan.preferences.reduced_motion !== "boolean"
                || typeof plan.preferences.muted_audio !== "boolean")
            return "invalid trusted preferences"
        if (!Array.isArray(plan.nodes) || plan.nodes.length > (plan.profile === "core" ? 256 : 512)
                || !plan.nodes.every(validNode) || !withinProfileBudgets(plan))
            return "invalid render nodes"
        if (plan.state !== "ready" && plan.nodes.length !== 0)
            return "non-ready plan contains cartridge nodes"
        return ""
    }

    function acceptPlan(plan) {
        const error = validatePlan(plan)
        if (error !== "") {
            acceptedPlan = null
            planAccepted = false
            validationError = error
            return false
        }
        acceptedPlan = plan
        planAccepted = true
        validationError = ""
        return true
    }

    function componentForKind(kind) {
        switch (kind) {
        case "terminal": return terminalComponent
        case "grid": return gridComponent
        case "status": return statusComponent
        case "button": return buttonComponent
        case "image": return imageComponent
        case "meter": return meterComponent
        case "sprite": return spriteComponent
        case "particle_field": return particleComponent
        case "audio_cue": return audioComponent
        case "platform_placeholder": return placeholderComponent
        default: return null
        }
    }

    function smokeExercise() {
        let expected = 0
        let exercised = 0
        let focusObserved = false
        for (let index = 0; index < nodeRepeater.count; index++) {
            const loader = nodeRepeater.itemAt(index)
            if (!loader || !loader.item)
                continue
            if (loader.modelData.kind === "grid") {
                expected++
                loader.item.forceActiveFocus()
                focusObserved = focusObserved || loader.item.activeFocus
                loader.item.moveSelection(0, 1)
                loader.item.triggerSelected()
                exercised++
            } else if (loader.modelData.kind === "button") {
                expected++
                loader.item.forceActiveFocus()
                focusObserved = focusObserved || loader.item.activeFocus
                loader.item.trigger()
                exercised++
            }
        }
        return {"expected": expected, "exercised": exercised, "focus_observed": focusObserved}
    }

    function focusInitial() {
        for (let index = 0; index < nodeRepeater.count; index++) {
            const loader = nodeRepeater.itemAt(index)
            if (loader && loader.item
                    && (loader.modelData.kind === "button" || loader.modelData.kind === "grid")) {
                loader.item.forceActiveFocus()
                return true
            }
        }
        root.forceActiveFocus()
        return true
    }

    Component { id: terminalComponent; Nodes.TrustedTerminalNode {} }
    Component { id: gridComponent; Nodes.TrustedGridNode {} }
    Component { id: statusComponent; Nodes.TrustedStatusNode {} }
    Component { id: buttonComponent; Nodes.TrustedButtonNode {} }
    Component { id: imageComponent; Nodes.TrustedImageNode {} }
    Component { id: meterComponent; Nodes.TrustedMeterNode {} }
    Component { id: spriteComponent; Nodes.TrustedSpriteNode {} }
    Component { id: particleComponent; Nodes.TrustedParticleFieldNode {} }
    Component { id: audioComponent; Nodes.TrustedAudioCueNode {} }
    Component { id: placeholderComponent; Nodes.TrustedPlaceholderNode {} }

    Column {
        anchors.fill: parent
        anchors.margins: 18
        spacing: 10

        Text {
            width: parent.width
            text: root.acceptedPlan
                ? "OMARCHYGS // " + root.acceptedPlan.origin.publisher_id + "/" + root.acceptedPlan.origin.game_key
                    + " v" + root.acceptedPlan.origin.cartridge_version
                    + " // " + root.acceptedPlan.origin.archive_sha256.slice(0, 12)
                : "OMARCHYGS // UNTRUSTED PLAN REJECTED"
            textFormat: Text.PlainText
            color: root.acceptedPlan && root.acceptedPlan.preferences.high_contrast
                   ? theme.highContrastForeground : theme.textMuted
            font.family: theme.fontFamily
            font.pixelSize: theme.captionSize
                            * (root.acceptedPlan ? root.acceptedPlan.preferences.scale : 1)
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.acceptedPlan ? root.acceptedPlan.state_message : root.validationError
            textFormat: Text.PlainText
            color: root.acceptedPlan && root.acceptedPlan.state === "ready"
                   ? theme.accent : theme.warning
            font.family: theme.fontFamily
            font.bold: true
            font.pixelSize: theme.sectionSize
                            * (root.acceptedPlan ? root.acceptedPlan.preferences.scale : 1)
            wrapMode: Text.Wrap
        }

        Flickable {
            width: parent.width
            height: parent.height - 82
            contentWidth: width
            contentHeight: nodeColumn.implicitHeight
            clip: true

            Column {
                id: nodeColumn
                width: parent.width
                spacing: 8

                Repeater {
                    id: nodeRepeater
                    model: root.planAccepted && root.acceptedPlan.state === "ready"
                        ? root.acceptedPlan.nodes : []

                    delegate: Loader {
                        id: nodeLoader
                        required property var modelData
                        width: nodeColumn.width
                        sourceComponent: root.componentForKind(modelData.kind)
                        onLoaded: {
                            item.assetRoot = root.assetRoot
                            item.scaleFactor = root.acceptedPlan.preferences.scale
                            item.highContrast = root.acceptedPlan.preferences.high_contrast
                            item.reducedMotion = root.acceptedPlan.preferences.reduced_motion
                            item.mutedAudio = root.acceptedPlan.preferences.muted_audio
                            item.nodeData = modelData
                            if (item.actionsEnabled !== undefined)
                                item.actionsEnabled = root.actionsEnabled
                        }

                        Connections {
                            target: nodeLoader.item
                            ignoreUnknownSignals: true
                            function onActionRequested(action, payload) {
                                if (root.actionsEnabled)
                                    root.actionRequested(action, payload)
                            }
                        }
                    }
                }
            }
        }
    }
}
