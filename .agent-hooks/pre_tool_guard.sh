#!/usr/bin/env bash
set -euo pipefail

AGENT="${1:?agent required}"
EVENT="${2:?event required}"

INPUT="$(cat)"

TOOL_NAME="$(jq -r '.tool_name // .toolName // .tool // .name // empty' <<<"${INPUT}")"
COMMAND="$(jq -r '
  .tool_input.command //
  .toolInput.command //
  .input.command //
  .arguments.command //
  .command //
  empty
' <<<"${INPUT}")"

deny() {
  local reason="$1"

  case "${AGENT}" in
    codex)
      jq -nc --arg reason "${reason}" '{
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "deny",
          permissionDecisionReason: $reason
        }
      }'
      ;;

    copilot)
      jq -nc --arg reason "${reason}" '{
        decision: "deny",
        reason: $reason
      }'
      ;;

    *)
      jq -nc --arg reason "${reason}" '{
        decision: "block",
        reason: $reason
      }'
      ;;
  esac

  exit 0
}

allow() {
  case "${AGENT}" in
    codex)
      # Codex PreToolUse は allow を返しても現状 fail-open 扱いなので空で通す
      exit 0
      ;;

    copilot)
      jq -nc '{decision: "allow"}'
      exit 0
      ;;

    *)
      exit 0
      ;;
  esac
}

# Bash 以外は基本通す
case "${TOOL_NAME}" in
  Bash|bash|shell|terminal|"")
    ;;
  *)
    allow
    ;;
esac

# 破壊的操作をブロック
if grep -Eq '(^|[;&|[:space:]])rm[[:space:]]+-rf[[:space:]]+(/|\*|~|\$HOME)([[:space:];&|]|$)' <<<"${COMMAND}"; then
  deny "Blocked destructive rm -rf command."
fi

if grep -Eq '(^|[;&|[:space:]])sudo[[:space:]]+' <<<"${COMMAND}"; then
  deny "Blocked sudo command. Ask the user before privilege escalation."
fi

if grep -Eq '(^|[;&|[:space:]])git[[:space:]]+push([[:space:]]|$)' <<<"${COMMAND}"; then
  deny "Blocked git push. Ask the user before pushing."
fi

# cargo fmt を禁止
if grep -Eq '(^|[;&|[:space:]])cargo[[:space:]]+fmt([[:space:];&|]|$)' <<<"${COMMAND}" \
 && ! grep -Eq -- '--check([[:space:];&|]|$)' <<<"${COMMAND}"; then
  deny "Blocked cargo fmt (except --check)."
fi

# cargo clippy --fix を禁止
if grep -Eq '(^|[;&|[:space:]])cargo[[:space:]]+clippy([^;&|]*)--fix([[:space:];&|]|$)' <<<"${COMMAND}"; then
  deny "Blocked cargo clippy --fix. Do not auto-apply lint fixes."
fi

if grep -Eq '(^|[;&|[:space:]])git[[:space:]]+reset[[:space:]]+--hard([[:space:]]|$)' <<<"${COMMAND}"; then
  deny "Blocked git reset --hard."
fi

if grep -Eq '(^|[;&|[:space:]])git[[:space:]]+clean[[:space:]]+-[a-zA-Z]*f' <<<"${COMMAND}"; then
  deny "Blocked git clean -f."
fi

# 秘密情報ファイルの直接読み取りをブロック
if grep -Eq '(^|[;&|[:space:]])(cat|less|more|head|tail|sed|awk|grep)[[:space:]].*(\.env|id_rsa|id_ed25519|\.pem|\.p12)' <<<"${COMMAND}"; then
  deny "Blocked direct read of likely secret material."
fi

allow
