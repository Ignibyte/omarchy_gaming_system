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


ACCOUNT_ID = "11111111-1111-4111-8111-111111111111"
SESSION_ID = "22222222-2222-4222-8222-222222222222"
PERSONA_A_ID = "33333333-3333-4333-8333-333333333333"
PERSONA_B_ID = "44444444-4444-4444-8444-444444444444"
TOKEN_A = "ogs1_" + "A" * 43
TOKEN_M = "ogs1_" + "M" * 43
TOKEN_U = "ogs1_" + "U" * 43
TOKEN_X = "ogs1_" + "X" * 43
CHALLENGE = "ogm1_" + "C" * 43
CREATED_AT = "2026-08-25T20:00:00.000Z"
EXPIRES_AT = "2099-08-25T20:05:00.000Z"


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

    def record_call(self, call: str) -> None:
        with self.lock:
            self.calls.append(call)

    def violate(self, message: str) -> None:
        with self.lock:
            self.violations.append(message)


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
        if self.path == "/__fixture__/status":
            self._json(200, {"calls": self.state.calls, "violations": self.state.violations})
            return
        if self.path == "/health":
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
        if self.path == "/v1/personas":
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
            elif token == TOKEN_A:
                self._json(200, {"personas": []})
            else:
                self.state.violate("persona inventory did not carry the expected bearer")
                self._error(401, "invalid_session", "device session is invalid")
            return
        self._error(404, "fixture_not_found", "fixture route not found")

    def do_POST(self) -> None:  # noqa: N802
        self.state.record_call(f"POST {self.path}")
        document = self._read_json()
        if document is None:
            return
        if self.path == "/v1/accounts":
            self._require_no_authorization("account registration")
            if set(document) != {"username", "password"}:
                self.state.violate("registration body did not have exact keys")
            username = str(document.get("username", "")).strip().lower()
            if username == "taken_user":
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
        self._error(404, "fixture_not_found", "fixture route not found")

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
        if len(values) != 6:
            print("live fixture config requires six NUL-delimited values", file=sys.stderr)
            return 2
        try:
            decoded = [value.decode("utf-8") for value in values]
        except UnicodeDecodeError:
            print("live fixture config must be UTF-8", file=sys.stderr)
            return 2
        document = dict(zip(
            ["server_url", "scenario", "username", "password", "persona_handle", "factor"],
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
    if mode not in {"normal", "slow", "malformed", "wrong_identity", "oversized"}:
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
