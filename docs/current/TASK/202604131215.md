# Current Task

- Date: 2026-04-12 14:53
- Summary: CLI 引数を撤廃し、タイムアウト・テキスト量・サウンド設定などをすべて Config 画面で編集・ファイル保存できるようにする。
- Scope:
  - すべての CLI 引数（--timeout, --level, --freq, --sound, --google, --groq）を削除する
  - GameSettings（timeout, text_scale, freq, sound_enabled）を AppConfig に追加する
  - Config 画面にゲーム設定セクションを追加し、編集・保存できるようにする
  - config.json のシリアライズにゲーム設定を追加する（#[serde(default)] により後方互換を維持）
  - App::new() を AppConfig のみ受け取るよう簡素化する
  - clap 依存を削除する
- Verification:
  - cargo build で警告・エラーがないこと
  - cargo test で全テストがパスすること
  - Config 画面に Game Settings セクションが表示されること
  - 設定保存でゲーム設定がファイルに永続化されること
