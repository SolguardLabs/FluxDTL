#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

WINDOWS_ROOT=""
if command -v wslpath >/dev/null 2>&1 \
  && command -v powershell.exe >/dev/null 2>&1 \
  && powershell.exe -NoProfile -Command "exit 0" >/dev/null 2>&1; then
  WINDOWS_ROOT="$(wslpath -w "$ROOT_DIR" 2>/dev/null || true)"
fi

run_powershell() {
  local command="$1"
  local escaped_root="${WINDOWS_ROOT//\'/\'\'}"
  powershell.exe -NoProfile -Command "Set-Location -LiteralPath '$escaped_root'; $command"
}

run_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    cargo "$@"
  elif [[ -n "$WINDOWS_ROOT" ]]; then
    run_powershell "cargo $*"
  else
    echo "cargo no esta disponible en PATH" >&2
    exit 127
  fi
}

run_node() {
  if command -v node >/dev/null 2>&1; then
    node "$@"
  elif [[ -n "$WINDOWS_ROOT" ]]; then
    run_powershell "node $*"
  else
    echo "node no esta disponible en PATH" >&2
    exit 127
  fi
}

run_cargo fmt --all -- --check
run_node ./node_modules/prettier/bin/prettier.cjs --check --config .prettierrc --ignore-path .prettierignore "**/*.{js,mjs,json,md,yml}"
run_node scripts/check-js.mjs
run_cargo build --all-targets --locked
run_cargo test --all-targets --locked
run_node scripts/run-node-tests.mjs
run_cargo clippy --all-targets --all-features --locked -- -D warnings
run_node scripts/verify-release.mjs
