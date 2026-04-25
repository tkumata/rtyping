#!/usr/bin/env bash
set -euo pipefail

AGENT="${1:?agent required}"
EVENT="${2:?event required}"

INPUT="$(cat)"

case "${EVENT}" in
  PreToolUse|preToolUse)
    .agent-hooks/pre_tool_guard.sh "${AGENT}" "${EVENT}"
    ;;

  Stop|agentStop)
    .agent-hooks/verify_pipeline.sh "${AGENT}" "${EVENT}"
    ;;

  *)
    exit 0
    ;;
esac
