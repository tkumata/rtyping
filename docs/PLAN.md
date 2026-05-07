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
