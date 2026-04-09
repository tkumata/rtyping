# Current Task

- Date: 2026-04-09 21:48
- Summary: 結果画面に正確率を追加し、総入力数と `incorrects` から安全に算出できるようにする。
- Scope:
  - Update requirements, ADR, specifications, and design for result accuracy display.
  - Add `Accuracy` to the result screen using total typed count and `incorrects`.
  - Track total typed count independently from the current input buffer so `Backspace` does not distort accuracy.
  - Define the zero-input behavior as `0.0%` to avoid division by zero.
  - Keep miss correction count and actual miss count split out of scope for this task.
- Verification:
  - Confirm the result screen shows `Typed`, `Misses`, `Accuracy`, `Time`, and `WPM`.
  - Confirm the accuracy formula matches the documented definition.
  - Confirm zero-input completion does not panic or show invalid numeric output.
