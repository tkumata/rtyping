# 設計 (DESIGN)

## テスト構成

Rust の標準的な慣習に従い、各ソースファイル内に `#[cfg(test)]` 属性を持つテストモジュール `tests` を配置する。

## 自動実行構成

GitHub Actions を用いて、テストコードを継続的に実行する。

### ワークフロー構成

- `.github/workflows/test.yml`
  - `push` と `pull_request` を契機に起動する。
  - `ubuntu-latest` と `macos-latest` の 2 環境で `cargo test` を実行する。
- `.github/workflows/version-check.yml`
  - `Cargo.toml` のバージョン更新を検知した場合のみ起動する。
  - リリースビルドの前段で `cargo test` を実行し、失敗時はリリース処理を停止する。

### 依存ライブラリ

- Linux 上では `rodio` 依存のビルドのために `pkg-config`, `libasound2-dev`, `libudev-dev` を導入する。

### ディレクトリ構成とテスト配置

- `src/usecase/wpm.rs` -> 同ファイル内 `mod tests`
- `src/usecase/generate_sentence.rs` -> 同ファイル内 `mod tests`
- `src/domain/entity.rs` -> 同ファイル内 `mod tests`

## テスト戦略

### 1. WPM 計算 (`wpm.rs`)

- 純粋関数であるため、入力値に対する期待値を検証する。
- **正常系**: 通常の入力での計算結果。
- **境界値**: 入力文字数0、経過時間0、ミス数0など。
  - *注意*: 経過時間 0 の場合のゼロ除算挙動を確認する（Rust では `f64` のゼロ除算は `inf` になる）。

### 2. 文生成 (`generate_sentence.rs`)

- 内部で乱数を使用しているため、出力結果の完全一致はテストしない。
- 以下の特性を検証する：
  - エラーにならず `Ok` が返ること。
  - 返された文字列が空でないこと。
  - 指定したレベル（長さ）に応じた出力が得られるか（目安）。

### 3. エンティティ (`entity.rs`)

- `include_str!` を使用しているため、データは静的。
- 読み込んだデータが空でないことを確認する。

## 将来的な拡張

- 現状、乱数生成器が注入できない設計になっているため、厳密なロジックテスト（特定のシードでの出力固定など）は行わない。将来的に RNG を DI するリファクタリングを行う際にテストを強化する。
- 将来的に `cargo fmt --check` や `cargo clippy` を同じワークフローへ段階的に追加できる。

## 2026-04-05 CLI デフォルト値見直し設計

### 変更対象

- `src/presentation/ui/ui_handler.rs`
  - `clap` の既定値とヘルプ文言を現行仕様に合わせる。
- `src/main.rs`
  - 正答時のタイプ音再生を `args.sound` 有効時のみに制限する。
- `docs/HELP.md`
  - `--sound` の説明を BGM とタイプ音の両方を有効化する意味に更新する。

### 設計方針

- 音の有効化フラグは増やさず、既存の `--sound` を音関連全体のマスター切り替えとして扱う。
- BGM の起動条件とタイプ音の再生条件を同一フラグに揃え、引数未指定時の挙動を静音に統一する。
- `--timeout` の既定値は実装上すでに 60 秒のため、今回の変更では仕様と文言の整合確認を含める。

## 2026-04-05 API 文字列生成と設定画面の設計

### 変更対象

- `src/presentation/ui/ui_handler.rs`
  - `--google` と `--groq` の排他的フラグを追加する。
- `src/presentation/ui/app.rs`
  - `Menu`、`Config`、`Loading` を含む状態管理、メニュー選択、Config 入力欄、メッセージ表示を追加する。
- `src/presentation/ui/render.rs`
  - タイトルメニュー、Config 画面、生成中表示、エラーメッセージ表示を描画する。
- `src/main.rs`
  - メニュー遷移、Config 保存、API/ローカル生成開始、非同期生成結果の反映を制御する。
- `src/config.rs`
  - 設定の読み書き、暗号化・復号、保存パス管理を実装する。
- `src/usecase/generate_sentence.rs`
  - ローカル生成と外部 API 生成を統合する生成ユースケースへ拡張する。

### 設定設計

- 設定ファイルは `~/.config/rtyping/config.json` とする。
- JSON には `google` と `groq` の各設定を保持し、各設定は `api_url`、`model`、`api_key_ciphertext`、`api_key_nonce` を持つ。
- 復号鍵は `~/.config/rtyping/config.key` に分離して保持し、存在しない場合は初回保存時に生成する。
- Config 画面は平文 API key を編集できるが、保存時のみ暗号化し、画面再表示時は復号して入力欄へ戻す。

### 文字列生成設計

- 生成元は `Local`、`Google`、`Groq` の 3 種類とする。
- CLI で選択したプロバイダを起動時の生成元として `App` に保持する。
- 外部 API 利用時は `--level * 5` を目標文字数とし、プロンプトで「指定文字数前後の英数字中心テキスト」を要求する。
- API 応答は改行や過剰な空白を整形し、目標文字数を大きく超える場合は切り詰める。
- API 設定不足や通信失敗時は Result ではなく Menu へ戻し、エラーメッセージを表示する。

### UI 状態遷移

- `Menu`
  - `Start Game` 選択時に文字列生成を開始する。
  - `Config` 選択時に Config 画面へ遷移する。
- `Config`
  - `↑↓` で入力欄移動、文字入力と `Backspace` で編集、`Enter` で保存、`Esc` で Menu に戻る。
- `Loading`
  - API 生成中はローディング画面を表示し、別スレッドの結果を待つ。
- `Typing`
  - 既存どおりタイピングを行う。
- `Result`
  - 既存どおり結果を表示する。
