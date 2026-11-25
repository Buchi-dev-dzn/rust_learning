# 第8章 モジュールとクレート、Cargo

## この章のゴール

- `mod` でコードを区切り、`pub` で公開範囲を制御できる
- `use` / `self` / `super` / `crate` でパスを書ける
- モジュールをファイルに分ける規則 (`src/foo.rs`, `src/foo/mod.rs`) を知っている
- パッケージ・クレート・バイナリクレート・ライブラリクレートの違いを説明できる
- `cargo new` / `cargo build` / `cargo run` / `cargo test` / `cargo add` と `Cargo.toml` を使える
- ワークスペースで複数のクレートをまとめて管理できる

TUI アプリは「入力」「状態」「描画」「端末制御」などの部品に分かれていきます。
部品の境界を `mod` と `pub` で引いておくと、変更の影響範囲が小さくなり、
ロジックだけをテストすることも簡単になります (Stage 4 で重要になります)。

## mod と pub

モジュールは名前空間です。中のアイテム (関数・構造体・enum・定数・モジュール…) は
**デフォルトで非公開 (private)** で、`pub` を付けたものだけが外から見えます。

```rust
mod geometry {
    #[derive(Debug, Clone, Copy)]
    pub struct Size {
        pub width: u16,
        pub height: u16,
    }

    pub fn area(s: Size) -> u32 {
        helper(s.width) * helper(s.height)
    }

    // pub がないので geometry の外からは呼べない
    fn helper(n: u16) -> u32 {
        n as u32
    }
}

fn main() {
    let s = geometry::Size { width: 80, height: 24 };
    println!("{}", geometry::area(s));
    // geometry::helper(1); // error[E0603]: function `helper` is private
}
```

ルール:

- 親から子の非公開アイテムは見えない。**子から親 (祖先) のアイテムは、非公開でも見える**。
- 構造体を `pub` にしても、**フィールドは個別に `pub` を付けないと非公開** のまま。
- enum を `pub` にすると、**バリアントはすべて公開** される。

### 非公開フィールドで不変条件を守る

フィールドを非公開にし、コンストラクタとメソッドだけを公開すると、
「この値は必ずこの条件を満たす」ことを型の作者が保証できます。

```rust
mod ui {
    #[derive(Debug)]
    pub struct Percent(u8); // フィールドは非公開

    impl Percent {
        /// 0〜100 に収めて作る。外からはこの方法でしか作れない
        pub fn new(v: i32) -> Self {
            Percent(v.clamp(0, 100) as u8)
        }
        pub fn get(&self) -> u8 {
            self.0
        }
    }

    pub fn gauge(p: &Percent, width: usize) -> String {
        let filled = width * p.get() as usize / 100;
        format!("[{}{}] {:>3}%", "#".repeat(filled), "-".repeat(width - filled), p.get())
    }
}

fn main() {
    let p = ui::Percent::new(130); // 100 に丸められる
    println!("{}", ui::gauge(&p, 20));
    println!("{}", ui::gauge(&ui::Percent::new(35), 20));
    // let bad = ui::Percent(250); // error[E0603]: tuple struct constructor `Percent` is private
}
```

第4章の演習で作った `Cursor` も、フィールドを非公開にすれば「カーソルは必ず画面内」を保証できます。

### 公開範囲の細かい指定

| 書き方 | 見える範囲 |
|--------|------------|
| (なし) | 同じモジュールとその子孫 |
| `pub` | どこからでも (親が見えていれば) |
| `pub(crate)` | 同じクレート内だけ |
| `pub(super)` | 親モジュールまで |

アプリ内部だけで使う部品は `pub(crate)` にしておくと、ライブラリとして公開したときに
うっかり外部 API にならずに済みます。

## パスと use

```rust
mod app {
    pub mod input {
        #[derive(Debug, PartialEq)]
        pub enum Key {
            Char(char),
            Esc,
        }

        pub fn parse(b: u8) -> Key {
            if b == 27 { Key::Esc } else { Key::Char(b as char) }
        }
    }

    pub mod state {
        // super = 親モジュール (app)。そこから input へ
        use super::input::Key;

        #[derive(Debug, Default)]
        pub struct State {
            pub text: String,
            pub quit: bool,
        }

        impl State {
            pub fn handle(&mut self, key: Key) {
                match key {
                    Key::Esc => self.quit = true,
                    Key::Char(c) => self.text.push(c),
                }
            }
        }
    }

    // crate = クレートのルート。絶対パス
    pub fn run(bytes: &[u8]) -> crate::app::state::State {
        let mut st = state::State::default(); // 相対パス (自分の子)
        for &b in bytes {
            st.handle(input::parse(b));
            if st.quit {
                break;
            }
        }
        st
    }
}

use app::input::{self, Key}; // input 自体と Key を同時に取り込む

fn main() {
    let st = app::run(b"hi\x1bignored");
    println!("{st:?}");
    println!("{:?}", input::parse(b'x') == Key::Char('x'));
}
```

- `crate::...` はクレートのルートからの **絶対パス**、`super::...` は親から、`self::...` は自分からの相対パス。
- `use` は名前を今のスコープに持ち込むだけです。所有権などとは関係ありません。
- `use a::b::{self, C};` / `use a::b::*;` (グロブ。テストの `use super::*;` でおなじみ) / `use a::B as Other;` (別名)。
- 慣習: 関数は親モジュールまで `use` して `input::parse(..)` と呼び、構造体・enum は型そのものを `use` します。

### pub use で再公開

内部の構造をそのまま見せず、使いやすい位置に名前を出し直せます。

```rust
mod widgets {
    mod block {
        pub struct Block;
    }
    mod list {
        pub struct List;
    }
    // 利用者は widgets::Block と書ける (widgets::block::Block でなくてよい)
    pub use block::Block;
    pub use list::List;
}

fn main() {
    let _b = widgets::Block;
    let _l = widgets::List;
    println!("ratatui::widgets::Block もこの仕組みで公開されている");
}
```

## モジュールをファイルに分ける

ここまでは 1 ファイルの中に `mod name { ... }` と書きました。実際のプロジェクトでは
`mod name;` (セミコロン) と書くと、コンパイラが対応するファイルを読み込みます。

```text
my_app/
├── Cargo.toml
└── src/
    ├── main.rs          // mod app; mod terminal;
    ├── app.rs           // mod app の中身。pub mod input; pub mod state;
    ├── app/
    │   ├── input.rs     // mod app::input の中身
    │   └── state.rs     // mod app::state の中身
    └── terminal.rs      // mod terminal の中身
```

```rust,noplayground
// src/main.rs
mod app;       // src/app.rs (または src/app/mod.rs) を読み込む
mod terminal;  // src/terminal.rs

fn main() {
    let state = app::run(b"hello");
    terminal::draw(&state);
}
```

```rust,noplayground
// src/app.rs
pub mod input; // src/app/input.rs
pub mod state; // src/app/state.rs

pub fn run(bytes: &[u8]) -> state::State {
    // ...
    todo!()
}
```

- `mod foo;` は **宣言** です。各ファイルは、どこかで一度だけ `mod` 宣言されて初めてクレートの一部になります。
  (`src/foo.rs` を作っただけでは使えません。よくあるつまずきです)
- 古いスタイルの `src/app/mod.rs` も使えますが、今は `src/app.rs` + `src/app/` が推奨です。
- ファイルに分けても、`pub` やパスのルールは 1 ファイルで `mod {}` と書いたときと全く同じです。

## パッケージとクレート

- **クレート (crate)**: コンパイルの単位。1 つのルートファイル (`main.rs` や `lib.rs`) から辿れるモジュールの木。
  - **バイナリクレート**: `fn main` を持ち、実行ファイルになる。ルートは `src/main.rs`。
  - **ライブラリクレート**: 他から使われる部品。ルートは `src/lib.rs`。
- **パッケージ (package)**: `Cargo.toml` を持つ単位。ライブラリクレートを最大 1 つ、バイナリクレートをいくつでも含められる。

### lib.rs と main.rs を分ける

1 つのパッケージに `src/lib.rs` と `src/main.rs` の両方を置くと、
main.rs から自分のライブラリを **パッケージ名** で使えます。

```rust,noplayground
// src/lib.rs (パッケージ名が my_editor の場合)
pub mod buffer;
pub mod keys;

// src/main.rs
use my_editor::buffer::Buffer; // 外部クレートと同じように使う

fn main() {
    let buf = Buffer::new();
    // 端末の raw モード、描画ループなど I/O はここに
}
```

この形は Stage 4 / 5 の TUI プロジェクトで必ず使います。
**ロジック (lib.rs) は端末なしでテストでき、端末 I/O (main.rs) は薄く保てる** からです。

## Cargo

Cargo は Rust のビルドツール兼パッケージマネージャです。

```text
$ cargo new hello_tui        # バイナリパッケージを作る (src/main.rs)
$ cargo new --lib tui_core   # ライブラリパッケージを作る (src/lib.rs)
$ cd hello_tui
$ cargo build                # ビルド (target/debug/hello_tui)
$ cargo run                  # ビルドして実行
$ cargo run -- --size 80x24  # -- の後ろはプログラムへの引数
$ cargo test                 # テストを実行
$ cargo check                # 型チェックだけ (速い。書いている最中に便利)
$ cargo build --release      # 最適化ビルド (target/release/)
$ cargo fmt                  # 整形
$ cargo clippy               # リント (より良い書き方を提案してくれる)
$ cargo add crossterm        # 依存を追加 (Cargo.toml に書き込む)
$ cargo doc --open           # 依存も含めたドキュメントを生成して開く
```

### Cargo.toml

```toml
[package]
name = "hello_tui"
version = "0.1.0"
edition = "2024"

[dependencies]
crossterm = "0.28"                      # crates.io から
ratatui = { version = "0.29", default-features = false, features = ["crossterm"] }
my_lib = { path = "../my_lib" }         # ローカルのクレート

[dev-dependencies]
# テストでだけ使う依存

[profile.release]
opt-level = 3
```

- `edition`: 言語のエディション。このコースは `2024`。
- バージョン `"0.28"` は「0.28.x の互換範囲」の意味 (SemVer)。実際に使ったバージョンは `Cargo.lock` に記録されます。
  アプリでは `Cargo.lock` をリポジトリにコミットし、全員が同じバージョンでビルドします。
- 依存クレートは、コードからは `use crossterm::...` のようにクレート名で参照します (`-` は `_` に置き換え)。

### ワークスペース

複数のパッケージを 1 つのリポジトリでまとめて管理する仕組みです。
`target/` ディレクトリと `Cargo.lock` が共有され、依存のバージョンを揃えられます。
このコースのリポジトリもワークスペースになっています。

```toml
# ルートの Cargo.toml
[workspace]
resolver = "3"
members = ["projects/*", "finals/*"]

[workspace.package]
edition = "2024"
version = "0.1.0"
publish = false

[workspace.dependencies]
crossterm = "0.28"
ratatui = "0.29"
```

```toml
# projects/p05_editor/Cargo.toml (メンバー側)
[package]
name = "p05_editor"
edition.workspace = true
version.workspace = true
publish.workspace = true

[dependencies]
crossterm = { workspace = true }
```

```text
$ cargo build -p p05_editor     # 特定のパッケージだけビルド
$ cargo test -p p05_editor
$ cargo run -p p05_editor -- README.md
$ cargo test --workspace        # 全部
```

## テストとモジュール

演習ファイルの末尾にある `#[cfg(test)] mod tests { use super::*; ... }` も、ただのモジュールです。

- `#[cfg(test)]` — `cargo test` (`rustc --test`) のときだけコンパイルする。
- `use super::*;` — 親 (テスト対象のモジュール) のアイテムを全部取り込む。
  子モジュールなので、親の **非公開** 関数もテストできます。

```rust
mod keys {
    pub fn is_quit(b: u8) -> bool {
        b == b'q' || is_ctrl_c(b)
    }
    fn is_ctrl_c(b: u8) -> bool {
        b == 3
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn ctrl_c_is_quit() {
            assert!(is_ctrl_c(3)); // 非公開関数もテストできる
            assert!(is_quit(b'q'));
        }
    }
}

fn main() {
    println!("{}", keys::is_quit(3));
}
```

統合テスト (`tests/` ディレクトリ) は別クレートとしてコンパイルされるので、`pub` なものしか使えません (第13章)。

## よくあるコンパイルエラーと読み方

| コード | メッセージ | 直し方 |
|--------|------------|--------|
| E0603 | `function xxx is private` | `pub` を付ける (必要最小限に) |
| E0616 | `field xxx of struct Y is private` | フィールドに `pub`、または getter メソッドを作る |
| E0451 | `field xxx is private` (構造体リテラル) | `pub` フィールドにするか、`new` を用意する |
| E0433 | `failed to resolve: use of undeclared crate or module` | パスの綴り / `mod` 宣言 / `use` / Cargo.toml の依存 |
| E0432 | `unresolved import` | `use` のパスが違う。`crate::` / `super::` から書き直す |
| E0583 | `file not found for module` | `mod foo;` に対応する `src/foo.rs` がない |

E0603 のメッセージには、どこで定義された非公開アイテムかが `note: the function xxx is defined here` として示されます。
「本当に外に見せるべきか?」を考えてから `pub` を付けましょう。

## まとめ

- `mod` で名前空間を作る。中身は既定で非公開、`pub` で公開。構造体のフィールドは個別に `pub`。
- パスは `crate::` (絶対) / `super::` (親) / `self::` (自分)。`use` で短く書ける。`pub use` で再公開。
- `mod foo;` で `src/foo.rs` を読み込む。宣言しないとファイルはクレートに含まれない。
- パッケージ = Cargo.toml、クレート = コンパイル単位。ロジックは `lib.rs`、I/O は `main.rs` に。
- Cargo: `new` / `build` / `run` / `test` / `check` / `add`。ワークスペースで複数パッケージを管理。

## 次へ

これで Stage 1 は終わりです。プロジェクト `p01_minigrep` と `p02_todo_cli` で、ここまでの知識を使って
小さな CLI を実際に作ってみましょう。Stage 2 ではトレイトとジェネリクスで、再利用できる抽象を作ります。
