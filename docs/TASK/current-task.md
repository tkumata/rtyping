# Current Task

- Summary: タイトルメニューから開始できるリズムモードを追加する。
- Docs:
  - Done: 既存 `docs/TASK/current-task.md` を `docs/TASK/202605261058.md` に退避する。
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、README、HELP をリズムモード仕様に同期する。
- Implementation:
  - Done: `Start Game with Rhythm` をタイトルメニューに追加する。
  - Done: `RhythmSpeed` 設定を Game Settings に追加する。
  - Done: Local 生成後にリズムモードへ入る状態遷移を追加する。
  - Done: リズム用セッション状態、描画、入力判定、結果表示を通常 Typing と分離して実装する。
  - Done: リズムモードの直近判定を `Hit` / `OK` / `Miss` で分類し、Miss 数と Hit+OK 数をリアルタイム表示する。
  - Done: リズムモードの `^` 付近にも直近判定を表示する。
- Verification:
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
