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
  - [ ] termion::Restore 後 io::stdin で Backspace が  "^R\\n" 扱いになってしまうを修正する。
  - [x] Warp で temion::clear::All の挙動がおかしいので対応する。

### とにかくメモ
ユーザ入力をトリガに typo チェックするか、裏で typo チェッカをぐるぐる走らせるか？色をつけるとなるとユーザ入力をトリガにしたほうが良いかも？

カウントした制限時間を表示専用の関数に渡す。

print!() は改行コードなしなので flush() しないといけなくて、そのせいで \\r が入ってその後 flush() して　\\r が実行されると。で入力としての \\r が文字で残る。termion のせいというより実装方法の問題かもしれない。→ bingo!

cursor_pos() はクロージャーの中じゃないといけない。

stdin 中はカーソルを移動させないようにしたい → stdin じゃなくて別の入力ロジックを実装する必要がある。


#### クロスコンパイル
例えば Intel Mac で Apple Silicon のバイナリを生成する場合。

    rustup target add aarch64-apple-darwin
    cargo build --target=aarch64-apple-darwin
