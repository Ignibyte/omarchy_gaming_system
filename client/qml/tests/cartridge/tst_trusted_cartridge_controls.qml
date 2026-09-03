import QtQuick
import QtTest
import "../../cartridge" as Cartridge

TestCase {
    id: testCase

    name: "TrustedCartridgeControls"
    when: windowShown
    property var hostWindow: null
    property var surface: null

    Component {
        id: hostComponent

        Window {
            width: 920
            height: 640
            visible: true
            property alias surface: trustedSurface

            Cartridge.TrustedCartridgeSurface {
                id: trustedSurface
                anchors.fill: parent
                assetRoot: ""
                actionsEnabled: false
            }
        }
    }

    SignalSpy {
        id: actionSpy
        target: testCase.surface
        signalName: "actionRequested"
    }

    function origin() {
        return {
            "publisher_id": "test-publisher",
            "game_key": "test-game",
            "cartridge_version": 1,
            "archive_sha256": "a".repeat(64)
        };
    }

    function preferences() {
        return {
            "scale": 1,
            "high_contrast": false,
            "reduced_motion": true,
            "muted_audio": true
        };
    }

    function button(id, label, action) {
        return {
            "kind": "button",
            "id": id,
            "label": label,
            "action": action,
            "accessible_label": label
        };
    }

    function plan(nodes) {
        return {
            "format": "omarchygs.render-plan/v1",
            "profile": "core",
            "state": "ready",
            "state_message": "Ready",
            "origin": origin(),
            "title": "Control regression",
            "preferences": preferences(),
            "nodes": nodes,
            "requested_actions_are_unconfirmed": true
        };
    }

    function accept(nodes) {
        verify(surface.acceptPlan(plan(nodes)));
        tryCompare(surface, "instantiatedNodeCount", nodes.length);
        tryVerify(function () {
            return countTrustedNodes(surface) === nodes.length;
        }, 3000, "trusted surface instantiated a duplicate or missing delegate");
    }

    function countTrustedNodes(item) {
        let count = item.objectName
                && item.objectName.indexOf("trusted-node-") === 0 ? 1 : 0;
        for (let index = 0; index < item.children.length; index++)
            count += countTrustedNodes(item.children[index]);
        return count;
    }

    function node(id) {
        const objectName = "trusted-node-" + id;
        tryVerify(function () {
            return findChild(surface, objectName) !== null;
        }, 3000, "missing trusted node " + id);
        return findChild(surface, objectName);
    }

    function controlsDoNotOverlap(buttons) {
        let previousBottom = -1;
        for (let index = 0; index < buttons.length; index++) {
            const control = findChild(surface, "trusted-node-" + buttons[index].id);
            if (control === null)
                return false;
            const position = control.mapToItem(surface, 0, 0);
            if (control.height <= 0 || position.y < previousBottom)
                return false;
            previousBottom = position.y + control.height;
        }
        return true;
    }

    function init() {
        hostWindow = createTemporaryObject(hostComponent, testCase);
        verify(hostWindow !== null);
        surface = hostWindow.surface;
        hostWindow.requestActivate();
        tryVerify(function () {
            return hostWindow.active;
        }, 3000);
        surface.actionsEnabled = false;
        accept([]);
        actionSpy.clear();
    }

    function cleanup() {
        surface = null;
        if (hostWindow)
            hostWindow.destroy();
        hostWindow = null;
    }

    function test_pointer_follows_disabled_to_enabled_transition() {
        accept([button("continue", "Continue", "continue")]);
        const control = node("continue");
        compare(control.actionsEnabled, false);

        surface.actionsEnabled = true;
        tryCompare(control, "actionsEnabled", true);
        verify(control.visible);
        verify(control.width > 0);
        verify(control.height > 0);
        compare(surface.loadedNodeCount(), 1);
        compare(surface.actionNodeCount("continue"), 1);
        compare(surface.actionNodeCount("missing"), 0);
        compare(control.enabled, true);
        mouseClick(control, control.width / 2, control.height / 2, Qt.LeftButton);

        compare(actionSpy.count, 1);
        compare(actionSpy.signalArguments[0][0], "continue");
        compare(Object.keys(actionSpy.signalArguments[0][1]).length, 0);
    }

    function test_return_emits_one_current_action() {
        accept([button("attack", "Attack", "attack")]);
        const control = node("attack");
        surface.actionsEnabled = true;
        tryCompare(control, "actionsEnabled", true);
        control.forceActiveFocus();
        tryVerify(function () {
            return control.activeFocus;
        });

        keyClick(Qt.Key_Return);

        compare(actionSpy.count, 1);
        compare(actionSpy.signalArguments[0][0], "attack");
    }

    function test_plan_replacement_removes_old_delegates() {
        accept([button("old-one", "Old one", "old_one"), button("old-two", "Old two", "old_two")]);
        node("old-one");
        node("old-two");

        accept([button("new-one", "New one", "new_one")]);
        tryVerify(function () {
            return findChild(surface, "trusted-node-old-one") === null && findChild(surface, "trusted-node-old-two") === null;
        });
        const control = node("new-one");
        compare(surface.instantiatedNodeCount, 1);

        surface.actionsEnabled = true;
        tryCompare(control, "actionsEnabled", true);
        wait(1);
        compare(control.enabled, true);
        mouseClick(control, control.width / 2, control.height / 2, Qt.LeftButton);
        compare(actionSpy.count, 1);
        compare(actionSpy.signalArguments[0][0], "new_one");
    }

    function test_large_screen_replacement_keeps_one_delegate_per_control() {
        hostWindow.height = 1600;
        tryCompare(surface, "height", 1600);
        let levelTwentyButtons = [];
        let levelTwentyOneButtons = [];
        for (let index = 0; index < 23; index++)
            levelTwentyButtons.push(button("old-" + index, "Old " + index, "old_" + index));
        for (let index = 0; index < 24; index++)
            levelTwentyOneButtons.push(button("current-" + index, "Current " + index, "current_" + index));

        accept(levelTwentyButtons);
        compare(countTrustedNodes(surface), 23);
        verify(surface.acceptPlan(plan(levelTwentyOneButtons)));
        compare(surface.nodesMaterialized, false);
        compare(countTrustedNodes(surface), 0);
        tryCompare(surface, "nodesMaterialized", true);
        tryCompare(surface, "instantiatedNodeCount", 24);
        tryVerify(function () {
            for (let index = 0; index < levelTwentyOneButtons.length; index++) {
                const control = findChild(surface,
                        "trusted-node-current-" + index);
                if (control === null || !control.accessibilityReady)
                    return false;
            }
            return true;
        }, 3000, "current buttons were exposed before layout settled");
        tryVerify(function () {
            return findChild(surface, "trusted-node-old-0") === null;
        });
        compare(countTrustedNodes(surface), 24);
        compare(surface.loadedNodeCount(), 24);
        compare(surface.actionNodeCount("current_0"), 1);
        tryVerify(function () {
            return controlsDoNotOverlap(levelTwentyOneButtons);
        }, 3000, "trusted controls overlap after layout settles");

        surface.actionsEnabled = true;
        for (let pointerIndex = 0; pointerIndex < levelTwentyOneButtons.length;
                pointerIndex++) {
            const pointerControl = findChild(surface,
                    "trusted-node-current-" + pointerIndex);
            verify(pointerControl !== null);
            tryCompare(pointerControl, "actionsEnabled", true);
            const center = pointerControl.mapToItem(surface,
                    pointerControl.width / 2, pointerControl.height / 2);
            verify(center.x >= 0 && center.x < surface.width);
            verify(center.y >= 0 && center.y < surface.height);
            mouseClick(surface, center.x, center.y, Qt.LeftButton);
            compare(actionSpy.count, pointerIndex + 1);
            compare(actionSpy.signalArguments[pointerIndex][0],
                    levelTwentyOneButtons[pointerIndex].action);
        }

        actionSpy.clear();
        for (let index = 0; index < levelTwentyOneButtons.length; index++) {
            const control = findChild(surface, "trusted-node-current-" + index);
            verify(control !== null);
            tryCompare(control, "actionsEnabled", true);
            control.forceActiveFocus();
            tryVerify(function () { return control.activeFocus; });
            keyClick(Qt.Key_Return);
            compare(actionSpy.count, index + 1);
            compare(actionSpy.signalArguments[index][0],
                    levelTwentyOneButtons[index].action);
        }
    }
}
