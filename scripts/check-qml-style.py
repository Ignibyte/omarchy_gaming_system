#!/usr/bin/env python3
"""Enforce the trusted QML visual and plain-text boundary."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
QML_ROOT = ROOT / "client" / "qml"
THEME = QML_ROOT / "components" / "OgsTheme.qml"
SCOPED_FILES = [QML_ROOT / "Main.qml", QML_ROOT / "game" / "SignalSiegeSurface.qml"]
SCOPED_FILES.extend(sorted((QML_ROOT / "components").glob("*.qml")))
SCOPED_FILES.extend(sorted((QML_ROOT / "screens").glob("*.qml")))
SCOPED_FILES.extend(sorted((QML_ROOT / "cartridge").glob("*.qml")))
SCOPED_FILES.extend(sorted((QML_ROOT / "cartridge" / "nodes").glob("*.qml")))

HEX_COLOR = re.compile(r"#[0-9a-fA-F]{6}(?![0-9a-fA-F])")
TEXT_BLOCK = re.compile(r"(?<![\w.])Text\s*\{")
UNSAFE_TEXT_FORMATS = ("Text.RichText", "Text.AutoText", "Text.StyledText", "Text.MarkdownText")
REQUIRED_THEME_TOKENS = (
    "background",
    "surface",
    "surfaceRaised",
    "textPrimary",
    "textSecondary",
    "textMuted",
    "accent",
    "warning",
    "danger",
    "focus",
    "highContrastBackground",
    "highContrastForeground",
    "focusWidth",
    "controlHeight",
)


def line_number(source: str, offset: int) -> int:
    return source.count("\n", 0, offset) + 1


def qml_block(source: str, opening_brace: int) -> str:
    depth = 0
    quote = ""
    escaped = False
    line_comment = False
    block_comment = False
    index = opening_brace

    while index < len(source):
        char = source[index]
        following = source[index + 1] if index + 1 < len(source) else ""

        if line_comment:
            if char == "\n":
                line_comment = False
        elif block_comment:
            if char == "*" and following == "/":
                block_comment = False
                index += 1
        elif quote:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = ""
        elif char == "/" and following == "/":
            line_comment = True
            index += 1
        elif char == "/" and following == "*":
            block_comment = True
            index += 1
        elif char in ('"', "'"):
            quote = char
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return source[opening_brace : index + 1]
        index += 1

    return source[opening_brace:]


def main() -> int:
    failures: list[str] = []

    for path in SCOPED_FILES:
        source = path.read_text(encoding="utf-8")
        relative = path.relative_to(ROOT)

        if path != THEME:
            for match in HEX_COLOR.finditer(source):
                failures.append(
                    f"{relative}:{line_number(source, match.start())}: "
                    f"raw color {match.group(0)} must come from OgsTheme"
                )

        for token in UNSAFE_TEXT_FORMATS:
            offset = source.find(token)
            if offset != -1:
                failures.append(
                    f"{relative}:{line_number(source, offset)}: "
                    f"{token} is outside the plain-text rendering boundary"
                )

        for match in TEXT_BLOCK.finditer(source):
            block = qml_block(source, source.find("{", match.start()))
            if "textFormat: Text.PlainText" not in block:
                failures.append(
                    f"{relative}:{line_number(source, match.start())}: "
                    "Text blocks must explicitly select Text.PlainText"
                )

    theme_source = THEME.read_text(encoding="utf-8")
    for token in REQUIRED_THEME_TOKENS:
        if not re.search(rf"readonly property \w+ {re.escape(token)}\s*:", theme_source):
            failures.append(f"{THEME.relative_to(ROOT)}: missing required theme token {token}")

    if failures:
        print("QML visual policy failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(f"QML visual policy passed ({len(SCOPED_FILES)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
