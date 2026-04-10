# Current Task

- Date: 2026-04-10 08:58
- Summary: `typing.rs` の `clippy::items_after_test_module` 警告を解消する。
- Scope:
  - Update requirements, ADR, specifications, and design docs for the lint cleanup.
  - Reorder items in `src/presentation/ui/render/typing.rs` so the test module is the final item.
  - Keep rendering behavior unchanged while removing the warning.
- Verification:
  - Confirm `src/presentation/ui/render/typing.rs` no longer triggers `items_after_test_module`.
  - Confirm the Typing screen rendering logic remains unchanged.
