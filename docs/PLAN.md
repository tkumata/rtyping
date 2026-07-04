# PLAN

## 目的

- Stats 画面を、保存済み Timed 履歴の現在地と改善点が一目で分かる成績ダッシュボードへ改善する。
- 既存の履歴保存形式と集計定義を変更せず、`HistoryStats` の表示方法だけを見直す。
- Result 画面、入力処理、履歴永続化、リズムモードには影響させない。

## 作業項目

1. 現在の Stats 描画、履歴集計値、描画テストを確認する。
2. REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、TASK、README を Stats 画面改善方針に同期する。
3. Stats 画面上段に Best WPM、Avg WPM、Avg Accuracy、Runs の KPI 表示を追加する。
4. Stats 画面中段に直近10回 WPM の数値列とミニグラフを表示する。
5. Stats 画面下段に頻出ミス文字の横棒グラフを表示する。
6. 履歴がない場合は空統計ではなく、Timed セッション完了を促す空状態メッセージを表示する。
7. Stats 描画テストを新しい主要文言に合わせて更新する。
8. `make check`、`make build` を確認する。

## 確認観点

- Stats 画面で KPI、直近 WPM 推移、頻出ミス文字が視覚的に分かれて表示されること。
- 履歴がない場合でも戻り操作案内と空状態メッセージが表示されること。
- 既存の `HistoryStats` 集計定義、履歴保存形式、Stats 画面の `Enter` / `Esc` 復帰が変わらないこと。
- Result 画面の履歴サマリ表示が退行しないこと。
