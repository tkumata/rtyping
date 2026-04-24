#!/usr/bin/env bash
set -euo pipefail

STATE_DIR=".codex/state"
STATE_FILE="${STATE_DIR}/pipeline_state"
LOG_DIR="${STATE_DIR}/logs"

mkdir -p "${LOG_DIR}"

if [ ! -f "${STATE_FILE}" ]; then
  echo "check_pending" > "${STATE_FILE}"
fi

PHASE="$(cat "${STATE_FILE}")"

run_and_log() {
  local name="$1"
  local cmd="$2"
  local log="${LOG_DIR}/${name}.log"

  {
    echo "\$ ${cmd}"
    echo
    bash -lc "${cmd}"
  } >"${log}" 2>&1
}

emit_continue() {
  local reason="$1"
  printf '%s\n' "$(jq -nc --arg reason "${reason}" '{decision:"block", reason:$reason}')"
}

emit_stop() {
  local msg="$1"
  printf '%s\n' "$(jq -nc --arg msg "${msg}" '{continue:false, stopReason:$msg, systemMessage:$msg}')"
}

if [ "${PHASE}" = "done" ]; then
  emit_stop "Rust validation pipeline already completed."
  exit 0
fi

if [ "${PHASE}" = "check_pending" ]; then
  if run_and_log "check" "make check"; then
    echo "build_pending" > "${STATE_FILE}"
    emit_continue "make check passed. Now run make build, inspect the result, and continue only if build succeeds."
  else
    echo "check_pending" > "${STATE_FILE}"
    emit_continue "make check failed. Fix the Rust issues and continue until check passes. Review .codex/state/logs/check.log before editing."
  fi
  exit 0
fi

if [ "${PHASE}" = "build_pending" ]; then
  if run_and_log "build" "make build"; then
    echo "done" > "${STATE_FILE}"
    emit_stop "make build passed. Task complete."
  else
    echo "build_pending" > "${STATE_FILE}"
    emit_continue "make build failed. Fix the Rust build errors and continue until build passes. Review .codex/state/logs/build.log before editing."
  fi
  exit 0
fi

emit_stop "Unknown pipeline phase: ${PHASE}"
