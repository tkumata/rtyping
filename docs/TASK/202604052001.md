# Current Task

- Date: 2026-04-05 11:06
- Summary: 外部 API による文字列生成、暗号化設定保存、タイトルメニュー、および Config 画面を追加する。
- Steps:
  - Update requirements, ADR, design, and specifications for API generation and configuration editing.
  - Implement encrypted config persistence in `~/.config/rtyping/config.json`.
  - Add CLI provider flags `--google` and `--groq`.
  - Implement provider-aware sentence generation for local, Google AI Studio, and Groq.
  - Add title menu navigation and Config form editing in the TUI.
  - Verify behavior with tests and a successful build.
