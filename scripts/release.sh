#!/usr/bin/env bash
# Release helper for my-idea · Помічник релізу · Release-Helfer.
# Usage / Використання: ./scripts/release.sh 0.3.0

set -euo pipefail

VERSION="${1:-}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Version must use MAJOR.MINOR.PATCH (example: 0.3.0)." >&2
  exit 1
fi

if [[ ! -d .git || ! -f package.json || ! -f src-tauri/Cargo.toml ]]; then
  echo "Run this script from the repository root." >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Commit or stash existing changes before creating a release." >&2
  exit 1
fi

echo "Preparing my-idea v$VERSION"

# Node updates JSON without fragile quote-sensitive sed expressions.
node -e '
  const fs = require("node:fs");
  const version = process.argv[1];
  for (const path of ["package.json", "src-tauri/tauri.conf.json"]) {
    const data = JSON.parse(fs.readFileSync(path, "utf8"));
    data.version = version;
    fs.writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`);
  }
  const lockPath = "package-lock.json";
  const lock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
  lock.version = version;
  lock.packages[""].version = version;
  fs.writeFileSync(lockPath, `${JSON.stringify(lock, null, 2)}\n`);
' "$VERSION"

# Only the first version assignment is the application package version.
sed -E -i "0,/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/s//version = \"$VERSION\"/" src-tauri/Cargo.toml

cargo check --manifest-path src-tauri/Cargo.toml
npm test
npm run check
npm run build

git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "release: v$VERSION"
git tag -a "v$VERSION" -m "my-idea v$VERSION | Версія v$VERSION | Veröffentlichung v$VERSION"

# Atomic push prevents a branch-only or tag-only partial release.
git push --atomic origin main "v$VERSION"
echo "Release v$VERSION pushed successfully."
