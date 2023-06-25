# Description
I want to make something app for purpose of learning the Rust. For the time begining, I threw away readable code and tried to make to typing app. I would be happy to point out any my mistakes.

![sample](./ScreenShot.png)

# How to run

```shell
cargo run
or
cargo run -- --timeout 30 --level 20 --sound
or
cargo run -- --help
```

# How to build and install
How to install to under your `~/.cargo/bin/` directory.

```shell
cargo check
cargo build --release --locked
cargo install --path .
rtyping
or
rtyping -s -t 100
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

  - [x] Implements timeout while user input.
  - [x] Implements user can input.
  - [x] Print some words on screen.
  - [x] Using basic of random function of the Rust.
  - [x] Decorate strings which print on the screen.
  - [x] Implements timeout counter on top left.
  - [x] Fix that Backspace is behavior as "^R\\n" in `std::io` after `termion::Restore`. (I use Raw mode.)
  - [x] `temion::clear::All` ihas buggy when I use the Warp. (I did not use `clear::All`.)
  - [x] To check typo.
  - [ ] To calculate the wpm (words per minutes?).
  - [x] Play a BGM.
  - [ ] Implements adding sound effect while typing.
  - [x] Include external file, assets file etc, when building.
  - [ ] Fix that terminal tty is broken after executing `process::exit()` with raw mode.
  - [x] Implements options.
  - [x] Change behavior depending on options.
  - [ ] Implements Validating arguments of options.


# Appendix
## How to create project

```shell
cargo new rtyping
```

## How to cross-compile
### Prepairing
#### Apple silicon
```shell
rustup target add aarch64-apple-darwin
```
#### Windows
```
rustup target add x86_64-pc-windows-gnu
```

### How to build
#### Apple silicon
```shell
cargo build --release --target=aarch64-apple-darwin
```
#### Windows
```
cargo build --release --target=x86_64-pc-windows-gnu
```
