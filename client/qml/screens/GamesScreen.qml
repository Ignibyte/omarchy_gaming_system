import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../components" as Components

Item {
    id: root

    required property var controller
    required property var cartridgeController
    required property var marketplaceController
    required property var sessionController

    Components.OgsTheme { id: theme }

    function focusInitial() {
        refreshButton.forceActiveFocus()
    }

    Keys.onEscapePressed: function(event) {
        sessionController.showPlayerScreen("home")
        event.accepted = true
    }

    ScrollView {
        id: scroll
        anchors.fill: parent
        anchors.margins: theme.spaceLg
        contentWidth: availableWidth

        ColumnLayout {
            width: scroll.availableWidth
            spacing: theme.spaceMd

            Components.OgsScreenHeader {
                Layout.fillWidth: true
                screenKey: "games"
                title: "GAME CARTRIDGES"
                statusText: controller.statusText
                statusTone: controller.busy || controller.loadState === "loading"
                            ? "working" : "success"
                errorText: controller.errorText
                navigationHint: "ESC HOME // ENTER START OR OPEN"
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                Components.OgsButton {
                    id: refreshButton
                    objectName: "gamesRefreshButton"
                    text: "REFRESH"
                    accessibleName: "Refresh game cartridges and sessions"
                    enabled: !controller.busy && !cartridgeController.busy
                             && !marketplaceController.busy
                    onClicked: {
                        marketplaceController.refresh()
                        controller.refreshGames()
                        cartridgeController.refresh()
                    }
                }

                Components.OgsButton {
                    objectName: "gamesChallengesButton"
                    text: "CHALLENGES"
                    accessibleName: "Open game challenges"
                    enabled: !controller.busy
                    onClicked: sessionController.showPlayerScreen("challenges")
                }

                Components.OgsButton {
                    objectName: "gamesHomeButton"
                    text: "HOME"
                    accessibleName: "Return to player home"
                    enabled: !controller.busy
                    onClicked: sessionController.showPlayerScreen("home")
                }
            }

            Components.OgsButton {
                objectName: "gamesRetryButton"
                visible: controller.hasRetryableMutation
                text: "RETRY SAME OPERATION"
                accessibleName: "Retry the same game operation identity"
                enabled: !controller.busy
                onClicked: controller.retryPendingMutation()
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "MARKETPLACE TRUST & CLIENT PACKAGES"
            }

            Components.OgsStatusBanner {
                Layout.fillWidth: true
                message: marketplaceController.errorText !== ""
                         ? marketplaceController.errorText
                         : marketplaceController.statusText
                tone: marketplaceController.errorText !== "" ? "error"
                    : marketplaceController.busy ? "working"
                    : marketplaceController.marketplaceReady ? "success" : "warning"
                accessibleDescription: "Independent marketplace trust, key rotation, and reviewed client package state"
            }

            ColumnLayout {
                Layout.fillWidth: true
                visible: marketplaceController.trust !== null
                         && marketplaceController.trust.mode === "channel"
                spacing: 3

                Text {
                    Layout.fillWidth: true
                    text: marketplaceController.trust === null ? ""
                          : "CHANNEL " + marketplaceController.trust.channel_name
                            + " // " + marketplaceController.trust.channel_origin
                            + " // BUNDLE " + marketplaceController.trust.bundle_version
                            + " // EXPIRES UNIX "
                            + marketplaceController.trust.expires_at_unix
                    textFormat: Text.PlainText
                    color: theme.textSecondary
                    font.family: theme.fontFamily
                    font.pixelSize: theme.captionSize
                    wrapMode: Text.WrapAnywhere
                }

                Repeater {
                    model: marketplaceController.trust === null
                           ? [] : marketplaceController.trust.keys
                    delegate: Text {
                        required property var modelData
                        Layout.fillWidth: true
                        text: "KEY " + modelData.status.toUpperCase() + " // "
                              + modelData.key_sha256 + " // SNAPSHOTS "
                              + modelData.first_snapshot_version + "–"
                              + (modelData.last_snapshot_version === undefined
                                 ? "CURRENT" : modelData.last_snapshot_version)
                        textFormat: Text.PlainText
                        color: modelData.status === "revoked"
                               ? theme.error : modelData.status === "retired"
                                 ? theme.warning : theme.textSecondary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.captionSize
                        wrapMode: Text.WrapAnywhere
                    }
                }
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight

                Components.OgsButton {
                    objectName: "marketplaceTrustSyncButton"
                    text: marketplaceController.trust !== null
                          && marketplaceController.trust.enrolled ? "SYNC TRUST" : "ENROLL"
                    accessibleName: text + " with the independently packaged marketplace channel"
                    enabled: marketplaceController.configured
                             && !marketplaceController.busy
                             && (marketplaceController.trust === null
                                 || marketplaceController.trust.mode === "channel")
                    onClicked: marketplaceController.synchronize()
                }

                Components.OgsButton {
                    objectName: "marketplacePackageRefreshButton"
                    text: "CHECK PACKAGES"
                    accessibleName: "Check root-authenticated OmarchyGS client packages"
                    enabled: marketplaceController.marketplaceReady
                             && !marketplaceController.busy
                             && marketplaceController.channelMode
                    onClicked: marketplaceController.refresh()
                }
            }

            Repeater {
                model: marketplaceController.packages
                delegate: Components.OgsCard {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: 154
                    tone: "info"

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 10

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4

                            Text {
                                Layout.fillWidth: true
                                text: "CLIENT " + modelData.package_version.toUpperCase()
                                      + " // " + modelData.filename
                                textFormat: Text.PlainText
                                color: theme.textPrimary
                                font.family: theme.fontFamily
                                font.bold: true
                                font.pixelSize: theme.bodySize
                                elide: Text.ElideRight
                            }

                            Text {
                                Layout.fillWidth: true
                                text: "PACKAGE SHA-256 // " + modelData.sha256
                                textFormat: Text.PlainText
                                color: theme.textSecondary
                                font.family: theme.fontFamily
                                font.pixelSize: theme.captionSize
                                wrapMode: Text.WrapAnywhere
                            }

                            Text {
                                Layout.fillWidth: true
                                text: "SOURCE REVISION // " + modelData.source_revision
                                      + "\nSOURCE SHA-256 // " + modelData.source_sha256
                                      + "\nBUILD PROVENANCE SHA-256 // "
                                      + modelData.build_provenance_sha256
                                textFormat: Text.PlainText
                                color: theme.textSecondary
                                font.family: theme.fontFamily
                                font.pixelSize: theme.captionSize
                                wrapMode: Text.WrapAnywhere
                            }
                        }

                        Components.OgsButton {
                            objectName: "marketplacePackageStageButton"
                            text: "VERIFY & STAGE"
                            accessibleName: "Download, verify, and stage client package "
                                            + modelData.package_version
                            enabled: !marketplaceController.busy
                            onClicked: marketplaceController.stage(modelData)
                        }
                    }
                }
            }

            Components.OgsCard {
                Layout.fillWidth: true
                Layout.preferredHeight: 148
                visible: marketplaceController.stagedPackage !== null
                tone: "warning"

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 10
                    spacing: 5

                    Text {
                        Layout.fillWidth: true
                        text: marketplaceController.stagedPackage === null ? ""
                              : "STAGED " + marketplaceController.stagedPackage.package_version
                                + " // SHA-256 "
                                + marketplaceController.stagedPackage.sha256
                        textFormat: Text.PlainText
                        color: theme.warning
                        font.family: theme.fontFamily
                        font.bold: true
                        font.pixelSize: theme.bodySize
                        wrapMode: Text.WrapAnywhere
                    }

                    Text {
                        Layout.fillWidth: true
                        text: marketplaceController.stagedPackage === null ? ""
                              : marketplaceController.stagedPackage.staged_path
                        textFormat: Text.PlainText
                        color: theme.textSecondary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.captionSize
                        elide: Text.ElideMiddle
                    }

                    TextEdit {
                        id: installCommandClipboard
                        Layout.preferredWidth: 1
                        Layout.preferredHeight: 1
                        opacity: 0
                        readOnly: true
                        text: marketplaceController.stagedPackage === null ? ""
                              : marketplaceController.stagedPackage.install_command
                    }

                    Components.OgsButton {
                        Layout.alignment: Qt.AlignRight
                        objectName: "marketplacePackageCopyCommandButton"
                        text: "COPY PACMAN COMMAND"
                        accessibleName: "Copy the verified package install command"
                        accessibleDescription: "Copies text only; OmarchyGS does not run pacman, sudo, or a shell"
                        onClicked: {
                            installCommandClipboard.selectAll()
                            installCommandClipboard.copy()
                            installCommandClipboard.deselect()
                        }
                    }
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "SIGNED SERVER CARTRIDGES (" + cartridgeController.catalog.length + ")"
            }

            Components.OgsStatusBanner {
                Layout.fillWidth: true
                message: cartridgeController.errorText !== ""
                         ? cartridgeController.errorText : cartridgeController.statusText
                tone: cartridgeController.errorText !== "" ? "error"
                    : cartridgeController.busy || cartridgeController.loadState === "loading"
                      ? "working" : cartridgeController.loadState === "unavailable"
                        ? "warning" : "success"
                accessibleDescription: "Signed cartridge acquisition and local mount state"
            }

            Text {
                Layout.fillWidth: true
                visible: cartridgeController.loadState !== "loading"
                         && cartridgeController.catalog.length === 0
                text: "No downloadable signed cartridges are available from this server."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
                wrapMode: Text.Wrap
            }

            Repeater {
                objectName: "cartridgeCatalogRepeater"
                model: cartridgeController.catalog
                delegate: Components.OgsCard {
                    required property var modelData
                    readonly property var exactMount: cartridgeController.mountForExact(modelData)
                    Layout.fillWidth: true
                    Layout.preferredHeight: exactMount === null ? 126 : 176
                    tone: modelData.marketplace.lifecycle_status === "deprecated"
                          ? "warning" : "info"
                    highlighted: cartridgeController.isMountedExact(modelData)

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 6

                        Text {
                            Layout.fillWidth: true
                            text: modelData.display_name + " // CARTRIDGE v"
                                  + modelData.cartridge_version + " // "
                                  + modelData.archive_sha256.slice(0, 12)
                            textFormat: Text.PlainText
                            color: theme.textPrimary
                            font.family: theme.fontFamily
                            font.bold: true
                            font.pixelSize: theme.bodySize
                            elide: Text.ElideRight
                        }

                        Text {
                            Layout.fillWidth: true
                            visible: exactMount !== null
                            text: exactMount === null ? ""
                                  : "LOCAL TRUST "
                                    + (exactMount.trust_status === undefined
                                       ? "TRUSTED" : exactMount.trust_status.toUpperCase())
                                    + " // EVIDENCE KEY "
                                    + exactMount.marketplace_key_sha256
                                    + (exactMount.policy_marketplace_key_sha256 === undefined
                                       ? "" : " // POLICY KEY "
                                         + exactMount.policy_marketplace_key_sha256)
                            textFormat: Text.PlainText
                            color: exactMount !== null
                                   && ["revoked", "expired", "unknown"]
                                      .indexOf(exactMount.trust_status) !== -1
                                   ? theme.error : exactMount !== null
                                     && exactMount.trust_status === "retired"
                                       ? theme.warning : theme.textSecondary
                            font.family: theme.fontFamily
                            font.pixelSize: theme.captionSize
                            wrapMode: Text.WrapAnywhere
                        }

                        Text {
                            Layout.fillWidth: true
                            text: modelData.marketplace.marketplace_name + " // REVIEWED BY "
                                  + modelData.marketplace.reviewed_by.toUpperCase() + " // "
                                  + modelData.marketplace.lifecycle_status.toUpperCase()
                            textFormat: Text.PlainText
                            color: modelData.marketplace.lifecycle_status === "deprecated"
                                   ? theme.warning : theme.textSecondary
                            font.family: theme.fontFamily
                            font.pixelSize: theme.captionSize
                            elide: Text.ElideRight
                        }

                        Text {
                            Layout.fillWidth: true
                            visible: modelData.warning !== undefined
                            text: modelData.warning === undefined ? "" : modelData.warning
                            textFormat: Text.PlainText
                            color: theme.warning
                            font.family: theme.fontFamily
                            font.pixelSize: theme.captionSize
                            elide: Text.ElideRight
                        }

                        RowLayout {
                            Layout.alignment: Qt.AlignRight

                            Components.OgsButton {
                                objectName: "cartridgeInstallButton"
                                text: cartridgeController.actionLabel(modelData)
                                accessibleName: text + " " + modelData.display_name
                                accessibleDescription: "Verify and mount the exact signed release for this server profile"
                                enabled: !cartridgeController.busy
                                         && cartridgeController.helperAvailable
                                         && cartridgeController.marketplaceTrusted
                                         && cartridgeController.acquisitionSupported
                                         && !cartridgeController.isMountedExact(modelData)
                                onClicked: cartridgeController.install(modelData)
                            }

                            Components.OgsButton {
                                objectName: "cartridgeRemoveButton"
                                text: "REMOVE"
                                accessibleName: "Remove local mount for " + modelData.display_name
                                accessibleDescription: "Remove presentation bytes from this server profile without deleting game state"
                                enabled: !cartridgeController.busy
                                         && cartridgeController.helperAvailable
                                         && cartridgeController.isMountedExact(modelData)
                                onClicked: cartridgeController.remove(modelData)
                            }
                        }
                    }
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "SOLO CARTRIDGES"
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.soloGames().length === 0
                text: "No solo cartridges are available."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
            }

            Repeater {
                model: controller.soloGames()
                delegate: Components.OgsButton {
                    required property var modelData
                    Layout.fillWidth: true
                    text: "START " + modelData.display_name.toUpperCase()
                    accessibleName: "Start " + modelData.display_name + " version " + modelData.version
                    enabled: !controller.busy
                    onClicked: controller.startSolo(modelData)
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "INSTALLED CATALOG (" + controller.catalog.length + ")"
            }

            Repeater {
                model: controller.catalog
                delegate: Components.OgsCard {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: 52
                    tone: "info"

                    Text {
                        anchors.fill: parent
                        anchors.margins: 10
                        text: modelData.display_name + " // v" + modelData.version + " // "
                              + modelData.min_human_players + "–" + modelData.max_human_players
                              + " PLAYERS // " + modelData.authority.toUpperCase()
                        textFormat: Text.PlainText
                        color: theme.textSecondary
                        font.family: theme.fontFamily
                        font.pixelSize: theme.bodySize
                        wrapMode: Text.Wrap
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }

            Components.OgsSectionLabel {
                Layout.fillWidth: true
                text: "YOUR SESSIONS (" + controller.sessions.length + ")"
            }

            Text {
                Layout.fillWidth: true
                visible: controller.loadState !== "loading" && controller.sessions.length === 0
                text: "No matches yet. Start a solo cartridge or accept a challenge."
                textFormat: Text.PlainText
                color: theme.textMuted
                font.family: theme.fontFamily
                font.pixelSize: theme.bodySize
                wrapMode: Text.Wrap
            }

            Repeater {
                model: controller.sessions
                delegate: Components.OgsButton {
                    required property var modelData
                    Layout.fillWidth: true
                    text: controller.gameName(modelData.game_key, modelData.game_version).toUpperCase()
                          + " // " + modelData.status.toUpperCase() + " // REV " + modelData.revision
                    accessibleName: "Open " + controller.gameName(modelData.game_key, modelData.game_version)
                                    + " " + modelData.status + " session"
                    enabled: !controller.busy
                    onClicked: controller.openSession(modelData)
                }
            }
        }
    }
}
