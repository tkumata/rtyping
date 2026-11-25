# Issue #172 UI 修正

## Status

- Phase: completed
- Approval: approved

## Goal

Config とプレイ中の画面で罫線を減らし、設定と出題本文を読みやすくする。
Stats では直近の WPM と棒の長さを同じ行で比較できるようにする。

## Scope

- Config、Stats、通常モードの Typing の描画を変更する。
- 承認後、変更する表示について現行の要件・仕様・設計を更新する。

## Non-goals

- 設定の編集操作、設定と履歴の保存形式、WPM の集計方法の変更。
- Result の `Final WPM Trend` とリズムモードの表示変更。

## Required Behavior

- Config は `Google AI Studio`、`Groq`、`Game Settings` の3ブロックだけを青い枠線で囲む。
- 各ブロックは上枠に見出しを表示する。設定項目の間に1行空ける。最後の設定項目の次の行に下枠を表示する。
- Config の操作案内、選択中の項目の強調、入力カーソル、API キーのマスクを維持する。
- Stats 画面の外枠を削除する。
- Stats の `Recent 10 WPM` は、保存順の古いものから1件ずつ縦に表示する。
- 各行の WPM の右に `Frequent Misses` と同じ `■` (U+25A0) を表示する。WPM を10で割り、四捨五入した整数を `■` の個数とする。
- Typing の `WPM Trend` は枠線を削除し、グラフを表示する。
- Typing の `Target Text` は枠線を維持し、枠線色を端末の既定の前景色に合わせる。

## Tasks

### Task 1: Config の枠線と設定項目の間隔を整理する

- Objective: 設定項目と操作案内を読みやすくする。
- Scope: Config の描画と入力カーソルの表示位置。
- Acceptance Criteria: 青い枠線は指定の3ブロックにだけ表示される。各ブロックの項目間に1行の空行があり、最後の項目の直後に下枠がある。
- Verification: 実端末で3ブロックと操作案内を確認する。設定項目を順に選び、カーソル位置とマスク表示を確認する。
- Result: macOS の cmux で青い枠線が3ブロックに限られることと、項目間の空行を確認した。両プロバイダのキーのマスク表示と、選択中の入力欄のカーソル表示を確認した。

### Task 2: 直近の WPM を横棒で表示する

- Objective: 各 WPM と棒の対応を一目で確認できるようにする。
- Scope: Stats の外枠、`Recent 10 WPM`、縦表示に必要な領域配分。
- Acceptance Criteria: Stats の外枠なしで、表示領域が足りるときは直近最大10件を古い順に表示する。4 WPM は `■` 0個、5 WPM と10 WPM は1個、15 WPM は2個とする。棒の開始位置をそろえる。履歴がないときは既存の案内を表示する。
- Verification: 描画バッファで外枠の有無、順序、行数、棒の個数と開始列を確認する。実端末で外枠の有無、数値と棒の整列、`Frequent Misses`、戻る操作の案内を確認する。
- Result: 描画バッファと cmux で、外枠がなく、直近10件が縦に並ぶことを確認した。4 WPM、5 WPM、15 WPM の棒の個数と開始列を確認した。`Frequent Misses` の表示と `Enter` / `Esc` で戻る操作を確認した。

### Task 3: Typing の枠線を調整する

- Objective: グラフ周辺の罫線を減らし、出題本文の枠を端末の既定の前景色で表示する。
- Scope: Typing の `WPM Trend` と `Target Text`。共有グラフの呼び出し元である Result も確認対象に含める。
- Acceptance Criteria: Typing の `WPM Trend` は枠線なしで描画される。`Target Text` の枠線は端末の既定の前景色で表示される。Result の `Final WPM Trend` は既存の枠線で描画される。
- Verification: 実端末で Typing と Result の対象ブロックを確認する。`make check` と `make build` で Task 1〜3 の変更を検証する。
- Result: cmux で枠線なしの `WPM Trend`、端末の既定の前景色の `Target Text` 枠線、薄い黄色の `Final WPM Trend` 枠線を確認した。`make check` は131テストが成功した。`make build` も成功した。

## Open Questions

- なし。

## References

- [Issue #172 UI 修正](https://github.com/tkumata/rtyping/issues/172)
- [現行の要件](../../current/REQUIREMENTS.md)
- [現行の仕様](../../current/SPECIFICATIONS.md)
- [現行の設計](../../current/DESIGN.md)
