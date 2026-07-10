#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

WINDOWS_ROOT=""
if command -v wslpath >/dev/null 2>&1 \
  && command -v powershell.exe >/dev/null 2>&1 \
  && powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "exit 0" >/dev/null 2>&1; then
  WINDOWS_ROOT="$(wslpath -w "$ROOT_DIR" 2>/dev/null || true)"
fi

run_powershell() {
  local command="$1"
  local escaped_root="${WINDOWS_ROOT//\'/\'\'}"
  powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "Set-Location -LiteralPath '$escaped_root'; $command"
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

run_cargo test --locked
run_node --test "tests/node/*.test.js"
