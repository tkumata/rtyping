# Description
I want to make something app for purpose of learning the Rust. For the time begining, I threw away readable code and tried to make to typing app. I would be happy to point out any my mistakes.

![sample](./ScreenShot.png)

# How to build

```shell
cargo run
or
cargo run -- --timeout 30 --level 20 --sound
or
cargo run -- --help
```

# How to install
How to install to under your `~/.cargo/bin/` directory.

```shell
cargo check
cargo build --release --locked
cargo install --path .
rtyping
```

# Usage

```shell
This is typing practice app on terminal.

Usage: rtyping [OPTIONS]

Options:
  -t, --timeout <TIMEOUT>  Seconds [default: 60]
  -l, --level <LEVEL>      Number of words [default: 4]
  -s, --sound              Turn BGM on
  -h, --help               Print help
```


# Todo list

  - [x] 入力時の制限時間の実装
  - [x] ユーザの入力
  - [x] 出題文字列の取得
  - [x] 乱数の基礎
  - [x] 一部文字装飾
  - [x] 制限時間カウンタ表示 (入力待ちしつつ別の場所に時間のカウンターを置くことが一番苦労した)
  - [x] termion::Restore 後 io::stdin で Backspace が "^R\\n" 扱いになってしまうを修正する。(Canonicl mode をやめて Raw mode に移行することで解決)
  - [x] Warp で `temion::clear::All` 前後の挙動がおかしいので対応する。(clear::All じゃなくて前後を clear する事で解決)
  - [x] typo チェック
  - [ ] wpm の集計
  - [x] 文字装飾
  - [x] BGM 追加
  - [ ] SE 追加
  - [x] build 時に外部ファイルもバイナリに含める
  - [ ] Fix that terminal tty is broken after executing `process::exit()` on raw mode.
  - [x] Implements options.
  - [x] Change behavior depending on options.
  - [ ] Validation options arguments.


# Appendix
## How to create project

```shell
cargo new rtyping
```

## How to cross-compile
For example, build to Apple silicon on Intel Mac.

### Prepairing

```shell
rustup target add aarch64-apple-darwin
```

### How to build

```shell
cargo build --release --target=aarch64-apple-darwin
```
