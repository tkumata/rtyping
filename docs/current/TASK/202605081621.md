# Current Task

- Summary: Timed セッションの結果履歴を保存し、Result と Stats 画面で統計を表示する。
- Docs:
  - REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、PLAN を履歴管理仕様に同期する。
- Implementation:
  - `~/.config/rtyping/history.json` の読み書きを追加する。
  - Timed セッション完了時だけ履歴を保存する。
  - 正解側ミス文字を記録し、頻出ミス文字として集計する。
  - Result 画面と Stats 画面に自己ベスト、平均、直近10回、頻出ミス文字を表示する。
  - Menu に `Stats` を追加する。
- Verification:
  - Timed と Practice Mode の保存対象差をテストする。
  - 統計集計と Menu / Stats 遷移をテストする。
  - `make check` と `make build` を実行する。
