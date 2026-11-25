# 教材の書き方 (AUTHORING)

学習サイト `studio` はこのリポジトリのファイルを読み、規約に従ってページを生成する。
**この規約に従う限り、ファイルを追加するだけでサイトに反映される。**

共通ルール
- 言語: 日本語 (コードの識別子は英語)。Rust Edition 2024 / rustc 1.96。
- 対象読者: 他言語経験はあるが Rust は初めての人。最終目標は「TUI を含むターミナルアプリを全部自作できる」。
- 章の ID 一覧・タイトルは `course/course.toml` が正。勝手に章を増やさない。

## 1. 解説 `docs/<chapter_id>.md`

- 1 行目は `# 第N章 タイトル` (23_final は `# 卒業課題の進め方`)。
- 構成目安: この章のゴール → 説明 (小見出し `##`) → コード例 → よくあるコンパイルエラーと読み方 → まとめ → 次へ。
- 分量: 200〜450 行程度。説明は具体的に。TUI につながる話題 (なぜこれが後で必要か) を随所で触れる。
- コード例は ```` ```rust ```` で書く。**`fn main` を含む完結したコード例は、サイト上で「▶ 実行」ボタンが付き、そのまま `rustc --edition 2024` でビルド・実行される**。
  外部クレートを使う例 (crossterm/ratatui) は ```` ```rust,noplayground ```` と書く (実行ボタンなし)。
  断片的なコード (fn main なし) は自動的に実行ボタンなしになる。
- 実行可能な例は 標準ライブラリのみ・標準入力を読まない・数秒以内に終了すること。

## 2. 演習 `course/exercises/<chapter_id>/exNN_<slug>.rs`

- 1 章あたり 1〜3 個。ファイル名順に並ぶ (`ex01_...`, `ex02_...`)。
- 演習 ID は `<chapter_id>/exNN_<slug>` (例: `02_ownership/ex01_move`)。
- **単体ファイル・標準ライブラリのみ**。`rustc --edition 2024 --test <file>` でビルドしてテストを実行し、全テスト合格で「合格」。
- ファイル先頭は `//!` コメントの課題文 (Markdown)。1 行目は `//! # タイトル`。

```rust
//! # 変数とミュータビリティ
//!
//! `TODO` の部分を直して、すべてのテストを通そう。
//!
//! - ゴール: ...
//! - 関連: 第1章「変数」

// TODO: 下の関数を完成させる
fn add(a: i32, b: i32) -> i32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds() {
        assert_eq!(add(2, 3), 5);
    }
}
```

- 初期状態は **必ず不合格** (コンパイルエラー or テスト失敗 / `todo!()` によるパニック)。
- 学習者が直すべき場所に `// TODO:` コメントを置く。テストは学習者が編集しない前提 (触るなと書く)。
- 警告は気にしなくてよい (サーバは `-A warnings` 相当で実行)。

## 3. 模範解答 `course/solutions/<chapter_id>/exNN_<slug>.rs`

- 演習と同じファイル名。課題文コメントはそのまま、TODO を埋めたもの。**全テスト合格必須**。
- 解答の要点を `//` コメントで短く解説してよい。

## 4. ヒント `course/hints/<chapter_id>/exNN_<slug>.md`

- `---` だけの行で区切ると段階的ヒントになる (1 段目は軽く、最後はほぼ答え)。2〜3 段階。

## 5. プロジェクト `projects/<project_id>/`

```
projects/p05_editor/
├── Cargo.toml        # package.name = "p05_editor" (完成版リファレンス実装)
├── src/...
├── STEPS.md          # 手順書
└── starter/
    ├── Cargo.toml    # package.name = "p05_editor_starter" (学習者が書き進める骨組み)
    └── src/...
```

- Cargo.toml は workspace 継承: `edition.workspace = true`, `version.workspace = true`, `publish.workspace = true`。
  依存は `crossterm = { workspace = true }` / `ratatui = { workspace = true }` / `libc = { workspace = true }` のみ使用可。
- 完成版: `cargo build -p <id>` と `cargo test -p <id>` が通ること。ロジック部分 (キー解析・バッファ操作など) には単体テストを付ける。
  TUI は端末に依存する部分 (raw モード・描画ループ) と純粋なロジックを分離し、ロジックをテスト可能にする。
  必ず終了キー (q / Ctrl-Q 等) と、パニック時もターミナルを元に戻す処理を入れる。
- starter: `cargo build -p <id>_starter` が通ること (中身は `todo!()` や `// TODO(STEP n):` コメント)。
- `STEPS.md`:
  - 1 行目 `# タイトル`、続いて概要・完成イメージ・学ぶこと。
  - 各ステップは `## STEP 1: 見出し` 形式 (サイトでは STEP ごとに別ページ)。各 STEP に「やること」「コード例/ポイント」「確認方法」を書く。
  - 4〜8 STEP。最後に「発展課題」。

## 6. 卒業課題 `finals/<final_id>/` (Stage 5)

```
finals/f3_keyparser/
├── Cargo.toml          # package.name = "f3_keyparser"
├── SPEC.md             # 仕様書 (1 行目 `# タイトル`)
├── HINTS.md            # 段階ヒント (`---` 区切り)
├── src/                # 学習者が編集する starter (todo!() 入り、ビルドは通る)
└── tests/acceptance.rs # 受け入れテスト (学習者は編集しない)

course/finals/f3_keyparser/
├── starter/src/...     # src/ の初期状態 (リセット用。finals/<id>/src と完全に同一)
└── solution/src/...    # 模範解答 (これを src にコピーすると受け入れテストが全部通る)
```

- 合格 = `cargo test -p <final_id>` が成功。初期状態は不合格。
- TUI 課題 (F6, F7) は「ロジックをライブラリ (lib.rs) に、端末 I/O を main.rs に」分け、受け入れテストはロジックを対象にする。
  完成すれば `cargo run -p <final_id>` で実際に TUI として動くこと。
