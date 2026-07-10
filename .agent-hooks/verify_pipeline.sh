#!/usr/bin/env bash
set -euo pipefail

AGENT="${1:-codex}"

STATE_DIR=".agent-hooks/state"
STATE_FILE="${STATE_DIR}/pipeline_state"
LOG_DIR="${STATE_DIR}/logs"
RUST_PATHS=(
  ':(glob)*.rs'
  ':(glob)**/*.rs'
  Cargo.toml
  Cargo.lock
  build.rs
  rust-toolchain
  rust-toolchain.toml
  ':(glob).cargo/**'
)

mkdir -p "${LOG_DIR}"

PHASE="idle"
CHECK_FINGERPRINT=""
VALIDATED_FINGERPRINT=""

load_state() {
  [ -f "${STATE_FILE}" ] || return

  while IFS='=' read -r key value; do
    case "${key}" in
      phase) PHASE="${value}" ;;
      check_fingerprint) CHECK_FINGERPRINT="${value}" ;;
      validated_fingerprint) VALIDATED_FINGERPRINT="${value}" ;;
    esac
  done < "${STATE_FILE}"
}

save_state() {
  {
    printf 'phase=%s\n' "${PHASE}"
    printf 'check_fingerprint=%s\n' "${CHECK_FINGERPRINT}"
    printf 'validated_fingerprint=%s\n' "${VALIDATED_FINGERPRINT}"
  } > "${STATE_FILE}"
}

has_rust_changes() {
  ! git diff --quiet HEAD -- "${RUST_PATHS[@]}" \
    || [ -n "$(git ls-files --others --exclude-standard -- "${RUST_PATHS[@]}")" ]
}

rust_fingerprint() {
  {
    git diff --binary HEAD -- "${RUST_PATHS[@]}"

    while IFS= read -r -d '' path; do
      printf 'untracked:%s\0' "${path}"
      git hash-object -- "${path}"
    done < <(git ls-files -z --others --exclude-standard -- "${RUST_PATHS[@]}" | sort -z)
  } | shasum -a 256 | awk '{print $1}'
}

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

  case "${AGENT}" in
    codex)
      jq -nc --arg reason "${reason}" \
        '{decision:"block", reason:$reason}'
      ;;
    copilot)
      jq -nc --arg reason "${reason}" \
        '{continue:false, message:$reason}'
      ;;
    *)
      jq -nc --arg reason "${reason}" \
        '{decision:"block", reason:$reason}'
      ;;
  esac
}

emit_stop() {
  local msg="$1"

  case "${AGENT}" in
    codex)
      jq -nc --arg msg "${msg}" \
        '{continue:false, stopReason:$msg, systemMessage:$msg}'
      ;;
    copilot)
      jq -nc --arg msg "${msg}" \
        '{continue:true, message:$msg}'
      ;;
    *)
      jq -nc --arg msg "${msg}" \
        '{continue:false, stopReason:$msg, systemMessage:$msg}'
      ;;
  esac
}

load_state

if ! has_rust_changes; then
  PHASE="idle"
  CHECK_FINGERPRINT=""
  save_state
  emit_stop "No Rust-related changes require validation."
  exit 0
fi

FINGERPRINT="$(rust_fingerprint)"

if [ "${FINGERPRINT}" = "${VALIDATED_FINGERPRINT}" ]; then
  PHASE="done"
  CHECK_FINGERPRINT=""
  save_state
  emit_stop "Validation pipeline already completed for the current Rust changes."
  exit 0
fi

if [ "${PHASE}" = "build_pending" ] && [ "${FINGERPRINT}" != "${CHECK_FINGERPRINT}" ]; then
  PHASE="check_pending"
  CHECK_FINGERPRINT=""
fi

if [ "${PHASE}" != "build_pending" ]; then
  if run_and_log "check" "make check"; then
    PHASE="build_pending"
    CHECK_FINGERPRINT="${FINGERPRINT}"
    save_state
    emit_continue "make check passed. Next run make build if Rust-related changes remain unchanged."
  else
    PHASE="check_pending"
    CHECK_FINGERPRINT=""
    save_state
    emit_continue "make check failed. Fix the root cause and review .agent-hooks/state/logs/check.log."
  fi
  exit 0
fi

if run_and_log "build" "make build"; then
  if [ "$(rust_fingerprint)" = "${CHECK_FINGERPRINT}" ]; then
    PHASE="done"
    VALIDATED_FINGERPRINT="${CHECK_FINGERPRINT}"
    CHECK_FINGERPRINT=""
    save_state
    emit_stop "make build passed. Task complete."
  else
    PHASE="check_pending"
    CHECK_FINGERPRINT=""
    save_state
    emit_continue "Rust-related changes changed during build. Run make check again."
  fi
else
  PHASE="build_pending"
  save_state
  emit_continue "make build failed. Fix the root cause and review .agent-hooks/state/logs/build.log."
fi
