# ADR

## 2026-07-10 Stop hook の fingerprint ベース検証

### 決定

Stop hook は Rust 関連の作業ツリー変更の内容 fingerprint を基準に検証を判断する。対象がなければ検証を行わず、未検証の fingerprint があれば `make check`、続く同一 fingerprint に `make build` を実行する。

### 理由

従来の固定 `done` state は後続の Rust 変更を見逃し、初回 Stop は docs だけの変更でも検証を開始していた。また check と build の間の変更も検出していなかった。

### 影響

- Rust と Cargo のビルド入力に関係する変更だけがローカル品質ゲートを起動する。
- untracked Rust ファイルも fingerprint に含める。
- dispatcher を削除し、PreToolUse の JSON を guard script へ直接渡す。
