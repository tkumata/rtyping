# 設計 (DESIGN)

## モジュール責務

- `src/main.rs`
  - 設定読み込み、端末初期化、終了処理を担当する（CLI 引数解析は行わない）。
- `src/runtime/mod.rs`
  - ランタイム構成要素を束ねる。
- `src/runtime/session.rs`
  - イベントループと画面更新の進行を担当する。
- `src/runtime/input/mod.rs`
  - 状態別入力処理を束ねる。
- `src/runtime/input/menu.rs`
  - Menu 状態の入力処理を担当する。`Practice Mode` 選択時の開始条件切り替えもここで扱う。
- `src/runtime/input/config_screen.rs`
  - Config 状態の入力処理を担当する。
- `src/runtime/input/gameplay.rs`
  - Loading、Typing、Result の入力処理と生成ジョブ反映を担当する。Typing では strict 判定、`Esc` のメニュー復帰、練習モード時の遷移制御を扱う。
- `src/runtime/timer.rs`
  - タイマースレッドとタイマー補助処理を担当する。`timeout=0` の場合はタイムアウト通知を送らず、経過時間の計測のみを維持する。
- `src/domain/config.rs`
  - 設定モデルを保持し、UI と永続化の共有境界を担う。`GameSettings`（timeout / text_scale / freq / sound_enabled を文字列で管理）を含む。
- `src/presentation/ui/app.rs`
  - TUI 状態、選択中メニュー、現在入力中文字列、総入力数、WPM 履歴、設定編集対象などの画面状態を保持する。
- `src/presentation/ui/render/mod.rs`
  - 画面描画の入口を束ねる。
- `src/presentation/ui/render/menu.rs`
  - Menu 画面を描画する。`Practice Mode` を含む 5 項目のタイトルメニューを描画する。
- `src/presentation/ui/render/config_screen.rs`
  - Config 画面を描画する。Provider セクション（Google / Groq）と Game Settings セクションを表示する。
- `src/presentation/ui/render/loading.rs`
  - Loading 画面を描画する。
- `src/presentation/ui/render/typing.rs`
  - Typing 画面を描画する。出題文字列領域と WPM Sparkline 領域を分離して配置する。
  - 現在入力位置のカーソルは縦棒として描画し、四角カーソルは使わない。
  - 縦棒カーソルは文字の左側ではなく右側に置き、入力済み文字列の末尾に続く位置関係を優先する。
  - カーソル座標は、表示済みの折返し結果と一致するように算出し、改行境界でのずれを防ぐ。
  - 補助関数を先に、`#[cfg(test)] mod tests` をファイル末尾に置くことで、Clippy の構造警告を避ける。
- `src/presentation/ui/render/result.rs`
  - Result 画面を描画し、入力文字数、ミス数、正確率、経過時間、WPM と最終 WPM Sparkline を表示する。
- `src/presentation/ui/render/wpm_graph.rs`
  - Typing / Result 両画面で共通利用する WPM グラフ描画補助を担当する。
- `src/usecase/wpm.rs`
  - WPM 計算ロジックを提供する。
- `src/config/mod.rs`
  - 設定永続化の入口を提供する。
- `src/config/paths.rs`
  - 設定ファイルと鍵ファイルの探索を担当する。
- `src/config/crypto.rs`
  - API key の暗号化・復号を担当する。
- `src/config/storage.rs`
  - 設定の保存形式変換、互換復元、ファイル入出力を担当する。
- `src/usecase/generate_sentence.rs`
  - ローカル生成と外部 API 生成の統一入口を提供する。

## 実行フロー

1. `main` が設定を読み込む。
2. `main` が端末と音声、タイマースレッドを初期化する。`sound_enabled` が `true` の場合のみ BGM を開始する。
3. `runtime` がイベントループを実行し、`AppState` ごとの入力処理を分岐する。
4. タイトルメニューの開始系項目を選択した場合は、選択項目に対応する生成元と制限時間モードを `App` に反映したうえで別スレッドの文字列生成を開始し、結果をチャネルで受け取る。`Practice Mode` は `Local` 生成と `timeout=0` を組み合わせる。
5. `Typing` 中はタイマースレッドの経過秒数を参照し、WPM を再計算して履歴へ追加しながら、完了またはタイムアウトで `Result` へ遷移する。
6. `timeout=0` の場合はタイムアウト遷移を行わず、全文入力完了まで `Typing` を維持する。
7. `Typing` 中は strict 判定を行い、誤入力では入力位置を進めず、`Backspace` では現在入力中文字列だけを減らす。
8. `Typing` 中に `Esc` を押した場合は `Menu` に戻り、進行中のセッションを破棄する。
9. `Result` 描画時は総入力数と `incorrects` から正確率を算出し、未入力終了時は `0.0%` を表示する。
10. `Result` 描画時は `App` が保持している `wpm_history` をそのまま使い、タイピング終了時点のグラフを固定表示する。
11. 終了時は raw mode、画面、スレッド、BGM を順に停止する。

## 設定保存

- 保存先は優先パスの `~/.config/rtyping/` 配下を使う。
- `config.json` には URL、モデル、暗号化済み API key を保存する。
- `config.json` の `game` セクションにタイムアウト、テキスト量、周波数、サウンド設定を保存する。
- `config.key` は別ファイルで管理し、起動時は優先パスと互換パスの候補を順に試す。
- 旧 AEAD ラベルと旧 XOR 形式の API key も復元対象に含める。

## 開発環境設定

- `Cargo.toml` の `[lints.clippy]` は Cargo 標準の lint 設定として維持する。
- エディタ拡張のスキーマが Cargo の lint テーブルに追従していない場合は、`.vscode/settings.json` の TOML 検証設定で対象診断を抑制する。
- manifest 構文の確認は `cargo metadata --no-deps --format-version 1` を用いる。

## テスト方針

- `src/usecase/wpm.rs`
  - 計算式の正常系と境界値を固定する。
- `src/presentation/ui/app/typing.rs`
  - WPM 履歴がセッション開始時に初期化され、入力や経過時間更新に応じて追記されることを固定する。
- `src/presentation/ui/render/result.rs`
  - 結果画面のレイアウトが指標領域と WPM グラフ領域を両立できることを確認する。
- accuracy 計算ロジック
  - 正常系と `typed_count = 0` の境界値を固定する。
- `src/presentation/ui/app/typing.rs`
  - 総入力数が `Backspace` で減らないことを固定する。
- `src/usecase/generate_sentence.rs`
  - ローカル生成、URL 組み立て、設定不足時の失敗、正規化処理を確認する。
- `src/config/mod.rs`
  - 公開入口の設定ロード・保存を確認する。
- `src/config/storage.rs`
  - 保存往復、鍵欠損、鍵破損、旧形式互換、壊れた JSON を確認する。
- `src/runtime/input/config_screen.rs`
  - Config 入力受理ルールを固定する。
- `src/runtime/input/gameplay.rs`
  - プロバイダ設定選択、生成結果反映、ゲーム進行中の状態遷移を固定する。`Practice Mode`、`timeout=0`、strict 判定、`Esc` 復帰の挙動もここで固定する。
- `src/runtime/session.rs`
  - イベントループと状態更新の接続点を固定する。タイマー更新に伴う WPM 履歴反映もここで確認する。

## 保守メモ

- UI 仕様変更を伴わない内部整理では、まず `runtime/session` と `runtime/input` の責務境界を確認する。
- 新しい生成元や設定項目を追加する場合は、`domain::config::AppConfig`、`ConfigField`、`provider_config_for_source`、描画処理の順に追うと全体を把握しやすい。
- タイトルメニュー項目を変更する場合は、`MenuItem`、`App` の選択遷移、`runtime/input/menu` の確定処理、`render/menu` の表示を同時に更新する。
- `Practice Mode` を追加する場合は、メニュー項目の並び順と `timeout=0` の一時上書きが結果画面復帰後に通常値へ漏れないように確認する。
- 詳細なミス統計を追加する場合は `App` のカウンタ追加だけでなく、`Backspace` を含む入力イベント定義と結果画面文言を同時に見直す。
- タイムアップなしの練習モードを追加する場合は、`timeout=0` の扱いとタイマー停止条件を先に固定する。
- Typing / Result 画面のレイアウト変更時は、主要テキストの可読性、Sparkline 領域との非重複、狭い端末での退避挙動を同時に確認する。
- Typing 画面のカーソル形状を変更する場合は、出題文字列の可読性を損なわないよう縦棒カーソルの幅と位置を先に決める。右側配置に寄せると、入力位置が文字列の読み順に沿って見える。
- Typing 画面のカーソル位置を調整する場合は、文字数からの単純換算ではなく、表示幅と折返し結果に合わせて座標を求める。改行ごとのずれはこの層で吸収する。
- `GameSettings` のフィールドは `String` 型で保持する。これにより UI の入力処理が統一され、数値バリデーションは保存時または使用時に行う。
- Rust モジュールでテストを追加する場合は、通常項目の後に `#[cfg(test)] mod tests` を置く配置を維持する。
