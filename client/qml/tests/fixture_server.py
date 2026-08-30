#!/usr/bin/env python3
"""Quiet deterministic HTTP fixture for the QML onboarding contract."""

from __future__ import annotations

import json
import os
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, unquote, urlparse


ACCOUNT_ID = "11111111-1111-4111-8111-111111111111"
SESSION_ID = "22222222-2222-4222-8222-222222222222"
PERSONA_A_ID = "33333333-3333-4333-8333-333333333333"
PERSONA_B_ID = "44444444-4444-4444-8444-444444444444"
SOCIAL_ACTOR_ID = "55555555-5555-4555-8555-555555555555"
SOCIAL_PEER_ID = "66666666-6666-4666-8666-666666666666"
SOCIAL_FRIEND_ID = "77777777-7777-4777-8777-777777777777"
SOCIAL_OUTGOING_ID = "88888888-8888-4888-8888-888888888888"
SOCIAL_BLOCKED_ID = "99999999-9999-4999-8999-999999999999"
SOCIAL_LOST_ID = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
REPORT_MALFORMED_ID = "abababab-abab-4bab-8bab-abababababab"
REPORT_OVERSIZED_ID = "acacacac-acac-4cac-8cac-acacacacacac"
REPORT_SESSION_LOST_ID = "adadadad-adad-4dad-8dad-adadadadadad"
REPORT_ID = "aeaeaeae-aeae-4eae-8eae-aeaeaeaeaeae"
CONVERSATION_ID = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"
SOLO_GAME_SESSION_ID = "10101010-1010-4010-8010-101010101010"
VERSUS_GAME_SESSION_ID = "20202020-2020-4020-8020-202020202020"
INCOMING_CHALLENGE_ID = "30303030-3030-4030-8030-303030303030"
OUTGOING_CHALLENGE_ID = "40404040-4040-4040-8040-404040404040"
MESSAGE_1_ID = "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
MESSAGE_2_ID = "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
MESSAGE_3_ID = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"
MESSAGE_4_ID = "ffffffff-ffff-4fff-8fff-ffffffffffff"
TOKEN_A = "ogs1_" + "A" * 43
TOKEN_M = "ogs1_" + "M" * 43
TOKEN_S = "ogs1_" + "S" * 43
TOKEN_U = "ogs1_" + "U" * 43
TOKEN_X = "ogs1_" + "X" * 43
CHALLENGE = "ogm1_" + "C" * 43
CREATED_AT = "2026-08-25T20:00:00.000Z"
EXPIRES_AT = "2099-08-25T20:05:00.000Z"
INVITE_CODE = "ogsi_" + "I" * 43
SERVER_ID = "12121212-1212-4212-8212-121212121212"
SECOND_SERVER_ID = "13131313-1313-4313-8313-131313131313"
REPLACEMENT_SERVER_ID = "14141414-1414-4414-8414-141414141414"
CATALOG_ONLY_SERVER_ID = "15151515-1515-4515-8515-151515151515"
DISCOVERY_CAPABILITIES = [
    "accounts.invite-registration.v1",
    "auth.device-sessions.v1",
    "auth.totp.v1",
    "games.cartridge-acquisition.v1",
    "games.cartridge-catalog.v1",
    "games.challenges.v1",
    "games.session-cartridge-acquisition.v1",
    "games.sessions.v1",
    "identity.personas.v1",
    "social.connections.v1",
    "social.private-inbox.v1",
    "social.reporting.v1",
    "sync.cursor.v1",
    "sync.websocket-hints.v1",
]
CARTRIDGE_DIGEST = "a" * 64
CARTRIDGE_IDENTITY = "b" * 64
COMPANION_TOKEN = "C" * 43
OPERATOR_KEY_SHA256 = "e" * 64
OPERATOR_WARNING = (
    "Operator-custom content: not reviewed or supported by the OmarchyGS marketplace."
)


def persona(persona_id: str, handle: str, display_name: str) -> dict[str, Any]:
    return {
        "id": persona_id,
        "handle": handle,
        "display_name": display_name,
        "bio": "Fixture persona",
        "status_message": "Ready",
        "created_at": CREATED_AT,
        "updated_at": CREATED_AT,
    }


class FixtureState:
    def __init__(self, mode: str) -> None:
        self.mode = mode
        self.lock = threading.Lock()
        self.calls: list[str] = []
        self.violations: list[str] = []
        self.peer_accepted = False
        self.outgoing_present = True
        self.friend_connected = True
        self.block_present = True
        self.sent_body = ""
        self.read_message_id = ""
        self.report_body: dict[str, Any] | None = None
        self.report_attempt_keys: dict[str, str] = {}
        self.game_revision = 0
        self.game_state = self._solo_state()
        self.versus_revision = 1
        self.versus_state = self._versus_state()
        self.incoming_challenge_status = "pending"
        self.outgoing_challenge_status = "absent"
        self.cartridge_mounted = False
        self.operator_trusted = False

    def reset_social(self) -> None:
        with self.lock:
            self.peer_accepted = False
            self.outgoing_present = True
            self.friend_connected = True
            self.block_present = True
            self.sent_body = ""
            self.read_message_id = ""
            self.report_body = None
            self.report_attempt_keys = {}
            self.game_revision = 0
            self.game_state = self._solo_state()
            self.versus_revision = 1
            self.versus_state = self._versus_state()
            self.incoming_challenge_status = "pending"
            self.outgoing_challenge_status = "absent"
            self.cartridge_mounted = False
            self.operator_trusted = False

    def record_call(self, call: str) -> None:
        with self.lock:
            self.calls.append(call)

    def violate(self, message: str) -> None:
        with self.lock:
            self.violations.append(message)

    @staticmethod
    def _solo_state() -> dict[str, Any]:
        return {
            "schema_version": 1,
            "rules_version": 1,
            "round": 0,
            "max_rounds": 12,
            "phase": "awaiting_human",
            "human": {"core": 8, "energy": 2},
            "bot": {"core": 8, "energy": 2},
            "last_round": None,
            "outcome": None,
        }

    @staticmethod
    def _versus_state() -> dict[str, Any]:
        return {
            "schema_version": 1,
            "rules_version": 2,
            "turn": 1,
            "max_turns": 24,
            "phase": "awaiting_action",
            "active_seat": 1,
            "players": [
                {"seat": 0, "core": 8, "energy": 4, "guard": 0},
                {"seat": 1, "core": 8, "energy": 2, "guard": 0},
            ],
            "last_turn": {
                "turn": 1,
                "actor_seat": 0,
                "action": "charge",
                "damage_to_opponent": 0,
                "blocked_damage": 0,
            },
            "outcome": None,
        }


class Handler(BaseHTTPRequestHandler):
    server_version = "OmarchyQmlFixture/1"
    protocol_version = "HTTP/1.1"

    @property
    def state(self) -> FixtureState:
        return self.server.fixture_state  # type: ignore[attr-defined]

    def log_message(self, _format: str, *_args: object) -> None:
        return

    def do_GET(self) -> None:  # noqa: N802
        self.state.record_call(f"GET {self.path}")
        parsed = urlparse(self.path)
        path = parsed.path
        query = parse_qs(parsed.query)
        if self.path == "/__fixture__/status":
            self._json(200, {"calls": self.state.calls, "violations": self.state.violations})
            return
        if self.path == "/__fixture__/reset-social":
            self.state.reset_social()
            self._json(200, {"ok": True})
            return
        if path == "/.well-known/omarchygs":
            self._require_no_authorization("server discovery")
            if self.state.mode == "slow":
                time.sleep(0.6)
            if self.state.mode == "malformed":
                self._raw(200, b"{not-json", "application/json")
                return
            if self.state.mode == "oversized":
                oversized = self._discovery_document(SERVER_ID, "Oversized Fixture")
                oversized["padding"] = "x" * 300_000
                self._json(200, oversized)
                return
            if self.state.mode == "wrong_identity":
                document = self._discovery_document(SERVER_ID, "Wrong Service")
                document["service"] = "not-omarchygs"
                self._json(200, document)
                return
            if self.state.mode == "incompatible":
                document = self._discovery_document(SERVER_ID, "Future Fixture")
                document["protocol_version"] = 2
                self._json(200, document)
                return
            if self.state.mode == "identity_changed":
                self._json(200, self._discovery_document(
                    REPLACEMENT_SERVER_ID, "Replacement Fixture"
                ))
                return
            if self.state.mode == "server_two":
                document = self._discovery_document(
                    SECOND_SERVER_ID, "Second Fixture Community"
                )
                document["capabilities"].append("future.arcade-mode.v1")
                document["capabilities"].sort()
                self._json(200, document)
                return
            if self.state.mode == "catalog_only":
                document = self._discovery_document(
                    CATALOG_ONLY_SERVER_ID, "Catalog-Only Fixture Community"
                )
                document["capabilities"].remove("games.cartridge-acquisition.v1")
                self._json(200, document)
                return
            if self.state.mode == "custom":
                document = self._discovery_document(
                    SERVER_ID, "Operator Cartridge Fixture Community"
                )
                document["capabilities"].append("games.operator-custom-cartridges.v1")
                document["capabilities"].sort()
                document["operator_custom"] = self._operator_discovery()
                self._json(200, document)
                return
            if self.state.mode in {
                "custom_modules", "custom_modules_hostile", "custom_modules_wrong_server"
            }:
                document = self._discovery_document(
                    SERVER_ID, "Custom Module Fixture Community"
                )
                document["capabilities"].append("server.operator-custom-modules.v1")
                document["capabilities"].sort()
                document["operator_custom_modules"] = self._module_disclosure()
                if self.state.mode == "custom_modules_hostile":
                    document["operator_custom_modules"]["component_bytes"] = "AGFzbQ"
                elif self.state.mode == "custom_modules_wrong_server":
                    document["operator_custom_modules"]["server_id"] = REPLACEMENT_SERVER_ID
                self._json(200, document)
                return
            self._json(200, self._discovery_document(
                SERVER_ID, "Fixture Community"
            ))
            return
        if path == "/health":
            self._require_no_authorization("health")
            if self.state.mode == "slow":
                time.sleep(0.6)
            if self.state.mode == "malformed":
                self._raw(200, b"{not-json", "application/json")
            elif self.state.mode == "wrong_identity":
                self._json(200, {
                    "service": "not-omarchygs",
                    "version": "0.1.0",
                    "status": "ok",
                    "database": "ok",
                })
            elif self.state.mode == "oversized":
                self._json(200, {
                    "service": "omarchy-gaming-system",
                    "version": "0.1.0",
                    "status": "ok",
                    "database": "ok",
                    "padding": "x" * 300_000,
                })
            else:
                self._json(200, {
                    "service": "omarchy-gaming-system",
                    "version": "0.1.0",
                    "status": "ok",
                    "database": "ok",
                })
            return
        if path == "/v1/games":
            self._require_no_authorization("game catalog")
            self._json(200, {"games": [
                {
                    "key": "signal_siege",
                    "version": 1,
                    "display_name": "Signal Siege",
                    "min_human_players": 1,
                    "max_human_players": 1,
                    "authority": "platform_compiled",
                    "provider_release_id": None,
                },
                {
                    "key": "signal_siege",
                    "version": 2,
                    "display_name": "Signal Siege Versus",
                    "min_human_players": 2,
                    "max_human_players": 2,
                    "authority": "platform_compiled",
                    "provider_release_id": None,
                },
            ]})
            return
        if path == "/v1/cartridges":
            if not self._require_social_bearer():
                return
            release = (self._operator_cartridge_release()
                       if self.state.mode == "custom" else self._cartridge_release())
            self._json(200, {"cartridges": [release]})
            return
        if path == f"/v1/mounts/{SERVER_ID}":
            if not self._require_companion_bearer():
                return
            if self.state.mode == "custom":
                mounts = ([self._operator_cartridge_mount()]
                          if self.state.cartridge_mounted else [])
                self._json(200, {
                    "mounts": [],
                    "operator_custom_mounts": mounts,
                })
            else:
                mounts = [self._cartridge_mount()] if self.state.cartridge_mounted else []
                self._json(200, {"mounts": mounts})
            return
        if path == "/v1/personas":
            token = self._bearer()
            if token == TOKEN_U:
                self._error(401, "invalid_session", "device session is invalid")
            elif token == TOKEN_M:
                self._json(200, {"personas": [
                    persona(PERSONA_A_ID, "mfa_one", "MFA One"),
                    persona(PERSONA_B_ID, "mfa_two", "MFA Two"),
                ]})
            elif token == TOKEN_X:
                leaked = persona(PERSONA_A_ID, "bad_shape", "Bad Shape")
                leaked["account_id"] = ACCOUNT_ID
                self._json(200, {"personas": [leaked]})
            elif token == TOKEN_S:
                self._json(200, {"personas": [self._social_persona(
                    SOCIAL_ACTOR_ID, "social_actor", "Social Actor"
                )]})
            elif token == TOKEN_A:
                self._json(200, {"personas": []})
            else:
                self.state.violate("persona inventory did not carry the expected bearer")
                self._error(401, "invalid_session", "device session is invalid")
            return
        if path.startswith("/v1/personas/by-handle/"):
            self._require_no_authorization("public persona lookup")
            handle = unquote(path.removeprefix("/v1/personas/by-handle/"))
            lookup = {
                "social_peer": self._social_persona(SOCIAL_PEER_ID, "social_peer", "Social Peer"),
                "social_friend": self._social_persona(SOCIAL_FRIEND_ID, "social_friend", "Social Friend"),
                "session_lost": self._social_persona(SOCIAL_LOST_ID, "session_lost", "Lost Session"),
                "malformed_peer": self._social_persona(SOCIAL_PEER_ID, "malformed_peer", "Malformed Peer"),
                "oversized_peer": self._social_persona(SOCIAL_PEER_ID, "oversized_peer", "Oversized Peer"),
                "malformed_report": self._social_persona(
                    REPORT_MALFORMED_ID, "malformed_report", "Malformed Report"
                ),
                "oversized_report": self._social_persona(
                    REPORT_OVERSIZED_ID, "oversized_report", "Oversized Report"
                ),
                "report_session_lost": self._social_persona(
                    REPORT_SESSION_LOST_ID, "report_session_lost", "Lost Report Session"
                ),
            }.get(handle)
            if lookup is None:
                self._error(404, "persona_not_found", "persona was not found")
            elif handle == "malformed_peer":
                lookup["account_id"] = ACCOUNT_ID
                self._json(200, lookup)
            elif handle == "oversized_peer":
                lookup["padding"] = "x" * 300_000
                self._json(200, lookup)
            else:
                self._json(200, lookup)
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/connection-requests":
            if not self._require_social_bearer():
                return
            incoming = [] if self.state.peer_accepted else [{
                "persona": self._social_persona(SOCIAL_PEER_ID, "social_peer", "Social Peer"),
                "created_at": CREATED_AT,
            }]
            outgoing = [] if not self.state.outgoing_present else [{
                "persona": self._social_persona(SOCIAL_OUTGOING_ID, "outgoing_peer", "Outgoing Peer"),
                "created_at": CREATED_AT,
            }]
            self._json(200, {"incoming": incoming, "outgoing": outgoing})
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/connections":
            if not self._require_social_bearer():
                return
            items = []
            if self.state.friend_connected:
                items.append({
                    "persona": self._social_persona(SOCIAL_FRIEND_ID, "social_friend", "Social Friend"),
                    "connected_at": CREATED_AT,
                })
            if self.state.peer_accepted:
                items.append({
                    "persona": self._social_persona(SOCIAL_PEER_ID, "social_peer", "Social Peer"),
                    "connected_at": CREATED_AT,
                })
            self._json(200, {"connections": items})
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/blocks":
            if not self._require_social_bearer():
                return
            items = [] if not self.state.block_present else [{
                "persona": self._social_persona(SOCIAL_BLOCKED_ID, "blocked_peer", "Blocked Peer"),
                "created_at": CREATED_AT,
            }]
            self._json(200, {"blocks": items})
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/game-sessions":
            if not self._require_social_bearer():
                return
            if query != {"limit": ["100"]}:
                self.state.violate("game session inventory did not use limit=100")
            self._json(200, {"sessions": [self._solo_session()]})
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/game-sessions/{SOLO_GAME_SESSION_ID}":
            if not self._require_social_bearer():
                return
            self._json(200, self._solo_session())
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/game-sessions/{VERSUS_GAME_SESSION_ID}":
            if not self._require_social_bearer():
                return
            self._json(200, self._versus_session())
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/game-challenges":
            if not self._require_social_bearer():
                return
            expected_queries = ({"limit": ["100"]},
                                {"limit": ["100"], "before": [OUTGOING_CHALLENGE_ID]})
            if query not in expected_queries:
                self.state.violate("challenge inventory did not use bounded pagination")
            items = []
            if self.state.outgoing_challenge_status != "absent":
                items.append(self._challenge(
                    OUTGOING_CHALLENGE_ID, "outgoing",
                    self.state.outgoing_challenge_status,
                ))
            items.append(self._challenge(
                INCOMING_CHALLENGE_ID, "incoming",
                self.state.incoming_challenge_status,
            ))
            self._json(200, {"challenges": items, "next_before": None})
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/conversations":
            if not self._require_social_bearer():
                return
            if query != {"limit": ["100"]}:
                self.state.violate("conversation inventory did not use limit=100")
            self._json(200, {"conversations": [self._conversation()]})
            return
        if path == f"/v1/personas/{SOCIAL_ACTOR_ID}/conversations/{CONVERSATION_ID}/messages":
            if not self._require_social_bearer():
                return
            before = query.get("before", [""])[0]
            if before == "2":
                self._json(200, {"messages": [self._message_one()], "next_before": None})
            else:
                messages = [self._message_two(), self._message_three()]
                if self.state.sent_body:
                    messages.append(self._message_four())
                self._json(200, {"messages": messages, "next_before": 2})
            return
        self._error(404, "fixture_not_found", "fixture route not found")

    def do_POST(self) -> None:  # noqa: N802
        self.state.record_call(f"POST {self.path}")
        document = self._read_json()
        if document is None:
            return
        if self.path == "/v1/operator-custom-trust/inspect":
            if not self._require_companion_bearer():
                return
            if set(document) != {"server_origin", "server_id"} \
                    or document.get("server_id") != SERVER_ID:
                self.state.violate("operator trust inspection was not exactly server-bound")
            self._json(200, {
                "discovery": self._operator_discovery(),
                "trusted": self.state.operator_trusted,
            })
            return
        if self.path == "/v1/operator-custom-trust":
            if not self._require_companion_bearer():
                return
            if set(document) != {"server_origin", "server_id", "confirmed_key_sha256"} \
                    or document.get("server_id") != SERVER_ID \
                    or document.get("confirmed_key_sha256") != OPERATOR_KEY_SHA256:
                self.state.violate("operator trust pin was not exactly confirmed")
            self.state.operator_trusted = True
            self._json(200, {"trust": self._operator_trust()})
            return
        if self.path == "/v1/operator-custom-trust/remove":
            if not self._require_companion_bearer():
                return
            if set(document) != {"server_origin", "server_id", "confirmed_key_sha256"} \
                    or document.get("server_id") != SERVER_ID \
                    or document.get("confirmed_key_sha256") != OPERATOR_KEY_SHA256:
                self.state.violate("operator trust removal was not exactly confirmed")
            if self.state.cartridge_mounted:
                self._json(503, {"error": {
                    "code": "companion_operator_custom_untrusted",
                }})
                return
            removed = self.state.operator_trusted
            self.state.operator_trusted = False
            self._json(200, {"removed": removed})
            return
        if self.path == "/v1/acquisitions":
            if not self._require_companion_bearer():
                return
            if set(document) != {
                "server_origin", "server_id", "device_bearer", "game_key",
                "archive_sha256", "admission_revision", "provenance_class"
            }:
                self.state.violate("cartridge acquisition body did not have exact keys")
            expected_provenance = ("operator_custom" if self.state.mode == "custom"
                                   else "marketplace_vetted")
            if document.get("server_id") != SERVER_ID \
                    or document.get("device_bearer") != TOKEN_S \
                    or document.get("game_key") != "door-legends" \
                    or document.get("archive_sha256") != CARTRIDGE_DIGEST \
                    or document.get("admission_revision") != 3 \
                    or document.get("provenance_class") != expected_provenance:
                self.state.violate("cartridge acquisition did not preserve exact authority")
            self.state.cartridge_mounted = True
            mount = (self._operator_cartridge_mount()
                     if self.state.mode == "custom" else self._cartridge_mount())
            self._json(200, {"mount": mount})
            return
        if self.path == "/v1/removals":
            if not self._require_companion_bearer():
                return
            if set(document) != {"server_id", "game_key", "archive_sha256",
                                 "admission_revision", "provenance_class"}:
                self.state.violate("cartridge removal body did not have exact keys")
            expected_provenance = ("operator_custom" if self.state.mode == "custom"
                                   else "marketplace_vetted")
            if document.get("server_id") != SERVER_ID \
                    or document.get("game_key") != "door-legends" \
                    or document.get("archive_sha256") != CARTRIDGE_DIGEST \
                    or document.get("admission_revision") != 3 \
                    or document.get("provenance_class") != expected_provenance:
                self.state.violate("cartridge removal crossed profile authority")
            self.state.cartridge_mounted = False
            self._json(200, {"removed": True})
            return
        if self.path == "/v1/session-acquisitions":
            if not self._require_companion_bearer():
                return
            if set(document) != {"server_origin", "server_id", "device_bearer",
                                 "persona_id", "game_session_id", "provenance_class"}:
                self.state.violate("session acquisition body did not have exact keys")
            if document.get("server_id") != SERVER_ID \
                    or document.get("device_bearer") != TOKEN_S \
                    or document.get("persona_id") != SOCIAL_ACTOR_ID \
                    or document.get("game_session_id") != SOLO_GAME_SESSION_ID \
                    or document.get("provenance_class") != "marketplace_vetted":
                self.state.violate("session acquisition did not preserve participant authority")
            self.state.cartridge_mounted = True
            self._json(200, {"mount": self._cartridge_mount()})
            return
        if self.path == "/v1/render-plans":
            if not self._require_companion_bearer():
                return
            if not self.state.cartridge_mounted:
                self._error(404, "companion_mount_missing", "exact mount is absent")
                return
            allowed = {"server_origin", "server_id", "game_key", "archive_sha256",
                       "admission_revision", "lifecycle_status", "active_session_policy",
                       "provenance_class", "view", "preferences"}
            if "screen_id" in document:
                allowed.add("screen_id")
            if set(document) != allowed:
                self.state.violate("render-plan body did not have exact keys")
            if document.get("provenance_class") != "marketplace_vetted":
                self.state.violate("render-plan body did not preserve cartridge provenance")
            screen_id = document.get("screen_id", "lobby")
            if screen_id not in {"lobby", "chronicle"}:
                self._error(422, "companion_render_failure", "unknown signed screen")
                return
            self._json(200, self._cartridge_render(screen_id))
            return
        if self.path == "/v1/accounts":
            self._require_no_authorization("account registration")
            if set(document) != {"invite_code", "username", "password"}:
                self.state.violate("registration body did not have exact keys")
            invite_code = str(document.get("invite_code", ""))
            username = str(document.get("username", "")).strip().lower()
            if invite_code == "ogsi_invalid_fixture":
                self._error(403, "invalid_invitation", "registration invitation is invalid")
            elif invite_code != INVITE_CODE:
                self.state.violate("registration did not send the expected invitation code")
                self._error(403, "invalid_invitation", "registration invitation is invalid")
            elif username == "taken_user":
                self._error(409, "username_taken", "username is already registered")
            elif username == "malformed_register":
                self._json(201, {"id": ACCOUNT_ID, "username": username, "password_hash": "no"})
            else:
                self._json(201, {"id": ACCOUNT_ID, "username": username})
            return
        if self.path == "/v1/sessions":
            self._require_no_authorization("primary login")
            if set(document) != {"username", "password", "device_name"}:
                self.state.violate("session body did not have exact keys")
            username = str(document.get("username", "")).strip().lower()
            if username == "bad_login":
                self._error(401, "invalid_credentials", "credentials are invalid")
            elif username == "mfa_user":
                self._json(202, {
                    "mfa_required": True,
                    "challenge_token": CHALLENGE,
                    "expires_at": EXPIRES_AT,
                })
            elif username == "malformed_login":
                self._json(201, self._session("not-a-token"))
            elif username == "unauthorized_user":
                self._json(201, self._session(TOKEN_U))
            elif username == "malformed_personas":
                self._json(201, self._session(TOKEN_X))
            elif username == "social_user":
                self._json(201, self._session(TOKEN_S))
            else:
                self._json(201, self._session(TOKEN_A))
            return
        if self.path == "/v1/sessions/mfa":
            self._require_no_authorization("MFA completion")
            if set(document) != {"challenge_token", "code"}:
                self.state.violate("MFA body did not have exact keys")
            if document.get("challenge_token") != CHALLENGE:
                self.state.violate("MFA completion did not return the issued challenge")
            factor = str(document.get("code", ""))
            if factor == "000000":
                self._error(401, "invalid_mfa_code", "MFA code is invalid")
            elif factor == "EXPIRED":
                self._error(401, "invalid_mfa_challenge", "MFA challenge is invalid")
            else:
                self._json(201, self._session(TOKEN_M))
            return
        if self.path == "/v1/personas":
            token = self._bearer()
            if token != TOKEN_A:
                self.state.violate("persona creation did not carry the normal bearer")
                self._error(401, "invalid_session", "device session is invalid")
                return
            if set(document) != {"handle", "display_name", "bio", "status_message"}:
                self.state.violate("persona creation body did not have exact keys")
            if document.get("handle") == "taken_handle":
                self._error(409, "handle_taken", "handle is already registered")
                return
            response = persona(
                PERSONA_A_ID,
                str(document.get("handle", "")).strip().lower(),
                str(document.get("display_name", "")).strip(),
            )
            response["bio"] = str(document.get("bio", ""))
            response["status_message"] = str(document.get("status_message", "")).strip()
            self._json(201, response)
            return
        if self.path == f"/v1/personas/{SOCIAL_ACTOR_ID}/game-sessions":
            if not self._require_social_bearer():
                return
            if set(document) != {"idempotency_key", "game_key", "game_version"}:
                self.state.violate("solo start body did not have exact keys")
            if document.get("game_key") != "signal_siege" or document.get("game_version") != 1:
                self.state.violate("solo start did not select Signal Siege v1")
            self._json(201, self._solo_session())
            return
        if self.path == f"/v1/personas/{SOCIAL_ACTOR_ID}/reports":
            if not self._require_social_bearer():
                return
            if set(document) != {"idempotency_key", "subject_persona_id", "category", "detail"}:
                self.state.violate("persona report body did not have exact keys")
            subject_id = document.get("subject_persona_id")
            idempotency_key = document.get("idempotency_key")
            if not isinstance(idempotency_key, str) or len(idempotency_key) != 36:
                self.state.violate("persona report did not carry a UUID idempotency key")
            elif isinstance(subject_id, str):
                earlier_key = self.state.report_attempt_keys.get(subject_id)
                if earlier_key is not None and earlier_key != idempotency_key:
                    self.state.violate("persona report retry replaced its idempotency key")
                self.state.report_attempt_keys[subject_id] = idempotency_key
            if subject_id == REPORT_SESSION_LOST_ID:
                self._error(401, "invalid_session", "device session is invalid")
                return
            if subject_id == REPORT_MALFORMED_ID:
                self._json(201, {
                    "id": REPORT_ID,
                    "idempotency_key": document.get("idempotency_key"),
                    "status": "open",
                    "created_at": CREATED_AT,
                    "subject_persona_id": subject_id,
                })
                return
            if subject_id == REPORT_OVERSIZED_ID:
                self._json(201, {
                    "id": REPORT_ID,
                    "idempotency_key": document.get("idempotency_key"),
                    "status": "open",
                    "created_at": CREATED_AT,
                    "padding": "x" * 300_000,
                })
                return
            if subject_id != SOCIAL_FRIEND_ID:
                self.state.violate("persona report did not resolve the exact subject")
            if document.get("category") != "cheating" \
                    or document.get("detail") != "Fixture report detail":
                self.state.violate("persona report did not preserve category and trimmed detail")
            self.state.report_body = document
            self._json(201, {
                "id": REPORT_ID,
                "idempotency_key": idempotency_key,
                "status": "open",
                "created_at": CREATED_AT,
            })
            return
        if self.path == f"/v1/personas/{SOCIAL_ACTOR_ID}/game-challenges":
            if not self._require_social_bearer():
                return
            if set(document) != {"idempotency_key", "challenged_persona_id",
                                 "game_key", "game_version"}:
                self.state.violate("challenge body did not have exact keys")
            if document.get("challenged_persona_id") != SOCIAL_FRIEND_ID \
                    or document.get("game_key") != "signal_siege" \
                    or document.get("game_version") != 2:
                self.state.violate("challenge did not derive the connected target and versus game")
            self.state.outgoing_challenge_status = "pending"
            self._json(201, self._challenge(OUTGOING_CHALLENGE_ID, "outgoing", "pending"))
            return
        if self.path == (f"/v1/personas/{SOCIAL_ACTOR_ID}/game-sessions/"
                         f"{SOLO_GAME_SESSION_ID}/commands"):
            if not self._require_social_bearer():
                return
            self._apply_solo_command(document)
            return
        if self.path == (f"/v1/personas/{SOCIAL_ACTOR_ID}/game-sessions/"
                         f"{VERSUS_GAME_SESSION_ID}/commands"):
            if not self._require_social_bearer():
                return
            self._apply_versus_command(document)
            return
        if self.path == f"/v1/personas/{SOCIAL_ACTOR_ID}/conversations/{CONVERSATION_ID}/messages":
            if not self._require_social_bearer():
                return
            if set(document) != {"body"}:
                self.state.violate("private message body did not have exact keys")
            self.state.sent_body = str(document.get("body", ""))
            self._json(201, self._message_four())
            return
        self._error(404, "fixture_not_found", "fixture route not found")

    def do_PUT(self) -> None:  # noqa: N802
        self.state.record_call(f"PUT {self.path}")
        if not self._require_social_bearer():
            return
        if self.headers.get("Content-Length") not in {None, "0"}:
            self.state.violate("bodyless social PUT carried a request body")
        prefix = f"/v1/personas/{SOCIAL_ACTOR_ID}"
        if self.path == f"{prefix}/game-challenges/{INCOMING_CHALLENGE_ID}/accept":
            self.state.incoming_challenge_status = "accepted"
            self._json(200, self._challenge(INCOMING_CHALLENGE_ID, "incoming", "accepted"))
            return
        if self.path == f"{prefix}/game-challenges/{INCOMING_CHALLENGE_ID}/decline":
            self.state.incoming_challenge_status = "declined"
            self._json(200, self._challenge(INCOMING_CHALLENGE_ID, "incoming", "declined"))
            return
        if self.path == f"{prefix}/connections/{SOCIAL_PEER_ID}":
            self.state.peer_accepted = True
            self._json(200, {
                "persona": self._social_persona(SOCIAL_PEER_ID, "social_peer", "Social Peer"),
                "connected_at": CREATED_AT,
            })
            return
        if self.path == f"{prefix}/connection-requests/{SOCIAL_PEER_ID}":
            self._json(201, {
                "persona": self._social_persona(SOCIAL_PEER_ID, "social_peer", "Social Peer"),
                "created_at": CREATED_AT,
            })
            return
        if self.path == f"{prefix}/connection-requests/{SOCIAL_LOST_ID}":
            self._error(401, "invalid_session", "device session is invalid")
            return
        if self.path == f"{prefix}/blocks/{SOCIAL_FRIEND_ID}":
            self.state.friend_connected = False
            self._json(201, {
                "persona": self._social_persona(SOCIAL_FRIEND_ID, "social_friend", "Social Friend"),
                "created_at": CREATED_AT,
            })
            return
        if self.path == f"{prefix}/conversations/{CONVERSATION_ID}/read/{MESSAGE_3_ID}":
            self.state.read_message_id = MESSAGE_3_ID
            self._json(200, {"through_message_id": MESSAGE_3_ID, "unread_count": 0})
            return
        self._error(404, "fixture_not_found", "fixture route not found")

    def do_DELETE(self) -> None:  # noqa: N802
        self.state.record_call(f"DELETE {self.path}")
        if self.path == "/v1/operator-custom-trust":
            document = self._read_json()
            if document is None:
                return
            if not self._require_companion_bearer():
                return
            if set(document) != {"server_origin", "server_id", "confirmed_key_sha256"} \
                    or document.get("server_id") != SERVER_ID \
                    or document.get("confirmed_key_sha256") != OPERATOR_KEY_SHA256:
                self.state.violate("operator trust removal was not exactly confirmed")
            if self.state.cartridge_mounted:
                self._json(503, {"error": {
                    "code": "companion_operator_custom_untrusted",
                }})
                return
            removed = self.state.operator_trusted
            self.state.operator_trusted = False
            self._json(200, {"removed": removed})
            return
        if not self._require_social_bearer():
            return
        prefix = f"/v1/personas/{SOCIAL_ACTOR_ID}"
        if self.path == f"{prefix}/game-challenges/{OUTGOING_CHALLENGE_ID}":
            self.state.outgoing_challenge_status = "cancelled"
            self._json(200, self._challenge(OUTGOING_CHALLENGE_ID, "outgoing", "cancelled"))
            return
        if self.path == f"{prefix}/connections/{SOCIAL_OUTGOING_ID}":
            self.state.outgoing_present = False
            self._raw(204, b"", "application/json")
            return
        if self.path == f"{prefix}/connections/{SOCIAL_PEER_ID}":
            self.state.peer_accepted = False
            self._raw(204, b"", "application/json")
            return
        if self.path == f"{prefix}/blocks/{SOCIAL_BLOCKED_ID}":
            self.state.block_present = False
            self._raw(204, b"", "application/json")
            return
        self._raw(204, b"", "application/json")

    def _social_persona(self, persona_id: str, handle: str, display_name: str) -> dict[str, Any]:
        return persona(persona_id, handle, display_name)

    def _conversation(self) -> dict[str, Any]:
        return {
            "id": CONVERSATION_ID,
            "other_persona": self._social_persona(
                SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
            ),
            "unread_count": 2,
            "latest_message": self._message_three(),
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT,
        }

    def _solo_session(self) -> dict[str, Any]:
        return {
            "id": SOLO_GAME_SESSION_ID,
            "game_key": "signal_siege",
            "game_version": 1,
            "revision": self.state.game_revision,
            "status": "active",
            "state": self.state.game_state,
            "authority": "platform_compiled",
            "provider_release_id": None,
            "availability": None,
            "presentation": None,
            "result": None,
            "participants": [{
                "seat": 0,
                "persona": self._social_persona(
                    SOCIAL_ACTOR_ID, "social_actor", "Social Actor"
                ),
            }],
            "completed_at": None,
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT,
        }

    def _versus_session(self) -> dict[str, Any]:
        return {
            "id": VERSUS_GAME_SESSION_ID,
            "game_key": "signal_siege",
            "game_version": 2,
            "revision": self.state.versus_revision,
            "status": "active",
            "state": self.state.versus_state,
            "authority": "platform_compiled",
            "provider_release_id": None,
            "availability": None,
            "presentation": None,
            "result": None,
            "participants": [
                {
                    "seat": 0,
                    "persona": self._social_persona(
                        SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
                    ),
                },
                {
                    "seat": 1,
                    "persona": self._social_persona(
                        SOCIAL_ACTOR_ID, "social_actor", "Social Actor"
                    ),
                },
            ],
            "completed_at": None,
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT,
        }

    def _challenge(self, challenge_id: str, direction: str, status: str) -> dict[str, Any]:
        challenger = self._social_persona(
            SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
        ) if direction == "incoming" else self._social_persona(
            SOCIAL_ACTOR_ID, "social_actor", "Social Actor"
        )
        challenged = self._social_persona(
            SOCIAL_ACTOR_ID, "social_actor", "Social Actor"
        ) if direction == "incoming" else self._social_persona(
            SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
        )
        return {
            "id": challenge_id,
            "game_key": "signal_siege",
            "game_version": 2,
            "direction": direction,
            "status": status,
            "challenger": challenger,
            "challenged": challenged,
            "game_session_id": VERSUS_GAME_SESSION_ID if status == "accepted" else None,
            "expires_at": EXPIRES_AT,
            "resolved_at": None if status == "pending" else CREATED_AT,
            "created_at": CREATED_AT,
            "updated_at": CREATED_AT,
        }

    def _apply_solo_command(self, document: dict[str, Any]) -> None:
        if set(document) != {"idempotency_key", "expected_revision", "command"}:
            self.state.violate("solo command body did not have exact keys")
        command = document.get("command", {})
        if document.get("expected_revision") != self.state.game_revision:
            self._error(409, "game_revision_conflict", "game revision changed")
            return
        if not isinstance(command, dict) or set(command) != {"kind", "action"}:
            self.state.violate("solo command did not derive the loaded revision and exact action")
        action = str(command.get("action", ""))
        energy = int(self.state.game_state["human"]["energy"])
        if action in {"strike", "guard"} and energy < 1:
            self._error(422, "game_command_rejected", "game command was rejected")
            return
        if action == "charge":
            self.state.game_state["human"]["energy"] = min(4, energy + 2)
        else:
            self.state.game_state["human"]["energy"] = energy - 1
        if action == "strike":
            self.state.game_state["bot"]["core"] = max(
                0, int(self.state.game_state["bot"]["core"]) - 2
            )
        self.state.game_state["bot"]["energy"] = min(
            4, int(self.state.game_state["bot"]["energy"]) + 2
        )
        self.state.game_state["round"] += 1
        self.state.game_state["last_round"] = {
            "round": self.state.game_state["round"],
            "human_action": action,
            "bot_action": "charge",
            "damage_to_human": 0,
            "damage_to_bot": 2 if action == "strike" else 0,
        }
        self.state.game_revision += 1
        self._json(200, self._command_response(
            SOLO_GAME_SESSION_ID, self.state.game_revision, self.state.game_state
        ))

    def _apply_versus_command(self, document: dict[str, Any]) -> None:
        if set(document) != {"idempotency_key", "expected_revision", "command"}:
            self.state.violate("versus command body did not have exact keys")
        command = document.get("command", {})
        action = str(command.get("action", "")) if isinstance(command, dict) else ""
        if document.get("expected_revision") != self.state.versus_revision \
                or action not in {"strike", "guard", "charge"}:
            self.state.violate("versus command did not derive loaded authority")
        player = self.state.versus_state["players"][1]
        if action == "charge":
            player["energy"] = min(4, int(player["energy"]) + 2)
        else:
            player["energy"] = int(player["energy"]) - 1
        if action == "strike":
            self.state.versus_state["players"][0]["core"] -= 2
        if action == "guard":
            player["guard"] = 2
        self.state.versus_state["turn"] = 2
        self.state.versus_state["active_seat"] = 0
        self.state.versus_state["last_turn"] = {
            "turn": 2,
            "actor_seat": 1,
            "action": action,
            "damage_to_opponent": 2 if action == "strike" else 0,
            "blocked_damage": 0,
        }
        self.state.versus_revision += 1
        self._json(200, self._command_response(
            VERSUS_GAME_SESSION_ID, self.state.versus_revision, self.state.versus_state
        ))

    @staticmethod
    def _command_response(session_id: str, revision: int,
                          state: dict[str, Any]) -> dict[str, Any]:
        return {
            "game_session_id": session_id,
            "revision": revision,
            "status": "active",
            "state": state,
            "authority": "platform_compiled",
            "provider_release_id": None,
            "availability": None,
        }

    def _message_one(self) -> dict[str, Any]:
        return {
            "type": "system",
            "id": MESSAGE_1_ID,
            "sequence": 1,
            "system": {
                "type": "connection_accepted",
                "actor": self._social_persona(
                    SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
                ),
            },
            "created_at": CREATED_AT,
        }

    def _message_two(self) -> dict[str, Any]:
        return {
            "type": "user",
            "id": MESSAGE_2_ID,
            "sequence": 2,
            "sender": self._social_persona(
                SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
            ),
            "body": "Fixture hello <b>as plain text</b>",
            "created_at": CREATED_AT,
        }

    def _message_three(self) -> dict[str, Any]:
        return {
            "type": "system",
            "id": MESSAGE_3_ID,
            "sequence": 3,
            "system": {
                "type": "game_challenge_created",
                "actor": self._social_persona(
                    SOCIAL_FRIEND_ID, "social_friend", "Social Friend"
                ),
                "challenge_id": SOCIAL_LOST_ID,
            },
            "created_at": CREATED_AT,
        }

    def _message_four(self) -> dict[str, Any]:
        return {
            "type": "user",
            "id": MESSAGE_4_ID,
            "sequence": 4,
            "sender": self._social_persona(
                SOCIAL_ACTOR_ID, "social_actor", "Social Actor"
            ),
            "body": self.state.sent_body,
            "created_at": CREATED_AT,
        }

    def _require_social_bearer(self) -> bool:
        if self._bearer() == TOKEN_S:
            return True
        self.state.violate("social request did not carry the expected bearer")
        self._error(401, "invalid_session", "device session is invalid")
        return False

    def _require_companion_bearer(self) -> bool:
        if self._bearer() == COMPANION_TOKEN:
            return True
        self.state.violate("companion request did not carry its local bearer")
        self._error(401, "companion_unauthorized", "companion authorization failed")
        return False

    def _cartridge_release(self) -> dict[str, Any]:
        return {
            "game_key": "door-legends",
            "publisher_id": "ignibyte",
            "rules_version": 1,
            "cartridge_version": 2,
            "display_name": "Door Legends",
            "archive_sha256": CARTRIDGE_DIGEST,
            "signed_identity_sha256": CARTRIDGE_IDENTITY,
            "marketplace": {
                "provenance_class": "marketplace_vetted",
                "marketplace_id": "omarchygs-marketplace",
                "marketplace_name": "OmarchyGS Marketplace",
                "reviewed_by": "review-team",
                "review_summary": "Bounded first-party review passed.",
                "policy_version": 1,
                "lifecycle_status": "active",
            },
            "server_admission": {"revision": 3},
        }

    def _cartridge_mount(self) -> dict[str, Any]:
        release = self._cartridge_release()
        return {
            "format": "omarchygs.client-cartridge-mount/v1",
            "server_id": SERVER_ID,
            "server_origin": f"http://{self.headers.get('Host', '')}",
            "game_key": release["game_key"],
            "publisher_id": release["publisher_id"],
            "rules_version": release["rules_version"],
            "cartridge_version": release["cartridge_version"],
            "display_name": release["display_name"],
            "archive_sha256": release["archive_sha256"],
            "signed_identity_sha256": release["signed_identity_sha256"],
            "marketplace_key_sha256": "d" * 64,
            "marketplace_id": release["marketplace"]["marketplace_id"],
            "marketplace_name": release["marketplace"]["marketplace_name"],
            "reviewed_by": release["marketplace"]["reviewed_by"],
            "review_summary": release["marketplace"]["review_summary"],
            "snapshot_version": 1,
            "policy_version": release["marketplace"]["policy_version"],
            "lifecycle_status": release["marketplace"]["lifecycle_status"],
            "admission_revision": release["server_admission"]["revision"],
        }

    @staticmethod
    def _operator_discovery() -> dict[str, Any]:
        return {
            "operator_name": "Fixture Server Operator",
            "authority_id": "fixture-operator",
            "key_id": "fixture-key",
            "key_sha256": OPERATOR_KEY_SHA256,
            "public_key": {
                "format_version": 1,
                "algorithm": "ed25519",
                "key_id": "fixture-key",
                "authority_id": "fixture-operator",
                "verifying_key": "K" * 43,
            },
        }

    def _operator_trust(self) -> dict[str, Any]:
        discovery = self._operator_discovery()
        return {
            "format": "omarchygs.client-operator-custom-trust/v1",
            "server_id": SERVER_ID,
            "server_origin": f"http://{self.headers.get('Host', '')}",
            "operator_name": discovery["operator_name"],
            "key_sha256": discovery["key_sha256"],
            "public_key": discovery["public_key"],
        }

    def _operator_cartridge_release(self) -> dict[str, Any]:
        discovery = self._operator_discovery()
        return {
            "game_key": "door-legends",
            "publisher_id": "ignibyte",
            "rules_version": 1,
            "cartridge_version": 2,
            "display_name": "Door Legends Operator Edition",
            "archive_sha256": CARTRIDGE_DIGEST,
            "signed_identity_sha256": CARTRIDGE_IDENTITY,
            "operator_custom": {
                "provenance_class": "operator_custom",
                "operator_name": discovery["operator_name"],
                "authority_id": discovery["authority_id"],
                "key_id": discovery["key_id"],
                "key_sha256": discovery["key_sha256"],
                "warning": OPERATOR_WARNING,
                "policy_version": 1,
                "lifecycle_status": "active",
            },
            "server_admission": {"revision": 3},
            "warning": OPERATOR_WARNING,
        }

    def _operator_cartridge_mount(self) -> dict[str, Any]:
        release = self._operator_cartridge_release()
        custom = release["operator_custom"].copy()
        custom.pop("policy_version")
        custom.pop("lifecycle_status")
        return {
            "format": "omarchygs.client-operator-custom-mount/v1",
            "server_id": SERVER_ID,
            "server_origin": f"http://{self.headers.get('Host', '')}",
            "game_key": release["game_key"],
            "publisher_id": release["publisher_id"],
            "rules_version": release["rules_version"],
            "cartridge_version": release["cartridge_version"],
            "display_name": release["display_name"],
            "archive_sha256": release["archive_sha256"],
            "signed_identity_sha256": release["signed_identity_sha256"],
            "operator_custom": custom,
            "policy_version": 1,
            "lifecycle_status": "active",
            "admission_revision": 3,
            "warning": OPERATOR_WARNING,
        }

    def _cartridge_render(self, screen_id: str) -> dict[str, Any]:
        target = "chronicle" if screen_id == "lobby" else "lobby"
        navigation_action = f"navigate.{target}"
        navigation_label = ("Read the chronicle" if target == "chronicle"
                            else "Return to the lobby")
        return {
            "format": "omarchygs.session-cartridge-render/v2",
            "screen_id": screen_id,
            "entry_screen_id": "lobby",
            "navigation": [{
                "action": navigation_action,
                "target_screen": target,
            }],
            "plan": {
                "format": "omarchygs.render-plan/v1",
                "profile": "rich2d",
                "state": "ready",
                "state_message": "Ready",
                "origin": {
                    "publisher_id": "ignibyte",
                    "game_key": "door-legends",
                    "cartridge_version": 2,
                    "archive_sha256": CARTRIDGE_DIGEST,
                },
                "title": ("Door Legends" if screen_id == "lobby"
                          else "Door Legends Chronicle"),
                "preferences": {
                    "scale": 1.0,
                    "high_contrast": False,
                    "reduced_motion": False,
                    "muted_audio": False,
                },
                "nodes": [{
                    "kind": "button",
                    "id": f"{screen_id}_enter",
                    "label": "Enter the brass door",
                    "action": "enter",
                    "accessible_label": "Enter Door Legends",
                }, {
                    "kind": "button",
                    "id": f"{screen_id}_navigation",
                    "label": navigation_label,
                    "action": navigation_action,
                    "accessible_label": navigation_label,
                }],
                "requested_actions_are_unconfirmed": True,
            },
            "asset_base_url": (f"http://{self.headers.get('Host', '')}"
                               f"/v1/render-assets/{'C' * 43}"),
        }

    def _session(self, token: str) -> dict[str, Any]:
        return {
            "token": token,
            "session": {
                "id": SESSION_ID,
                "device_name": "Omarchy QML",
                "created_at": CREATED_AT,
                "last_used_at": CREATED_AT,
                "expires_at": EXPIRES_AT,
                "revoked_at": None,
                "current": True,
            },
        }

    @staticmethod
    def _discovery_document(server_id: str, server_name: str) -> dict[str, Any]:
        return {
            "service": "omarchy-gaming-system",
            "server_id": server_id,
            "server_name": server_name,
            "protocol_version": 1,
            "capabilities": DISCOVERY_CAPABILITIES.copy(),
        }

    @staticmethod
    def _module_disclosure() -> dict[str, Any]:
        return {
            "format": "omarchygs.operator-custom-modules-disclosure/v1",
            "server_id": SERVER_ID,
            "active_count": 1,
            "behavior_capabilities": ["moderation_labels"],
            "warning": (
                "This server runs operator-custom code not reviewed or supported "
                "by OmarchyGS."
            ),
            "support_boundary": (
                "Security, privacy, availability, and support are the server "
                "operator's responsibility."
            ),
        }

    def _require_no_authorization(self, context: str) -> None:
        if self.headers.get("Authorization") is not None:
            self.state.violate(f"{context} unexpectedly carried Authorization")

    def _bearer(self) -> str:
        value = self.headers.get("Authorization", "")
        if not value.startswith("Bearer ") or value.count(" ") != 1:
            return ""
        return value.removeprefix("Bearer ")

    def _read_json(self) -> dict[str, Any] | None:
        content_type = self.headers.get("Content-Type", "")
        if not content_type.startswith("application/json"):
            self.state.violate("JSON request did not carry application/json")
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            self.state.violate("request content length was invalid")
            self._error(400, "fixture_invalid", "invalid fixture request")
            return None
        if length < 1 or length > 16_384:
            self.state.violate("request body was empty or exceeded fixture bound")
            self._error(400, "fixture_invalid", "invalid fixture request")
            return None
        body = self.rfile.read(length)
        try:
            document = json.loads(body)
        except (UnicodeDecodeError, json.JSONDecodeError):
            self.state.violate("request body was not valid JSON")
            self._error(400, "fixture_invalid", "invalid fixture request")
            return None
        if not isinstance(document, dict):
            self.state.violate("request body was not a JSON object")
            self._error(400, "fixture_invalid", "invalid fixture request")
            return None
        return document

    def _error(self, status: int, code: str, message: str) -> None:
        self._json(status, {"error": {"code": code, "message": message}})

    def _json(self, status: int, document: dict[str, Any]) -> None:
        body = json.dumps(document, separators=(",", ":")).encode("utf-8")
        self._raw(status, body, "application/json")

    def _raw(self, status: int, body: bytes, content_type: str) -> None:
        try:
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            return


def write_config(config_path: Path, document: dict[str, str]) -> None:
    config_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    if config_path.parent.is_symlink() or not config_path.parent.is_dir():
        raise OSError("fixture config parent must be a directory, not a symlink")
    os.chmod(config_path.parent, 0o700)
    open_flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC
    if hasattr(os, "O_NOFOLLOW"):
        open_flags |= os.O_NOFOLLOW
    descriptor = os.open(config_path, open_flags, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as config_file:
        json.dump(document, config_file, separators=(",", ":"))
        config_file.write("\n")
    os.chmod(config_path, 0o600)


def main() -> int:
    if len(sys.argv) == 3 and sys.argv[1] == "--write-config":
        config_path = Path(sys.argv[2])
        try:
            document = json.load(sys.stdin)
        except json.JSONDecodeError as error:
            print(f"invalid fixture config JSON: {error}", file=sys.stderr)
            return 2
        if not isinstance(document, dict) or not all(
            isinstance(key, str) and isinstance(value, str)
            for key, value in document.items()
        ):
            print("fixture config must be a string map", file=sys.stderr)
            return 2
        write_config(config_path, document)
        return 0

    if len(sys.argv) == 3 and sys.argv[1] == "--write-live-config":
        config_path = Path(sys.argv[2])
        values = sys.stdin.buffer.read().split(b"\0")
        if values and values[-1] == b"":
            values.pop()
        if len(values) != 11:
            print("live fixture config requires eleven NUL-delimited values", file=sys.stderr)
            return 2
        try:
            decoded = [value.decode("utf-8") for value in values]
        except UnicodeDecodeError:
            print("live fixture config must be UTF-8", file=sys.stderr)
            return 2
        document = dict(zip(
            ["server_url", "scenario", "username", "password", "persona_handle", "factor",
             "peer_handle", "message_body", "peer_username", "peer_password", "invite_code"],
            decoded,
            strict=True,
        ))
        write_config(config_path, document)
        return 0

    if len(sys.argv) != 3:
        print("usage: fixture_server.py <port-file> <mode>", file=sys.stderr)
        return 2
    port_file = Path(sys.argv[1])
    mode = sys.argv[2]
    if mode not in {
        "normal", "server_two", "catalog_only", "identity_changed", "incompatible", "slow",
        "custom_modules", "custom_modules_hostile", "custom_modules_wrong_server",
        "malformed", "wrong_identity", "oversized", "custom",
    }:
        print(f"unsupported fixture mode: {mode}", file=sys.stderr)
        return 2

    fixture_state = FixtureState(mode)
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.fixture_state = fixture_state  # type: ignore[attr-defined]
    port_file.write_text(str(server.server_address[1]), encoding="ascii")
    try:
        server.serve_forever(poll_interval=0.05)
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
