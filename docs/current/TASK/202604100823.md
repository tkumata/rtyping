# Current Task

- Date: 2026-04-09 23:00
- Summary: タイトル画面に `Practice Mode` を追加し、CLI の `-t 0` / `--timeout 0` と同じくタイムアップなしの開始経路を提供する。
- Scope:
  - Update requirements, ADR, specifications, design, help, and product overview docs for `Practice Mode`.
  - Define `Practice Mode` as a title menu item that starts `Local` generation with `timeout=0`.
  - Keep `-t 0` / `--timeout 0` as a direct no-timeout practice mode entry point.
  - Preserve the existing strict typing and `Esc` returning to `Menu` behavior in the same document set.
- Verification:
  - Confirm the title menu explicitly lists `Practice Mode`.
  - Confirm `Practice Mode` and `timeout=0` share the same no-timeout behavior.
  - Confirm the help text and product overview no longer describe `Esc` as a result-screen transition.
  - Confirm the new wording remains aligned with the existing strict typing flow.
