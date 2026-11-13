# Current Task

- Summary: Typing 画面で端末カーソルを非表示にし、現在入力すべき文字を `Yellow + Bold` で強調する。
- Docs:
  - Done: `docs/TASK/current-task.md` を `docs/TASK/202605091746.md` に退避する。
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR を現在文字強調仕様に同期する。
- Implementation:
  - Done: `src/presentation/ui/render/typing.rs` で出題文字列上の端末カーソル位置指定を削除する。
  - Done: 現在文字、未来文字、正答済み文字、誤入力位置のスタイルを仕様どおりに固定する。
  - Done: README.md に Typing 画面の表示仕様を反映する。
- Verification:
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
