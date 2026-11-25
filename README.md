# 🦀 Rust 学習スタジオ

Rust を基礎から学び、最終的に **TUI を含むターミナルアプリを全部自作できる** ようになるための学習環境です。
解説・演習・プロジェクト・卒業課題を、**localhost で動く学習サイト** からすべて操作できます。

> [!NOTE]
> **このリポジトリについて**
> これは作者が **個人で Rust を学習するため** に用意した教材・学習環境です。
> 学習サイト (studio) のコード、解説文、演習、模範解答、プロジェクト、卒業課題を含め、**中身はすべて AI (LLM コーディングエージェント) に生成させたもの** で、人間は方針の指示と動作確認のみを行っています。
> 一通りのビルド・テストによる検証は済んでいますが ([VALIDATION.md](VALIDATION.md))、解説の内容に誤りや不正確な点が含まれている可能性があります。公式ドキュメント ([The Rust Programming Language](https://doc.rust-lang.org/book/) など) と併せて参照してください。

## 必要なもの

- Rust stable (Edition 2024。構築・検証環境は rustc/cargo 1.96)
- macOS または Linux (raw モードを自作する一部の教材は Unix 向け)
- 初回のみ crates.io から依存クレートを取得するためのネットワーク接続

```sh
git clone https://github.com/Buchi-dev-dzn/rust_learning.git
cd rust_learning
cargo studio
```

## 起動 (コマンドはこれだけ)

```sh
cargo studio
```

→ ブラウザで **http://127.0.0.1:7878** を開きます (macOS では自動で開きます)。
初回はサーバ自体のビルドに 1〜2 分かかります。終了は Ctrl-C。

- ビルド・テスト・実行はすべてブラウザのボタンで行います。
- TUI アプリは「ターミナル」ページ (ブラウザ内ターミナル, xterm.js + PTY) でそのまま動きます。
  本物のターミナルで動かしたい場合は、表示されるコマンド (`cargo run -p p05_editor -- README.md` など) をリポジトリ直下で実行してください。
- 7878 が使用中なら 7879, 7880… を自動で使います (`STUDIO_PORT=8000 cargo studio` で変更可)。
- ブラウザを自動で開かないようにするには `STUDIO_NO_OPEN=1 cargo studio`。


## 学習の流れ

| Stage | 内容 | プロジェクト |
|---|---|---|
| 1 基礎 | 変数・関数・所有権・借用・構造体・enum・コレクション・エラー処理・モジュール | minigrep / todo CLI |
| 2 中級 | トレイト・ジェネリクス・ライフタイム・イテレータ・スマートポインタ・テスト・並行処理 | — |
| 3 システム/CLI | 標準入出力・ファイル・引数・終了コード・Unicode の表示幅 | — |
| 4 ターミナル/TUI | ANSI・raw モード・crossterm・TUI アーキテクチャ・ratatui・自作フレームワーク | ansi_raw / crossterm 入門 / エディタ / ratatui ファイラ / 自作 TUI |
| 5 卒業課題 | 仕様書 + 受け入れテストだけで作る総合演習 F1〜F7 | — |

サイトのページ構成: ホーム / ロードマップ / ステージ概要 / 解説 / 演習一覧・演習 / 模範解答 / プロジェクト・STEP / 卒業課題 / ターミナル / 進捗 / 早見表

## ディレクトリ

```
.
├── studio/          学習サイト (axum サーバ + askama テンプレート + 素の JS)
├── docs/            章ごとの解説 (Markdown)
├── exercises/       あなたが解く演習ファイル (初回起動時に course/exercises からコピーされる / git 管理外)
├── projects/        プロジェクト: 完成版 (src/) と、あなたが書き進める骨組み (starter/)
├── finals/          卒業課題: あなたが編集する src/ と受け入れテスト tests/
├── course/          教材の原本 (演習の初期状態・模範解答・ヒント・course.toml)
├── PLAN.md          構築時の設計プラン
├── VALIDATION.md    検証結果
└── progress.json    進捗 (自動保存 / git 管理外)
```

- 演習ファイルは VS Code などのエディタで直接編集しても OK。保存するとブラウザが検知して自動でテストします。
- 「リセット」は `course/` の原本から初期状態に戻します。
- `exercises/`・`finals/*/src`・`projects/*/starter` があなたの作業場所です。fork して自分の進捗をコミットしていくのもおすすめです (`exercises/` と `progress.json` は `.gitignore` 済みなので、残したい場合は `.gitignore` から外してください)。

## 安全について

このサーバはあなたの書いたコードをビルド・実行し、シェル (PTY) も提供します。
そのため `127.0.0.1` にのみ bind し、他サイトからのリクエスト (Origin / Host がローカル以外) は拒否します。外部に公開しないでください。

## 教材を追加・修正するには

規約は [course/AUTHORING.md](course/AUTHORING.md) を参照。ファイルを置くだけでサイトに反映されます。
教材の整合性チェック (全模範解答が合格・初期状態の演習は不合格 など):

```sh
cargo test -p studio
```
