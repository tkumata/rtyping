# Current Task

- Date: 2026-04-10 08:23
- Summary: タイピング中に小さな WPM グラフ領域を追加し、`ratatui` の `Sparkline` で推移をリアルタイム表示する。
- Scope:
  - Update requirements, ADR, specifications, and design docs for the WPM sparkline feature.
  - Add a dedicated graph area to the `Typing` screen without overlapping the target text.
  - Track WPM history per typing session and reset it between rounds.
  - Keep the existing numeric WPM display, strict typing flow, and result metrics consistent.
- Verification:
  - Confirm the `Typing` screen shows a live WPM trend graph.
  - Confirm the graph never overlaps the target text area.
  - Confirm narrow terminal layouts still avoid overlap.
  - Confirm WPM history is reset when a new typing session starts.
