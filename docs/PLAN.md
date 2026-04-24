# PLAN

## 目的

- Typing 画面の WPM グラフを見やすくする。
- 高い WPM の区間をオレンジで強調する。
- 入力が止まった区間は 0 として表示する。

## 作業項目

1. 現在の WPM グラフ描画と履歴更新の流れを確認する。
2. Typing 中の履歴更新を「入力がない区間は 0」に合わせて修正する。
3. WPM グラフのバーごとに色を付け、ピーク部分をオレンジで描画する。
4. REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR、TASK を同期する。
5. `cargo test`、`make check`、`make build` を確認する。

## 確認観点

- Typing 中に無入力が続くとグラフが 0 に落ちること。
- 高い WPM のバーだけがオレンジになること。
- Result 画面の既存グラフ表示が壊れないこと。
