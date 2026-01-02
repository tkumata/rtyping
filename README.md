# ⌨️ R-Typing - Typing Program 🦀

## 📖 Overview

R-Typing is a terminal-based typing practice application created as a learning project for Rust. The app is designed with simplicity in mind, focusing on essential features for a fun and educational experience. It features a modern TUI, real-time WPM calculation, and procedurally generated sentences.

![sample1](./docs/screenshot-01.png)
![sample2](./docs/screenshot-02.png)
![sample3](./docs/screenshot-03.png)
![sample4](./docs/screenshot-04.png)

## ⚙️ How to Run

1. Ensure you have the Rust toolchain installed.
2. Clone the repository and navigate to the project directory.
3. Run the following commands in the terminal:

```shell
# Default run
cargo run

# Run with custom options
cargo run -- --timeout 30 --level 20 --sound
```

## 🔨 Build and Install

To build and install the application in your `~/.cargo/bin/` directory:

```shell
cargo build --release
cargo install --path .
```

## 💻 Usage

```text
R-Typing: A terminal-based typing app.

Usage: rtyping [OPTIONS]

Options:
  -t, --timeout <TIMEOUT>    Timer duration in seconds [default: 60]
  -l, --level <LEVEL>        Number of words to generate [default: 30]
      --freq <FREQUENCY>     Feedback sound frequency in Hz [default: 80.0]
  -s, --sound                Enable background music (BGM)
  -h, --help                 Print help
```

## ✅ Features

- **TUI (Text User Interface)**: Rich terminal interface built with `ratatui` and `crossterm`.
- **Procedural Sentence Generation**: Uses a 4-gram Markov Chain to generate natural-feeling English sentences from a sample text.
- **Real-time Feedback**: 
  - Visual indicators for correct (green) and incorrect (red background) characters.
  - Real-time WPM (Words Per Minute) calculation.
  - Interactive countdown timer with color-coded urgency.
- **Audio Experience**:
  - Optional background music (BGM) playback using `rodio`.
  - Auditory feedback (sine wave beep) on correct keypresses.
- **Customizable Experience**: Command-line arguments to adjust time limits, sentence length, and sound settings.
- **Responsive Controls**: Supports standard typing controls including Backspace and Esc to finish early.

## 🔖 Appendix

### 🛠 Cross-Compilation Instructions

For Apple silicon (macOS):
```shell
rustup target add aarch64-apple-darwin
cargo build --release --target=aarch64-apple-darwin
```

For Windows (x86_64):
```shell
rustup target add x86_64-pc-windows-gnu
cargo build --release --target=x86_64-pc-windows-gnu
```
