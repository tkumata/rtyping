# Current Task

- Date: 2026-04-13 14:08
- Summary: `Cargo.toml` の `[lints.clippy]` に対する Even Better TOML のスキーマ診断を解消する。
- Scope:
  - Cargo が有効に解釈できる lint 設定は維持する
  - エディタ拡張の TOML スキーマ誤検出をワークスペース設定で扱う
  - アプリケーション実装と実行時挙動は変更しない
- Verification:
  - `cargo metadata --no-deps --format-version 1` で manifest が読めること
  - `Cargo.toml` の lint 強度が下がっていないこと
  - VS Code の TOML スキーマ診断対象が抑制されること
