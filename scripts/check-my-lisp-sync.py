#!/usr/bin/env python3
"""Не допускає повернення другого (git) каналу поряд із pinned submodule.

SUBMODULE-DEPENDENCY-MODEL-2026-09-16 (ecosystem docs/): для одного
upstream (my-lisp) у цьому репо має бути рівно один pin-механізм —
`external/my-lisp` git submodule, споживаний через Cargo `path`-залежність
(`src-tauri/Cargo.toml`). Раніше тут порівнювались два незалежні pins
(submodule SHA vs. floating `git branch = "main"` у Cargo.lock); тепер
другого каналу просто не повинно існувати, тож ця перевірка ловить його
повернення, а не розходження двох чисел.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = ROOT / "src-tauri" / "Cargo.toml"
LOCKFILE = ROOT / "Cargo.lock"
PACKAGES = {"my-lisp", "my-lisp-literate"}
SUBMODULE_PATH = ROOT / "external" / "my-lisp"


def gitlink_sha() -> str:
    """Повертає staged SHA підмодуля `external/my-lisp` — сам факт, що
    команда не падає, вже підтверджує, що це зареєстрований gitlink, а не
    осиротілий (порожній `.gitmodules`-запис зловив би це раніше)."""
    result = subprocess.run(
        ["git", "rev-parse", ":external/my-lisp"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def cargo_toml_uses_path_dependency() -> set[str]:
    """Повертає імена пакетів, які Cargo.toml оголошує через
    `path = "../external/my-lisp/..."` (єдиний дозволений канал)."""
    text = CARGO_TOML.read_text(encoding="utf-8")
    found: set[str] = set()
    for name in PACKAGES:
        pattern = re.compile(
            rf'^{re.escape(name)}\s*=\s*\{{[^}}]*path\s*=\s*"\.\./external/my-lisp/[^"]*"',
            re.MULTILINE,
        )
        if pattern.search(text):
            found.add(name)
    return found


def cargo_lock_git_sources() -> dict[str, str]:
    """Повертає {ім'я: git-source}, якщо в Cargo.lock лишився ДРУГИЙ,
    незалежний git-канал для цих пакетів — саме це заборонено моделлю."""
    offenders: dict[str, str] = {}
    for block in LOCKFILE.read_text(encoding="utf-8").split("[[package]]"):
        name = re.search(r'^name = "([^"]+)"$', block, flags=re.MULTILINE)
        source = re.search(r'^source = "([^"]+)"$', block, flags=re.MULTILINE)
        if name is None or name.group(1) not in PACKAGES or source is None:
            continue
        if source.group(1).startswith("git+"):
            offenders[name.group(1)] = source.group(1)
    return offenders


def main() -> int:
    if not SUBMODULE_PATH.exists():
        raise RuntimeError(
            f"external/my-lisp submodule not checked out at {SUBMODULE_PATH} "
            "— run `git submodule update --init`"
        )
    sha = gitlink_sha()

    declared = cargo_toml_uses_path_dependency()
    missing = PACKAGES - declared
    if missing:
        raise RuntimeError(
            "Cargo.toml має оголошувати ці пакети через "
            f"path = \"../external/my-lisp/...\": {', '.join(sorted(missing))}"
        )

    git_sources = cargo_lock_git_sources()
    if git_sources:
        details = ", ".join(f"{name}={source}" for name, source in sorted(git_sources.items()))
        raise RuntimeError(
            "Знайдено другий (git) канал поряд із pinned submodule — "
            f"одна залежність має один канал істини: {details}"
        )

    print(f"my-lisp: single channel confirmed (external/my-lisp @ {sha})")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"my-lisp single-channel check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
