# Current Task

- Summary: `version-check.yml` の自動リリースで、OS 別 build と Release 作成を分離し、明示的なリリースノートを生成する。
- Docs:
  - Done: `docs/TASK/current-task.md` を `docs/TASK/202605090932.md` に退避する。
  - Done: PLAN、REQUIREMENTS、SPECIFICATIONS、DESIGN、ADR を Release ノート改善仕様に同期する。
- Implementation:
  - Done: `.github/workflows/version-check.yml` を build artifact upload と単一 release job 構成へ変更する。
  - Done: Release job で `RELEASE_NOTES.md` を生成し、`body_path` で GitHub Release に渡す。
  - Done: README.md に自動リリース運用を反映する。
- Verification:
  - Done: `make check` を実行する。
  - Done: `make build` を実行する。
