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
