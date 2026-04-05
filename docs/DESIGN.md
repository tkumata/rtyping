# 設計 (DESIGN)

## モジュール責務

- `src/main.rs`
  - CLI 引数解析、設定読み込み、端末初期化、終了処理を担当する。
- `src/runtime/mod.rs`
  - ランタイム構成要素を束ねる。
- `src/runtime/session.rs`
  - イベントループと画面更新の進行を担当する。
- `src/runtime/input/mod.rs`
  - 状態別入力処理を束ねる。
- `src/runtime/input/menu.rs`
  - Menu 状態の入力処理を担当する。
- `src/runtime/input/config_screen.rs`
  - Config 状態の入力処理を担当する。
- `src/runtime/input/gameplay.rs`
  - Loading、Typing、Result の入力処理と生成ジョブ反映を担当する。
- `src/runtime/timer.rs`
  - タイマースレッドとタイマー補助処理を担当する。
- `src/domain/config.rs`
  - 設定モデルを保持し、UI と永続化の共有境界を担う。
- `src/presentation/ui/app.rs`
  - TUI 状態、選択中メニュー、入力中文字列、設定編集対象などの画面状態を保持する。
- `src/presentation/ui/render/mod.rs`
  - 画面描画の入口を束ねる。
- `src/presentation/ui/render/menu.rs`
  - Menu 画面を描画する。
- `src/presentation/ui/render/config_screen.rs`
  - Config 画面を描画する。
- `src/presentation/ui/render/loading.rs`
  - Loading 画面を描画する。
- `src/presentation/ui/render/typing.rs`
  - Typing 画面を描画する。
- `src/presentation/ui/render/result.rs`
  - Result 画面を描画する。
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

1. `main` が CLI 引数と設定を読み込む。
2. `main` が端末と音声、タイマースレッドを初期化する。
3. `runtime` がイベントループを実行し、`AppState` ごとの入力処理を分岐する。
4. タイトルメニューの開始系項目を選択した場合は、選択項目に対応する生成元を `App` に反映したうえで別スレッドの文字列生成を開始し、結果をチャネルで受け取る。
5. `Typing` 中はタイマースレッドの経過秒数を参照し、完了またはタイムアウトで `Result` へ遷移する。
6. 終了時は raw mode、画面、スレッド、BGM を順に停止する。

## 設定保存

- 保存先は優先パスの `~/.config/rtyping/` 配下を使う。
- `config.json` には URL、モデル、暗号化済み API key を保存する。
- `config.key` は別ファイルで管理し、起動時は優先パスと互換パスの候補を順に試す。
- 旧 AEAD ラベルと旧 XOR 形式の API key も復元対象に含める。

## テスト方針

- `src/usecase/wpm.rs`
  - 計算式の正常系と境界値を固定する。
- `src/usecase/generate_sentence.rs`
  - ローカル生成、URL 組み立て、設定不足時の失敗、正規化処理を確認する。
- `src/config/mod.rs`
  - 公開入口の設定ロード・保存を確認する。
- `src/config/storage.rs`
  - 保存往復、鍵欠損、鍵破損、旧形式互換、壊れた JSON を確認する。
- `src/runtime/input/config_screen.rs`
  - Config 入力受理ルールを固定する。
- `src/runtime/input/gameplay.rs`
  - プロバイダ設定選択、生成結果反映、ゲーム進行中の状態遷移を固定する。
- `src/runtime/session.rs`
  - イベントループと状態更新の接続点を固定する。

## 保守メモ

- UI 仕様変更を伴わない内部整理では、まず `runtime/session` と `runtime/input` の責務境界を確認する。
- 新しい生成元や設定項目を追加する場合は、`domain::config::AppConfig`、`ConfigField`、`provider_config_for_source`、描画処理の順に追うと全体を把握しやすい。
- タイトルメニュー項目を変更する場合は、`MenuItem`、`App` の選択遷移、`runtime/input/menu` の確定処理、`render/menu` の表示を同時に更新する。
