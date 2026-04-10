# Current Task

- Date: 2026-04-10 09:15
- Summary: 結果画面にも最終的な WPM グラフを表示し、タイピング中のリアルタイム表示と役割を分ける。
- Scope:
  - Update requirements, ADR, specifications, and design docs for final WPM graph rendering on the `Result` screen.
  - Reuse the existing `wpm_history` so the `Typing` screen remains real-time and the `Result` screen shows the final snapshot.
  - Adjust result-screen layout without regressing existing metrics or navigation text.
- Verification:
  - Confirm the `Typing` screen still shows a live WPM trend graph while typing.
  - Confirm the `Result` screen shows the final WPM trend graph after finishing or timing out.
  - Confirm the graph does not overlap existing result metrics in narrow layouts.
