# Current Task

- Summary: Config 画面の Google AI Studio / Groq 設定が未完了の場合、タイトル画面の外部プロバイダ開始メニューを非表示にする。
- Docs:
  - Done: `docs/TASK/current-task.md` を `docs/TASK/202605092000.md` に退避する。
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、README、HELP を表示条件に同期する。
- Implementation:
  - Done: `ProviderConfig::is_ready()` を使って表示可能なメニュー項目を組み立てる。
  - Done: タイトルメニュー描画と上下キー移動を同じ表示可能リストに揃える。
  - Done: Groq のタイトルメニュー表示名を `Start Game via GroqCloud` にする。
- Verification:
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
