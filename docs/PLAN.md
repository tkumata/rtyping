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
- WPM グラフ、ヘッダー、フッター、入力判定、カーソル形状、カーソル位置の仕様は変更しない。

## 作業項目

1. `src/presentation/ui/render/typing.rs` の `Target Text` ブロック描画を確認する。
2. 出題本文の描画行を、折り返し後の本文行と上下2行の空行で構成する。
3. `typing_cursor_position` の基準行が、本文開始位置のまま維持されることを確認する。
4. `make check`、`make build` を確認する。

## 確認観点

- `Target Text` ブロック内で本文の上に2行の空行があること。
- 折り返された本文の下に2行の空行があること。
- 本文が1行の場合も複数行の場合も、カーソルが現在入力位置の右側に表示されること。

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
