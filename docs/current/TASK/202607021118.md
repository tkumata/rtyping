# Current Task

- Summary: リズムモードに連続成功コンボ表示を追加する。
- Docs:
  - Done: 既存 `docs/TASK/current-task.md` を `docs/TASK/202605270819.md` に退避する。
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、README、HELP をコンボ仕様に同期する。
- Implementation:
  - Done: `Hit` と `OK` の連続成功数をリズムセッションに保持する。
  - Done: `Miss` でコンボ数を 0 に戻す。
  - Done: 2 以上のコンボを `^` 付近に `{count} Combo!!` 形式で表示する。
- Verification:
  - Done: コンボ加算、非表示閾値、Miss リセットのテストを追加する。
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
