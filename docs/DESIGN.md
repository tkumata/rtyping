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
  - Menu 状態の入力処理を担当する。`Practice Mode` 選択時の開始条件切り替えと `Stats` への遷移もここで扱う。
- `src/runtime/input/config_screen.rs`
  - Config 状態の入力処理を担当する。入力欄の上下移動、左右カーソル移動、カーソル位置への文字挿入、保存、破棄を扱う。
- `src/runtime/input/gameplay.rs`
  - Loading、Typing、Result の入力処理と生成ジョブ反映を担当する。Typing では strict 判定、`Esc` のメニュー復帰、練習モード時の遷移制御を扱う。
- `src/runtime/timer.rs`
  - タイマースレッドとタイマー補助処理を担当する。`timeout=0` の場合はタイムアウト通知を送らず、経過時間の計測のみを維持する。
- `src/domain/config.rs`
  - 設定モデルを保持し、UI と永続化の共有境界を担う。`GameSettings`（timeout / text_scale / freq / sound_enabled を文字列で管理）を含む。
- `src/domain/history.rs`
  - 成績履歴の保存単位とモード種別を保持する。
- `src/presentation/ui/app.rs`
  - TUI 状態、選択中メニュー、現在入力中文字列、総入力数、ミス文字、WPM 履歴、設定編集対象、Config 入力カーソル位置、履歴統計などの画面状態を保持する。
  - WPM 履歴には、入力操作の有無を判定するための進行状態も持たせる。
- `src/presentation/ui/app/menu.rs`
  - 現在の設定からタイトル画面で表示可能なメニュー項目を組み立てる。Google AI Studio と GroqCloud の開始項目は、各プロバイダの `API URL`、`API Key`、`Model` がすべて空でない場合だけ含める。
- `src/presentation/ui/render/mod.rs`
  - 画面描画の入口を束ねる。
- `src/presentation/ui/render/menu.rs`
  - Menu 画面を描画する。`Practice Mode` と `Stats` を含むタイトルメニューを描画し、外部プロバイダの開始項目は `App` が返す表示可能リストに従う。
- `src/presentation/ui/render/config_screen.rs`
  - Config 画面を描画する。Provider セクション（Google / Groq）と Game Settings セクションを表示し、現在の Config 入力カーソル位置に端末カーソルを置く。
- `src/presentation/ui/render/loading.rs`
  - Loading 画面を描画する。
- `src/presentation/ui/render/typing.rs`
  - Typing 画面を描画する。出題文字列領域と WPM 線グラフ領域を分離して配置する。
  - `Target Text` ブロックは、出題本文の前に2行、出題本文の後に2行の空行を持つ行リストとして描画する。
  - 出題本文が折り返される場合は、折り返し後の本文行全体を本文として扱い、その後に2行の空行を置く。
  - `Target Text` ブロックは枠線込みで最低8行を確保する。WPM グラフと両立できない高さでは、グラフを非表示にして本文領域を優先する。
  - 出題文字列上には端末カーソルを表示せず、現在入力すべき文字を `Yellow + Bold` で強調する。
  - 未来の未入力文字は `Gray`、正答済み文字は `Green`、誤入力位置は `White` on `Red` background で描画する。
  - 現在文字の強調は、表示済みの折返し結果と一致するように出題本文の文字 index で決める。
  - 補助関数を先に、`#[cfg(test)] mod tests` をファイル末尾に置くことで、Clippy の構造警告を避ける。
- `src/presentation/ui/render/result.rs`
  - Result 画面を描画し、入力文字数、ミス数、正確率、経過時間、WPM、保存済み履歴の統計、最終 WPM 線グラフを表示する。
- `src/presentation/ui/render/stats.rs`
  - Stats 画面を描画し、保存済み履歴の自己ベスト、平均、直近10回、頻出ミス文字を表示する。
- `src/presentation/ui/render/wpm_graph.rs`
  - Typing / Result 両画面で共通利用する WPM グラフ描画補助を担当する。
  - `Canvas` と折れ線描画を使い、高い線分をオレンジで強調する。
  - WPM グラフの `Block` 枠線色を共有定義として持ち、Typing / Result の WPM Trend 枠線だけを薄い黄色で描画する。
- `src/usecase/wpm.rs`
  - WPM 計算ロジックを提供する。
- `src/usecase/history_stats.rs`
  - 保存済み履歴から自己ベスト、平均、直近推移、頻出ミス文字を集計する。
- `src/config/mod.rs`
  - 設定永続化と履歴永続化の入口を提供する。
- `src/config/paths.rs`
  - 設定ファイルと鍵ファイルの探索を担当する。
- `src/config/crypto.rs`
  - API key の暗号化・復号を担当する。
- `src/config/storage.rs`
  - 設定の保存形式変換、互換復元、ファイル入出力を担当する。
- `src/config/history_storage.rs`
  - `history.json` の読み書きと保存ディレクトリ作成を担当する。
- `src/usecase/generate_sentence.rs`
  - ローカル生成と外部 API 生成の統一入口を提供する。

## 実行フロー

1. `main` が設定を読み込む。
2. `main` が端末と音声、タイマースレッドを初期化する。`sound_enabled` が `true` の場合のみ BGM を開始する。
3. `runtime` がイベントループを実行し、`AppState` ごとの入力処理を分岐する。
4. タイトルメニューは現在の設定に応じて表示可能な項目だけを描画し、上下キー移動も同じ項目リストを巡回する。
5. タイトルメニューの開始系項目を選択した場合は、選択項目に対応する生成元と制限時間モードを `App` に反映したうえで別スレッドの文字列生成を開始し、結果をチャネルで受け取る。`Practice Mode` は `Local` 生成と `timeout=0` を組み合わせる。
6. `Start Game with Rhythm` は `Local` 生成を指定して同じ生成ジョブ経路を使い、生成結果受領後に通常 `Typing` ではなく `RhythmTyping` へ遷移する。
7. 外部 API 生成の場合、`usecase::generate_sentence` がプロンプト内にリクエストごとの variation seed と日常的な場面カテゴリを含める。
8. 生成結果は正規化層で ASCII ベースへ整形し、`TextScale` から算出した目標文字数以下に切り詰める。
9. `Typing` 中はタイマースレッドの経過秒数を参照し、WPM を再計算して履歴へ追加しながら、完了またはタイムアウトで `Result` へ遷移する。
10. `Typing` 中の WPM 履歴は、入力操作がない区間でも 2 秒の猶予までは直前の WPM 推移を維持し、猶予経過後に 0 として記録する。
11. `timeout=0` の場合はタイムアウト遷移を行わず、全文入力完了まで `Typing` を維持する。
12. `Typing` 中は strict 判定を行い、誤入力では入力位置を進めず、`Backspace` では現在入力中文字列だけを減らす。誤入力時は本来入力すべきだった正解側文字をセッション内のミス文字として記録する。
13. `Typing` 中に `Esc` を押した場合は `Menu` に戻り、進行中のセッションを破棄する。
14. `RhythmTyping` 中は経過時間と `RhythmSpeed` から各文字の現在列を計算し、画面左端から3文字目の `^` 位置を基準に描画と入力判定を行う。
15. `RhythmTyping` 中は空白を入力対象にせず、文字間隔としてのみ保持する。入力判定は内側許容幅を `Hit`、外側許容幅を `OK`、不一致または通過を `Miss` として集計し、直近判定、Miss 数、Hit+OK 数をリアルタイム表示する。
16. `RhythmTyping` 中は `Hit` と `OK` の連続数をコンボとして `RhythmSession` に保持し、`Miss` で 0 に戻す。表示層は 2 以上のコンボだけを `^` 付近に描画する。
17. `RhythmTyping` 完了時は履歴保存を行わず、リズムモード専用指標を `Result` へ表示する。
18. Timed セッション完了時は Result 遷移前に現在結果を `history.json` へ追記し、Practice Mode は保存をスキップする。
19. `Result` 描画時は総入力数と `incorrects` から正確率を算出し、未入力終了時は `0.0%` を表示する。
20. `Result` 描画時は `App` が保持している `wpm_history` をそのまま使い、タイピング終了時点のグラフを固定表示する。リズムモードでは WPM グラフを表示しない。
21. `Stats` 選択時は保存済み履歴から集計済み統計を表示し、`Enter` または `Esc` で Menu に戻る。
22. 終了時は raw mode、画面、スレッド、BGM を順に停止する。

## 外部 API 生成プロンプト

- Google AI Studio と Groq は同じ `build_prompt` で生成したプロンプトを使う。
- `build_prompt` は目標文字数、variation seed、日常的な場面カテゴリを含める。
- variation seed と場面カテゴリはリクエストごとに生成し、Config には保存しない。
- API 側の出力トークン上限は追加せず、最終文字数は既存の正規化処理で制御する。

## Config 入力編集

- Config 画面では、`Up` / `Down` / `Tab` が編集対象フィールドを移動する。
- 文字列フィールドでは、`Left` / `Right` がフィールド内の入力位置を移動する。
- 文字入力は現在入力位置へ挿入し、`Backspace` は現在入力位置の直前の文字を削除する。
- フィールド移動時は、移動先フィールドの末尾へ入力カーソルを置く。
- API key 欄は実値を保持したまま、描画時だけ実文字数と同じ長さの `*` に置き換える。
- `SoundEnabled` は Space トグル専用の非文字列フィールドとして扱い、文字列編集操作では変更しない。

## 設定保存

- 保存先は優先パスの `~/.config/rtyping/` 配下を使う。
- `config.json` には URL、モデル、暗号化済み API key を保存する。
- `config.json` の `game` セクションにタイムアウト、テキスト量、周波数、サウンド設定、リズムモード速度を保存する。古い設定ファイルでリズムモード速度がない場合は既定値 2 を使う。
- `config.key` は別ファイルで管理し、起動時は優先パスと互換パスの候補を順に試す。
- 旧 AEAD ラベルと旧 XOR 形式の API key も復元対象に含める。
- `history.json` には Timed セッションの成績履歴を保存する。
- `history.json` が存在しない場合は空履歴、壊れている場合は警告付き空履歴として扱う。

## 開発環境設定

- `Cargo.toml` の `[lints.clippy]` は Cargo 標準の lint 設定として維持する。
- エディタ拡張のスキーマが Cargo の lint テーブルに追従していない場合は、`.vscode/settings.json` の TOML 検証設定で対象診断を抑制する。
- manifest 構文の確認は `cargo metadata --no-deps --format-version 1` を用いる。

## リリース自動化

- `.github/workflows/version-check.yml` は `Cargo.toml` の package version を Release の source of truth とする。
- `check-version` job は現在 version と直前コミットの version を比較し、変更時だけ後続 job を実行させる。
- Release 前検証 job は `make check` と `make build` を実行し、lint 抑制や warning 回避ではなく現行の品質ゲートをそのまま使う。
- OS 別 build job は `cargo build --release --target ...` と成果物 upload だけを担当し、GitHub Release は作成しない。
- 単一の release job が全 artifact を download し、`RELEASE_NOTES.md` を生成して `softprops/action-gh-release` に渡す。
- リリースノートは Git の前回 version tag から現在 commit までの commit subject を含め、成果物名を列挙する。
- Release 作成を matrix job から分離することで、同じ tag / Release に対する並列書き込みを避ける。

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
  - ローカル生成、URL 組み立て、設定不足時の失敗、外部 API 用プロンプト、正規化処理を確認する。
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
- リズムモード
  - 速度設定の境界値、メニュー遷移、Local 生成経路、時間経過による表示位置、入力許容幅、通過 miss、結果指標分離を確認する。
- `src/usecase/history_stats.rs`
  - 自己ベスト、平均、直近10回、頻出ミス文字の集計を固定する。
- `src/config/history_storage.rs`
  - 履歴ファイルの missing / broken / save round trip を確認する。

## 保守メモ

- UI 仕様変更を伴わない内部整理では、まず `runtime/session` と `runtime/input` の責務境界を確認する。
- 新しい生成元や設定項目を追加する場合は、`domain::config::AppConfig`、`ConfigField`、`provider_config_for_source`、描画処理の順に追うと全体を把握しやすい。
- タイトルメニュー項目を変更する場合は、`MenuItem`、`App` の選択遷移、`runtime/input/menu` の確定処理、`render/menu` の表示を同時に更新する。
- タイトルメニューの選択マーカーだけを変更する場合は、`src/presentation/ui/render/menu.rs` の `menu_line` に閉じ、`MenuItem` と `runtime/input/menu` は変更しない。
- `Practice Mode` を追加する場合は、メニュー項目の並び順と `timeout=0` の一時上書きが結果画面復帰後に通常値へ漏れないように確認する。
- 詳細なミス統計を追加する場合は `App` のカウンタ追加だけでなく、`Backspace` を含む入力イベント定義と結果画面文言を同時に見直す。
- 履歴統計を追加する場合は Result と Stats で同じ集計ロジックを使い、描画側で再計算しない。
- タイムアップなしの練習モードを追加する場合は、`timeout=0` の扱いとタイマー停止条件を先に固定する。
- Typing / Result 画面のレイアウト変更時は、主要テキストの可読性、Sparkline 領域との非重複、狭い端末での退避挙動を同時に確認する。
- Typing 画面のグラフ色分けを追加する場合は、通常色と強調色の境界を履歴の高値に合わせ、無入力区間は 0 に落とす。
- Typing 画面の現在位置表示を変更する場合は、出題文字列の可読性を損なわないよう、端末カーソルではなく文字スタイルで位置を示す。
- Typing 画面の現在文字強調を調整する場合は、文字数からの単純換算ではなく、折返し後に描画される各文字の index に合わせる。改行ごとのずれはこの層で吸収する。
- Typing 画面の `Target Text` ブロック余白を変更する場合は、本文前後の空行と現在文字強調の index 対応を維持する。
- `GameSettings` のフィールドは `String` 型で保持する。これにより UI の入力処理が統一され、数値バリデーションは保存時または使用時に行う。
- リズムモードの速度も `GameSettings` の文字列フィールドとして保持し、使用時に 1 から 5 の範囲へ正規化する。
- Rust モジュールでテストを追加する場合は、通常項目の後に `#[cfg(test)] mod tests` を置く配置を維持する。
