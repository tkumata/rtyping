# 概要
Rust の学習目的で何か作りたかったので立ててみたみたいな。

![sample](./ScreenShot.png)

## 実行

```shell
cargo new typing_game
cargo run
cargo check
cargo build --release --locked
cargo install --path .
```

## やる事リスト

- [x] 入力時の制限時間の実装
- [x] ユーザの入力
- [x] 出題文字列の取得
- [x] 乱数の基礎
- [x] 一部文字装飾
- [x] 制限時間カウンタ表示
- [x] termion::Restore 後 io::stdin で Backspace が "^R\\n" 扱いになってしまうを修正する。
- [x] Warp で `temion::clear::All` 前後の挙動がおかしいので対応する。
- [ ] typo チェック
- [ ] wpm の集計
- [ ] 文字装飾


# 付録
## クロスコンパイル
例えば Intel Mac で Apple Silicon Mac のバイナリを生成する場合。

### 準備
```shell
rustup target add aarch64-apple-darwin
```

### ビルド
```shell
cargo build --release --target=aarch64-apple-darwin
```
