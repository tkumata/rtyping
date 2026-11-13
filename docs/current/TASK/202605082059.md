# Current Task

- Summary: Config 画面の文字列入力欄で、左右キーによるカーソル移動とカーソル位置への文字入力を可能にする。
- Docs:
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR を Config 入力カーソル編集仕様に同期する。
- Implementation:
  - Done: `App` に Config 入力カーソル位置を追加する。
  - Done: `src/presentation/ui/app/config_editor.rs` で左右移動、挿入、カーソル直前削除を実装する。
  - Done: `src/runtime/input/config_screen.rs` で `Left` / `Right` を Config 入力カーソル移動に割り当てる。
  - Done: `src/presentation/ui/render/config_screen.rs` でカーソル座標と API key 同長マスクを反映する。
  - Done: README.md に Config 画面のカーソル編集操作を反映する。
- Verification:
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
