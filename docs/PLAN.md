# PLAN

## 目的

- Timed セッションの結果を履歴として保存する。
- Result 画面で自己ベスト、平均、直近推移、頻出ミス文字を確認できるようにする。
- Menu に `Stats` を追加し、保存済み履歴の統計サマリを確認できるようにする。

## 作業項目

1. 現在の Result 指標、Menu 遷移、設定保存パスを確認する。
2. REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、TASK を履歴管理仕様に同期する。
3. `~/.config/rtyping/history.json` の読み書きと統計集計を追加する。
4. Timed セッション完了時だけ履歴を保存し、Practice Mode は保存対象外にする。
5. Result 画面と Stats 画面に履歴統計を表示する。
6. `make check`、`make build` を確認する。

## 確認観点

- Timed セッション完了時だけ履歴が追加されること。
- Practice Mode 完了時に履歴が増えないこと。
- Result 画面の既存指標と最終 WPM グラフが壊れないこと。
- Stats 画面で保存済み履歴の統計を確認できること。

## タイトルメニュー選択マーカー変更

## 目的

- 起動直後のタイトル画面で、現在選択中のメニュー項目を `▶︎` で表示する。
- 表示記号だけを変更し、メニュー項目、選択順、キー操作、状態遷移は変更しない。

## 作業項目

1. `src/presentation/ui/render/menu.rs` の選択マーカー描画箇所を確認する。
2. 選択中の項目だけ `▶︎` を表示し、非選択項目は既存と同じ空白プレフィックスにする。
3. タイトル画面以外の `Typing`、`Config`、`Loading`、`Result`、`Stats` の表示へ影響させない。
4. `make check`、`make build` を確認する。

## 確認観点

- 起動直後に選択される `Start Game` の左に `▶︎` が表示されること。
- 上下キーで選択を移動しても、選択中の項目だけに `▶︎` が表示されること。
- `Enter`、`Esc`、`h` の既存操作が変わらないこと。

## Target Text ブロック余白変更

## 目的

- Typing 画面の `Target Text` ブロックで、出題本文の上下にそれぞれ2行の空行を確保する。
- 出題本文が複数行に折り返される場合でも、本文の後ろに2行の空行を維持する。
- WPM グラフ、ヘッダー、フッター、入力判定、現在文字強調の仕様は変更しない。

## 作業項目

1. `src/presentation/ui/render/typing.rs` の `Target Text` ブロック描画を確認する。
2. 出題本文の描画行を、折り返し後の本文行と上下2行の空行で構成する。
3. 本文開始位置と文字 index の対応が維持されることを確認する。
4. `make check`、`make build` を確認する。

## 確認観点

- `Target Text` ブロック内で本文の上に2行の空行があること。
- 折り返された本文の下に2行の空行があること。
- 本文が1行の場合も複数行の場合も、現在入力すべき文字が正しく強調されること。

## WPM Trend ブロック枠線色変更

## 目的

- WPM Trend ブロックの枠線だけを薄い黄色で表示する。
- WPM グラフの線色、高値強調、履歴更新、レイアウト、Result 画面の最終グラフ表示は変更しない。

## 作業項目

1. `src/presentation/ui/render/wpm_graph.rs` の共有 WPM グラフ描画を確認する。
2. WPM グラフ用 `Block` の枠線色だけを薄い黄色系の色へ変更する。
3. Typing 画面の `WPM Trend` と Result 画面の `Final WPM Trend` が同じ共有枠線色を使うことを確認する。
4. `make check`、`make build` を確認する。

## 確認観点

- WPM Trend ブロックの枠線が薄い黄色で表示されること。
- グラフ本体の緑系線色と高値部分のオレンジ強調が維持されること。
- WPM 履歴、入力判定、画面遷移、レイアウトが変わらないこと。

## Config 画面入力カーソル編集

## 目的

- Config 画面の文字列入力欄で、左右キーにより入力位置を移動できるようにする。
- 文字入力は現在のカーソル位置へ挿入し、Backspace はカーソル直前の文字だけを削除する。
- API key は平文表示せず、実文字数と同じ長さのマスク表示で入力位置を判別できるようにする。

## 作業項目

1. `src/presentation/ui/app/config_editor.rs` の Config 編集状態にカーソル位置を追加する。
2. `src/runtime/input/config_screen.rs` で `Left` / `Right` を入力欄内カーソル移動として扱う。
3. `src/presentation/ui/render/config_screen.rs` のカーソル座標を、文字列末尾ではなく編集カーソル位置に同期させる。
4. API key 欄の表示マスクを実文字数と同じ長さにし、平文を表示しない。
5. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## 確認観点

- `Up` / `Down` / `Tab` の入力欄移動が既存通り動くこと。
- `Left` / `Right` が現在の文字列入力欄の範囲内で停止すること。
- カーソル位置で文字が挿入され、Backspace がカーソル直前の文字を削除すること。
- `SoundEnabled` は既存通り Space トグルのみで編集でき、文字列編集操作で値が壊れないこと。

## 外部 API 生成の固定化回避

## 目的

- Google AI Studio と Groq に同じプロンプトを毎回送らないようにし、同じような文章が続く問題を解消する。
- 文字数は既存どおり `TextScale` から算出し、最終的な出題文を目標文字数以下に保つ。
- API 側の出力トークン上限は追加せず、短すぎる文章への退行を避ける。

## 作業項目

1. 外部 API 用プロンプトにリクエストごとに変わる variation seed を含める。
2. variation seed に対応する日常的な場面カテゴリをプロンプトへ含める。
3. Google AI Studio と Groq の request body 形式は既存のまま維持する。
4. `normalize_sentence` による文字数上限保証を維持する。
5. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## 確認観点

- 連続してプロンプトを生成したとき、variation seed が変わること。
- プロンプトに目標文字数と多様化指示が含まれること。
- Google AI Studio と Groq の request body に短すぎる出力を誘発する上限を追加しないこと。
- 最終的な出題文が目標文字数を超えない既存仕様を維持すること。

## GitHub Release ノート改善

## 目的

- `Cargo.toml` の version 更新を起点にした既存の自動リリース運用を維持する。
- OS 別ビルドジョブから Release 作成処理を分離し、GitHub Release を1回だけ作成する。
- GitHub の自動生成だけに依存せず、利用者が変更点と成果物を確認しやすいリリースノートを生成する。

## 作業項目

1. `.github/workflows/version-check.yml` の version 判定、検証、ビルド、Release 作成の責務を整理する。
2. release build matrix は成果物作成と artifact upload に限定する。
3. 単一の release job で artifact を収集し、Markdown のリリースノートを生成する。
4. `softprops/action-gh-release` は `body_path` と収集済み成果物を使って1回だけ実行する。
5. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## 確認観点

- version 変更がない場合はリリース関連 job が実行されないこと。
- version 変更時に `make check` と `make build` が Release 前に通ること。
- Linux と macOS の成果物が同じ Release に添付されること。
- リリースノートに version、変更履歴、成果物一覧が含まれること。

## Typing 画面の現在文字強調

## 目的

- Typing 画面で端末カーソルが `,` と `.` などの細い記号を隠す問題を解消する。
- 端末カーソルを非表示にしても、現在入力すべき文字を判別できるようにする。
- 入力判定、strict 挙動、WPM グラフ、Result / Stats 画面は変更しない。

## 作業項目

1. `src/presentation/ui/render/typing.rs` の `Target Text` 文字別スタイルを確認する。
2. Typing 画面では端末カーソルの位置指定を行わない。
3. 現在入力すべき文字を `Yellow + Bold` で表示する。
4. 未入力の未来文字を `Gray` で表示し、現在文字との差を強める。
5. 正答済み文字は `Green`、誤入力位置は `White` on `Red` background を維持する。
6. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## 確認観点

- Typing 画面で端末カーソルが出題文字に重ならないこと。
- 現在入力すべき文字が黄色太字で表示されること。
- 未来の未入力文字が灰色で表示されること。
- 正答済み文字と誤入力位置の既存配色が維持されること。

## 外部プロバイダ開始メニューの表示条件

## 目的

- Google AI Studio と GroqCloud の設定が未完了の場合、タイトル画面に外部 API 開始メニューを表示しない。
- 設定が未完了とは、`API URL`、`API Key`、`Model` のいずれかが空または空白のみである状態を指す。
- Local 開始、Practice Mode、Stats、Config の導線は常に維持する。

## 作業項目

1. `ProviderConfig::is_ready()` をタイトルメニューの表示条件として使う。
2. `App` が現在表示可能なメニュー項目リストを返すようにし、描画と上下キー移動で同じリストを使う。
3. Google AI Studio と GroqCloud のメニューは完全設定時だけ表示する。
4. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## 確認観点

- 初期設定が空の場合、タイトル画面に `Start Game via Google AI Studio` と `Start Game via GroqCloud` が表示されないこと。
- 各プロバイダの `API URL`、`API Key`、`Model` がすべて空でない場合だけ、対応する開始メニューが表示されること。
- 上下キーの巡回選択が非表示項目を選択しないこと。
- 外部プロバイダ設定の完全性判定が生成処理側の設定不足判定と矛盾しないこと。

## リズムモード

## 目的

- 起動直後のタイトルメニューから、通常 Typing とは別の「リズムモード」を開始できるようにする。
- 画面右から左へ流れる文字を、画面左端から3文字目の `^` マーク位置で入力する体験を追加する。
- リズムモードの成績を通常モードと分離し、初期指標として typed、correct、miss、accuracy を表示する。

## 作業項目

1. タイトルメニューに `Start Game with Rhythm` を追加する。
2. `Config` 画面の Game Settings に `RhythmSpeed` を追加し、秒間 1 から 5 文字の範囲で扱う。初期値は 2 とする。
3. リズムモード開始時は既存の AI を利用しないローカル生成ロジックで出題文字列を準備する。
4. 出題文字列を、空白を入力対象にしないリズム用ノート列へ変換する。
5. リズム画面では文字を右から左へ流し、`^` を超えた文字は表示しない。
6. `^` 位置の前後に入力許容幅を設け、許容幅内の対象文字に対する入力だけを正答とする。
7. 許容幅の内側を `Hit`、外側を `OK`、不一致または通過を `Miss` として分類し、直近判定、Miss 数、Hit+OK 数をリアルタイム表示する。
8. リズムモード完了時は通常 Typing の WPM / 履歴とは別の結果指標を表示する。
9. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## リズムコンボ表示

## 目的

- リズムモード中の連続成功を可視化し、`Hit` と `OK` の連続数をユーザーが入力位置付近で把握できるようにする。

## 作業項目

1. リズムセッションに現在の連続成功数を保持する。
2. `Hit` または `OK` の入力判定で連続成功数を 1 増やす。
3. `Miss` の入力判定または通過 Miss で連続成功数を 0 に戻す。
4. 連続成功数が 2 以上の場合だけ、`^` 付近に `{count} Combo!!` を表示する。
5. README と関連ドキュメントを同期し、`make check`、`make build` を確認する。

## 確認観点

- 初期メニューに `Start Game with Rhythm` が表示されること。
- 外部 API 設定が未完了でもリズムモードを開始できること。
- `RhythmSpeed` の使用値が 1 から 5 に制限され、未設定や不正値では 2 になること。
- 空白が入力対象にならず、文字間隔としてのみ表示されること。
- `^` を通過した文字が表示対象から外れること。
- 直近判定、Miss 数、Hit+OK 数がリズムモード中に更新されること。
- 連続する `Hit` と `OK` でコンボ数が増え、2 以上のときだけ `^` 付近に表示されること。
- `Miss` でコンボ数が 0 に戻ること。
- リズムモードの結果表示が通常モードの WPM / 履歴保存と混ざらないこと。
