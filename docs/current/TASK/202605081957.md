# Current Task

- Summary: 起動直後のタイトル画面で、選択を表す記号を `>` から `▶︎` へ変更する。
- Docs:
  - PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR をタイトルメニュー選択マーカー変更に同期する。
- Implementation:
  - Done: `src/presentation/ui/render/menu.rs` の `menu_line` で、選択中項目のマーカーを `▶︎` に変更する。
  - Done: 非選択項目、メニュー項目、選択順、キー操作、状態遷移は変更しない。
  - Done: README.md の Title Menu に選択マーカー仕様を追記する。
- Verification:
  - Done: `src/presentation/ui/render/menu.rs` で、初期選択項目 `Start Game` のマーカーが `▶︎` になることを確認する。
  - Done: `make check` と `make build` を実行する。
