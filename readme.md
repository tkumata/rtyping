Rust の学習目的で何か作りたかったので立ててみたみたいな。

#### 実行

    cargo run

![sample](./ScreenShot.png)

#### やる事リスト

  - [x] 入力時の制限時間の実装 (thread::spawn)
  - [x] ユーザの入力 (io::stdin)
  - [x] 出題文字列の取得 (fs::read_dir)
  - [x] 乱数の基礎 (rand::thread_rng)
  - [x] 一部文字装飾
  - [x] 制限時間カウンタ表示 (termion::cursor)
  - [ ] typo チェック
  - [ ] wpm の集計
  - [ ] 文字装飾

### とにかくメモ
ユーザ入力をトリガに typo チェックするか、裏で typo チェッカをぐるぐる走らせるか？色をつけるとなるとユーザ入力をトリガにしたほうが良いかも？

カウントした制限時間を表示専用の関数に渡す。

#### クロスコンパイル
例えば Intel Mac で Apple Silicon のバイナリを生成する場合。

    rustup target add aarch64-apple-darwin
    cargo build --target=aarch64-apple-darwin
