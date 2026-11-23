# 画面遷移時に残る背景色の修正

## Status

- Phase: completed
- Approval: approved

## Goal

画面を切り替えたとき、新しい画面に前の画面の背景色が残らないようにする。

## Scope

- 画面遷移時の描画と、背景色を指定する箇所を調査する。
- 背景色が残る原因に対応する箇所を修正する。
- 既存の描画テストで検出できない再発条件には、回帰テストを追加する。

## Required Behavior

- 画面遷移後の最初の描画で、各位置の背景色は遷移先の画面の指定に従うこと。
- 遷移先で背景色を指定していない位置は、端末の既定の背景色で表示すること。
- Typing の入力ミスと Config の選択中の入力欄は、既存の背景色で強調すること。

## Tasks

### Task 1: 画面遷移後に残る背景色を修正する

- Objective: 背景色が残る原因を切り分け、修正する。
- Scope: `src/runtime/session.rs` の描画ループと `src/presentation/ui/render/` の背景色指定を起点に調べる。描画バッファと端末出力を比較し、原因箇所を特定する。
- Acceptance Criteria: cmux で Stats からタイトル画面に戻った直後に、Stats の背景色が残らない。Typing と Config の強調表示が機能する。
- Verification:
  1. cmux でタイトル画面から Stats を開く。`Enter` または `Esc` でタイトル画面に戻る。修正前後の表示を比較する。
  2. 通常の Typing で入力ミスを表示してから Menu または Result に切り替える。Config の入力欄を選択してから Menu に戻る。強調表示と遷移直後の背景色を確認する。
- Result:
  - cmux で Stats の表示中に付けた検証用の背景色は、修正後、タイトル画面に戻った直後に消えた。
  - Typing の入力ミスと Config の選択欄の強調表示を確認した。タイトル画面に戻ると強調色は消えた。
  - `make check` は全 131 テストが成功した。`make build` も成功した。

## Open Questions

- cmux 以外の端末でも発生するかは未確認。
- 通常操作で報告された症状は再現できなかった。最初に背景色が残った原因は未特定。

## References

- [Issue #169 背景色の修正](https://github.com/tkumata/rtyping/issues/169)
- [描画ループ](../../../src/runtime/session.rs)
- [画面ごとの描画](../../../src/presentation/ui/render/mod.rs)
