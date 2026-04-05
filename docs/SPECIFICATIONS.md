# 仕様 (SPECIFICATIONS)

## CLI

- `CLI-001`
  - 引数未指定時は `Local` 生成を使う。
- `CLI-002`
  - `--google` 指定時は Google AI Studio を使う。
- `CLI-003`
  - `--groq` 指定時は Groq を使う。
- `CLI-004`
  - `--google` と `--groq` の同時指定は `clap` が拒否する。
- `CLI-005`
  - 引数未指定時の制限時間は 60 秒である。
- `CLI-006`
  - `--sound` 指定時のみ BGM と正答音を有効化する。

## Config

- `CFG-001`
  - 保存時に `config.json` へ URL、モデル、暗号化済み API key を書き込む。
- `CFG-002`
  - API key の平文は `config.json` に残さない。
- `CFG-003`
  - 次回起動時に URL、モデル、API key を復元できる。
- `CFG-004`
  - `config.key` 欠損時は URL とモデルだけを復元し、API key は空欄と警告にする。
- `CFG-005`
  - `config.key` 破損時もアプリは継続し、非秘密情報だけを復元する。
- `CFG-006`
  - 旧 AEAD ラベルと旧 XOR 形式の保存データを互換復元する。
- `CFG-007`
  - 壊れた `config.json` は parse error として扱う。

## Runtime

- `RUN-001`
  - タイトル画面では `Start Game` と `Config` を上下キーで切り替える。
- `RUN-002`
  - `Start Game` 選択時は `Loading` に遷移し、生成完了で `Typing` に進む。
- `RUN-003`
  - 生成失敗時は `Menu` に戻り、失敗理由を表示する。
- `RUN-004`
  - `Loading` 中に `Esc` を押すと生成要求を破棄して `Menu` に戻る。
- `RUN-005`
  - `Config` 画面で `Enter` は保存、`Esc` は破棄、`Backspace` は文字削除として扱う。
- `RUN-006`
  - `Typing` 中に全文入力またはタイムアウトで `Result` に遷移する。

## Sentence Generation

- `GEN-001`
  - `Local` 生成はサンプルテキストから空でない文字列を返す。
- `GEN-002`
  - Google の URL は `API URL` と `Model` から `:generateContent` 付きで組み立てる。
- `GEN-003`
  - Google と Groq は設定不足時に `io::Error` を返す。
- `GEN-004`
  - 応答文字列は ASCII ベースへ正規化し、目標文字数で切り詰める。

## Test Entry Points

- `cargo test`
  - `config`
  - `runtime`
  - `usecase::generate_sentence`
  - `usecase::wpm`
  - `domain::entity`
