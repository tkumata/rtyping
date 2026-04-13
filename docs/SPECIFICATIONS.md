# 仕様 (SPECIFICATIONS)

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
- `CFG-008`
  - Config 画面では `Timeout`（秒）、`TextScale`（文字数スケール）、`Freq`（Hz）、`SoundEnabled`（true/false）を編集できる。
- `CFG-009`
  - `SoundEnabled` フィールドではスペースキーでトグル操作を行う。
- `CFG-010`
  - ゲーム設定は `String` 型として保存し、`config.json` の `game` セクションに書き込む（旧ファイルは `#[serde(default)]` で読み込み可能）。
- `CFG-011`
  - BGM は起動時の設定を使用し、再起動後に反映される。キープレス音は設定変更後即時反映される。

## Runtime

- `RUN-001`
  - タイトル画面では `Start Game`、`Practice Mode`、`Start Game via Google AI Studio`、`Start Game via Groq`、`Config` を上下キーで巡回選択できる。
- `RUN-002`
  - `Start Game` 選択時は `Local` 生成で `Loading` に遷移し、生成完了で `Typing` に進む。
- `RUN-003`
  - `Practice Mode` 選択時は `Local` 生成かつタイムアウト無効で `Loading` に遷移し、生成完了で `Typing` に進む。
- `RUN-004`
  - `Start Game via Google AI Studio` 選択時は `Google AI Studio` 生成で `Loading` に遷移し、生成完了で `Typing` に進む。
- `RUN-005`
  - `Start Game via Groq` 選択時は `Groq` 生成で `Loading` に遷移し、生成完了で `Typing` に進む。
- `RUN-006`
  - 生成失敗時は `Menu` に戻り、失敗理由を表示する。
- `RUN-007`
  - `Loading` 中に `Esc` を押すと生成要求を破棄して `Menu` に戻る。
- `RUN-008`
  - `Config` 画面で `Enter` は保存、`Esc` は破棄、`Backspace` は文字削除として扱う。
- `RUN-009`
  - `Typing` 中に全文入力またはタイムアウトで `Result` に遷移する。
- `RUN-010`
  - Config で `timeout=0` に設定して開始した `Typing` は、全文入力時のみ `Result` に遷移し、タイムアウトでは終了しない。
- `RUN-011`
  - `Practice Mode` で開始した `Typing` は、全文入力時のみ `Result` に遷移する。
- `RUN-012`
  - `Typing` は strict 判定を行い、入力文字がターゲット文字と一致した場合のみ入力位置を進める。
- `RUN-013`
  - `Typing` 中に `Esc` を押すと `Menu` に戻る。
- `RUN-014`
  - `Result` 画面は `Typed`、`Misses`、`Accuracy`、`Time`、`WPM` を表示する。
- `RUN-015`
  - `Typed` は `Backspace` で減らない総入力数 `total_typed_count` を表示する。
- `RUN-016`
  - `Accuracy` は `accuracy = (total_typed_count - incorrects) / total_typed_count * 100` で算出し、小数 1 桁で表示する。
- `RUN-017`
  - `total_typed_count = 0` のまま `Result` に遷移した場合、`Accuracy` は `0.0%` と表示する。
- `RUN-018`
  - `Misses` は既存の `incorrects` をそのまま表示し、ミス修正回数や実ミス数の分離は今回の仕様に含めない。
- `RUN-019`
  - `Typing` 画面は現在 WPM の数値表示に加えて、`ratatui::widgets::Sparkline` による WPM 推移グラフを表示する。
- `RUN-020`
  - `Result` 画面は `Typed`、`Misses`、`Accuracy`、`Time`、`WPM` に加えて、セッション終了時点の WPM 推移グラフを表示する。
- `RUN-021`
  - WPM 推移グラフは各画面の本文領域と別座標に配置し、本文テキストと重ならない。
- `RUN-022`
  - WPM 推移サンプルはタイピング開始時に空で初期化し、タイピング中の再描画に合わせて更新する。
- `RUN-023`
  - `Result` 画面の WPM グラフは、Typing 中に蓄積した `wpm_history` の最終内容をそのまま表示し、結果画面専用の別履歴は持たない。
- `RUN-024`
  - 画面サイズが不足する場合でも、各画面の主要情報の可読性を優先し、グラフは縮小または簡略表示しても重なりを起こさない。
- `RUN-025`
  - `src/presentation/ui/render/typing.rs` と `src/presentation/ui/render/result.rs` を含む描画モジュールでは、`#[cfg(test)] mod tests` を通常関数より後ろ、すなわちファイル末尾へ配置し、`clippy::items_after_test_module` を発生させない。

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
  - accuracy 計算追加後の指標ロジック
  - `timeout=0` の練習モード判定
  - タイトル画面の `Practice Mode` 選択時の開始条件
  - strict 入力判定と `Esc` のメニュー復帰
  - WPM 履歴の初期化と更新
  - `Typing` 画面で WPM グラフ領域と出題文字列領域が分離されること
  - `Result` 画面で最終 WPM グラフが表示されること
  - `src/presentation/ui/render/typing.rs` で `cargo clippy` の `items_after_test_module` 警告が発生しないこと
