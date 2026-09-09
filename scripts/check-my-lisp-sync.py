#!/usr/bin/env python3
"""Не допускає розходження web і desktop pins мови my-lisp."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LOCKFILE = ROOT / "Cargo.lock"
PACKAGES = {"my-lisp", "my-lisp-literate"}
SHA_PATTERN = re.compile(r"#([0-9a-f]{40})$")


def gitlink_sha() -> str:
    """Повертає staged SHA, який Git фіксує для web submodule."""
    result = subprocess.run(
        ["git", "rev-parse", ":external/my-lisp"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def cargo_lock_shas() -> dict[str, str]:
    """Читає тільки git pins двох мовних пакетів із Cargo.lock."""
    packages: dict[str, str] = {}
    for block in LOCKFILE.read_text(encoding="utf-8").split("[[package]]"):
        name = re.search(r'^name = "([^"]+)"$', block, flags=re.MULTILINE)
        source = re.search(r'^source = "([^"]+)"$', block, flags=re.MULTILINE)
        if name is None or name.group(1) not in PACKAGES or source is None:
            continue
        sha = SHA_PATTERN.search(source.group(1))
        if sha is None:
            raise RuntimeError(f"{name.group(1)} не має git SHA у Cargo.lock")
        packages[name.group(1)] = sha.group(1)
    return packages


def main() -> int:
    web_sha = gitlink_sha()
    desktop_shas = cargo_lock_shas()
    missing = PACKAGES.difference(desktop_shas)
    if missing:
        raise RuntimeError(f"У Cargo.lock відсутні pins: {', '.join(sorted(missing))}")
    mismatched = {name: sha for name, sha in desktop_shas.items() if sha != web_sha}
    if mismatched:
        details = ", ".join(f"{name}={sha}" for name, sha in sorted(mismatched.items()))
        raise RuntimeError(
            "my-lisp інтеграційні pins розійшлися: "
            f"submodule={web_sha}; Cargo.lock: {details}"
        )
    print(f"my-lisp integration pins synchronized: {web_sha}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"my-lisp integration sync check failed: {error}", file=sys.stderr)
        raise SystemExit(1)
