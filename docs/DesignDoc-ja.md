# タイピングアプリケーション - R-Typing

**Last Updated:** 2024/12/20

## 背景

- **🦀 Rust の学習**  
  Rust の基本を実践的に学習したかった。
  
- **⌨️ キーボードの活用**  
  自作キーボードをフル活用するためひたすらタイピングしたかった。

## 🎯 アプリケーションの目的

### 主な目標

1. **Rust のスキルアップ**  
  Rust の以下の概念やツールの学習:
  - 変数、スコープ、型
  - ベクター、関数
  - 所有権
  - 構造体
  - スレッドと同期・非同期処理
  - クレートの活用
  - Assets のビルドへの組み込み
  - Cargo とクロスプラットフォームコンパイル

2. **タイピングの楽しさを追求**  
   手軽に、ただひたすらタイピングできる環境を提供。

## 📔 仕様

- **単語の抽出**  
  `/usr/bin` 配下からランダムな単語を抜き出し、目標文字列として表示する。

- **タイピングゲームの基本動作**  
  - 目標文字列を入力。
  - 制限時間内に全てタイプする。

- **UI とフィードバック**  
  - 制限時間を画面の左上にリアルタイムで表示。
  - 入力中の文字と目標文字列を重ねて表示し、進捗に応じて色を変える。
  - Typo 部分を別の色で強調。

- **音楽と効果音**  
  - 背景で BGM を再生。
  - タイプごとに効果音を鳴らす (引数で音を変更可能)。

- **タイピング統計**  
  WPM (Words Per Minute) の集計を実装。

## 🥅 Goals

- Rust の基本概念を理解し、実践に活かす。
- 次のステップ (高度な Rust 学習や別プロジェクト) に備える。

## Non-Goal

- 英文の自動生成機能の実装 (将来の課題とする)。

## 🐛 既知の問題

- **目標文字列の長さの制限**  
  Raw モード使用時、目標文字列が長すぎると `std::io` の挙動が不安定になる。

## 📋 Todo

- UI の再設計

Title Screen

```plaintext
                       Let' begin typing!
                         Go for high WPM.
 ____     _____            _    Credit 01
|  _ \   |_   _|   _ _ __ (_)_ __   __ _ 
| |_) | _  | || | | | '_ \| | '_ \ / _` |
|  _ < (_) | || |_| | |_) | | | | | (_| |
|_| \_\    |_| \__, | .__/|_|_| |_|\__, |
               |___/|_|            |___/ 
                               © 2025 kmt
Press *ENTER* key to start.🚀
```

Playing screen

```plaintext
Time: 00 sec / Types: 00 chars / Misses: 0 chars
-------------------------------------------------------------------------------
There is three man.........
A time pencil when a find........
-------------------------------------------------------------------------------
```

Result screen

```plaintext
Time: 60 sec / Types: 200 chars / Misses: 3 chars
-------------------------------------------------------------------------------
There is three man.........
A time pencil when a find........
-------------------------------------------------------------------------------

,-----------------------------.
| 🏁 Result                   |
|-----------------------------|
| Total Time      : 060 sec   |
| Total Typing    : 200 chars |
| Total Misses    : 003 chars |
| Words Per Minute: 023.1 wpm |
`-----------------------------'
```
