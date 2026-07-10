# PLAN

## 目的

- Stop hook が Rust 関連の変更にだけ `make check` と `make build` を実行するようにする。
- 同じ変更に対する重複実行を避けつつ、check 後に Rust 関連ファイルが変わった場合は check からやり直す。
- PreToolUse の標準入力を失わせる dispatcher を取り除き、設定から各 hook を直接呼び出す。

## 作業項目

1. Rust 関連ファイルの変更検出と内容 fingerprint を `verify_pipeline.sh` に実装する。
2. pipeline state を fingerprint を持つ key-value 形式へ移行する。
3. `make check` と `make build` の間で fingerprint が変わった場合に check へ戻す。
4. Codex と Copilot の設定を各 hook script の直接呼び出しへ変更し、dispatcher を削除する。
5. 退役済みの GitHub Actions workflow ファイルを削除する。
6. shell / JSON 構文、hook の状態遷移、`make check`、`make build` を確認する。

## 確認観点

- Rust 関連の変更がなければ Stop hook は検証コマンドを実行しないこと。
- Rust 関連の変更があれば check、続いて同一 fingerprint の build を実行すること。
- check 後に対象差分が変われば build を実行せず check へ戻ること。
- PreToolUse が元の JSON 標準入力を guard script へ渡せること。
