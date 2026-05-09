# Current Task

- Summary: Google AI Studio と Groq の外部 API 生成で、既存の文字数上限を維持したまま、毎回同じ文章へ固定される問題を避ける。
- Docs:
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR を外部 API 生成プロンプト固定化回避仕様に同期する。
- Implementation:
  - Done: 外部 API 用プロンプトに variation seed と場面カテゴリを追加する。
  - Done: Google AI Studio と Groq の request body 形状を維持する。
  - Done: README.md に外部 API 生成プロンプトの固定化回避を反映する。
- Verification:
  - Done: variation seed とプロンプト内容のユニットテストを追加する。
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
