# 第13章 テストの書き方

## この章のゴール

- `#[test]` と `assert!` / `assert_eq!` / `assert_ne!` で単体テストを書ける
- `#[should_panic]` と `Result` を返すテストで、異常系をテストできる
- `#[cfg(test)] mod tests` / `tests/` ディレクトリ / ドキュメントテストの使い分けを説明できる
- `cargo test` の絞り込み・出力表示のオプションを使える
- 「端末に依存する部分」と「純粋なロジック」を分けて、TUI アプリをテスト可能に設計できる

これまでの演習はすべて、あらかじめ書かれたテストを通す形式でした。
この章では **自分でテストを書く側** に回ります。
TUI アプリは画面を目で確認しがちですが、キー解析・バッファ操作・レイアウト計算などのロジックは
すべて普通の関数としてテストできます。後半のプロジェクトや卒業課題でも、この考え方が前提になります。

## 最小のテスト

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]           // cargo test (rustc --test) のときだけコンパイルされる
mod tests {
    use super::*;      // 親モジュール (テスト対象) の要素を使う

    #[test]            // これがテスト関数の印
    fn adds_two_numbers() {
        assert_eq!(add(2, 3), 5);
    }
}

fn main() {
    // 通常の実行ではテストは含まれない
    println!("{}", add(1, 1));
}
```

- テスト関数は引数なし・戻り値なし (または `Result`)。**panic したら失敗**、しなければ成功
- `cargo test` は `#[test]` の付いた関数を全部集めて並列に実行する
- 単体ファイルなら `rustc --test file.rs && ./file` でも同じことができる (演習の採点はこの方式)

実行結果の例:

```text
running 1 test
test tests::adds_two_numbers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## アサーションマクロ

| マクロ | 意味 |
|---|---|
| `assert!(cond)` | `cond` が `true` であること |
| `assert_eq!(left, right)` | 等しいこと (`PartialEq` と `Debug` が必要) |
| `assert_ne!(left, right)` | 等しくないこと |
| `debug_assert!(...)` | デバッグビルドのときだけ検査 (本体コード用) |

どれも最後に `format!` 形式のメッセージを付けられます。失敗時の調査が楽になります。

```rust
fn clamp_cursor(cursor: usize, len: usize) -> usize {
    if len == 0 { 0 } else { cursor.min(len - 1) }
}

fn main() {
    for (cursor, len) in [(0, 0), (5, 3), (1, 3)] {
        let got = clamp_cursor(cursor, len);
        assert!(got <= len, "cursor={cursor} len={len} got={got}");
    }
    assert_eq!(clamp_cursor(5, 3), 2, "末尾で止まるべき");
    assert_ne!(clamp_cursor(1, 3), 0);
    println!("all assertions passed");
}
```

`assert_eq!` が失敗すると、左右の値が `Debug` 表示されます。

```text
thread 'tests::clamps' panicked at src/lib.rs:12:9:
assertion `left == right` failed: 末尾で止まるべき
  left: 3
 right: 2
```

`left` が実際の値、`right` が期待値になるように `assert_eq!(actual, expected)` の順で書く習慣を付けると読みやすくなります。

## 異常系のテスト

### `#[should_panic]`

「この入力ではパニックすべき」をテストします。`expected` を付けると、
パニックメッセージにその文字列が **含まれる** ことも検査します。

```rust
pub struct Rect {
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(width: u16, height: u16) -> Rect {
        assert!(width > 0 && height > 0, "Rect のサイズは 1 以上: {width}x{height}");
        Rect { width, height }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "サイズは 1 以上")]
    fn zero_width_panics() {
        Rect::new(0, 10);
    }
}

fn main() {
    let r = Rect::new(80, 24);
    println!("{}x{}", r.width, r.height);
}
```

`expected` なしの `#[should_panic]` は「どんな理由でもパニックすれば成功」なので、
想定外のバグ (配列の範囲外アクセスなど) でも通ってしまいます。なるべく `expected` を付けましょう。

### `Result` を返すテスト

テスト関数が `Result<(), E>` を返すと、`?` 演算子が使えます。`Err` が返ればテスト失敗です。

```rust
fn parse_size(s: &str) -> Result<(u16, u16), String> {
    let (w, h) = s.split_once('x').ok_or(format!("'x' がない: {s}"))?;
    let w = w.parse().map_err(|e| format!("幅が不正: {e}"))?;
    let h = h.parse().map_err(|e| format!("高さが不正: {e}"))?;
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses() -> Result<(), String> {
        let (w, h) = parse_size("80x24")?; // Err ならここでテスト失敗
        assert_eq!((w, h), (80, 24));
        Ok(())
    }

    #[test]
    fn rejects_garbage() {
        // Err になること自体をテストするなら is_err() や matches! を使う
        assert!(parse_size("80*24").is_err());
        assert!(matches!(parse_size("ax24"), Err(e) if e.contains("幅")));
    }
}

fn main() {
    println!("{:?}", parse_size("120x40"));
    println!("{:?}", parse_size("oops"));
}
```

## テストの整理

### 単体テスト: 同じファイルの `mod tests`

- 慣習として、ファイル末尾に `#[cfg(test)] mod tests { use super::*; ... }` を置く
- 子モジュールなので、**非公開 (pub でない) 関数もテストできる**
- テスト用の補助関数 (サンプルデータを作る関数など) も `mod tests` の中に書ける (`#[test]` を付けなければテストとしては実行されない)

### 結合テスト: `tests/` ディレクトリ

Cargo プロジェクトでは、`src/` と並ぶ `tests/` に置いたファイルが **結合テスト** になります。

```text
my_app/
├── Cargo.toml
├── src/
│   ├── lib.rs        ← ロジック (pub な API)
│   └── main.rs       ← 端末 I/O だけ
└── tests/
    └── acceptance.rs ← 外部の利用者の立場で lib の pub API をテスト
```

```rust,noplayground
// tests/acceptance.rs
use my_app::Editor; // クレート名で lib.rs の pub 要素を使う

#[test]
fn insert_and_undo() {
    let mut ed = Editor::new();
    ed.insert('a');
    ed.undo();
    assert_eq!(ed.text(), "");
}
```

- 各ファイルが独立したクレートとしてコンパイルされる (`#[cfg(test)]` は不要)
- `pub` な API しか使えない。「外から見た振る舞い」を確かめるのに向いている
- `main.rs` だけのバイナリクレートは結合テストから使えない。**ロジックを `lib.rs` に置く** 理由の 1 つ

卒業課題 (Stage 5) の受け入れテストは、まさにこの `tests/acceptance.rs` の形式です。

### ドキュメントテスト

`///` のドキュメントコメントに書いたコード例は、`cargo test` で **実際に実行されます**。
ドキュメントが古くなって嘘をつくのを防げます。

````rust,noplayground
/// 文字列を幅 `width` に切り詰める。
///
/// ```
/// assert_eq!(my_app::truncate("hello", 3), "hel");
/// ```
pub fn truncate(s: &str, width: usize) -> String {
    s.chars().take(width).collect()
}
````

ドキュメントテストはライブラリクレート (`lib.rs`) でだけ動きます (`rustdoc` が実行するため、
単体ファイルを `rustc --test` するだけでは実行されません)。

## `cargo test` の使い方

| コマンド | 意味 |
|---|---|
| `cargo test` | 全テスト (単体・結合・ドキュメント) |
| `cargo test wrap` | 名前に `wrap` を含むテストだけ |
| `cargo test -p p05_editor` | ワークスペースの特定パッケージだけ |
| `cargo test --test acceptance` | `tests/acceptance.rs` だけ |
| `cargo test -- --nocapture` | 成功したテストの `println!` も表示 |
| `cargo test -- --test-threads=1` | 並列にせず 1 つずつ実行 |
| `cargo test -- --ignored` | `#[ignore]` を付けた (遅い) テストだけ |

`--` より後ろはテスト実行ファイルへのオプションです。
テストは **並列** に走るので、同じファイルや環境変数を触るテスト同士は干渉することがあります。

## TUI をテスト可能にする設計

端末 (raw モード・画面描画・キー読み取り) は自動テストしにくい部分です。
そこで次のように分けます。

1. **状態** (`App` 構造体) と **更新** (`fn update(&mut self, event)`) は純粋なロジック
2. **描画** は「状態 → バッファ (文字の 2 次元配列)」の関数にする
3. 端末への書き出しとキー読み取りだけを `main.rs` の薄い層に置く

```rust
#[derive(Debug, PartialEq)]
enum Key {
    Up,
    Down,
    Char(char),
}

struct App {
    items: Vec<&'static str>,
    selected: usize,
    quit: bool,
}

impl App {
    fn update(&mut self, key: Key) {
        match key {
            Key::Up => self.selected = self.selected.saturating_sub(1),
            Key::Down => self.selected = (self.selected + 1).min(self.items.len() - 1),
            Key::Char('q') => self.quit = true,
            Key::Char(_) => {}
        }
    }

    // 端末に書く代わりに「画面の各行」を返す → 文字列比較でテストできる
    fn render(&self) -> Vec<String> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{} {}", if i == self.selected { ">" } else { " " }, s))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_selection_and_renders() {
        let mut app = App { items: vec!["a", "b"], selected: 0, quit: false };
        app.update(Key::Down);
        app.update(Key::Down); // 末尾で止まる
        assert_eq!(app.render(), vec!["  a", "> b"]);
    }
}

fn main() {
    let mut app = App { items: vec!["open", "save", "quit"], selected: 0, quit: false };
    for key in [Key::Down, Key::Down, Key::Up, Key::Char('q')] {
        app.update(key);
    }
    println!("{}", app.render().join("\n"));
    println!("quit = {}", app.quit);
}
```

「キー列を流し込んで、最終状態と描画結果を比べる」テストは、
エディタでもファイルマネージャでも同じ形で書けます。第20章のアーキテクチャはこの考え方の延長です。

### 良いテストのコツ

- 1 つのテストでは 1 つの振る舞いを確かめる。名前で何を確かめるか分かるようにする (`cursor_stops_at_end`)
- 境界値を狙う: 空、1 要素、ちょうど幅いっぱい、幅 + 1、0、最大値、日本語 (マルチバイト)
- バグを見つけたら、**まずそれを再現するテストを書いてから** 直す (回帰テスト)
- テストが通ることを確かめたら、一度わざとコードを壊して **失敗することも** 確かめる

## よくあるコンパイルエラーと読み方

### E0433 / E0425: テスト対象が見つからない

```text
error[E0425]: cannot find function `add` in this scope
help: consider importing this function: use super::add;
```

`mod tests` は別モジュールなので、`use super::*;` を忘れると親の関数が見えません。

### `#[should_panic]` のテストが失敗する

```text
note: panic did not contain expected string
      panic message: "index out of bounds: the len is 3 but the index is 5"
 expected substring: "サイズは 1 以上"
```

パニックはしたが、想定と違う理由でした。テスト対象のバグか、`expected` の文字列違いです。

### テスト関数に引数を付けた / 戻り値の型が違う

```text
error: functions used as tests can not have any arguments
error[E0277]: the trait bound `i32: Termination` is not satisfied
```

テスト関数は引数なし、戻り値は `()` か `Result<(), E>` (`E: Debug`) のみです。

## まとめ

- `#[cfg(test)] mod tests` + `#[test]` + `assert_eq!(actual, expected)` が基本形
- 異常系は `#[should_panic(expected = "...")]` か、`Result` を返すテストで
- 単体テストは非公開関数も、結合テスト (`tests/`) は pub API を、ドキュメントテストは例を検証する
- TUI は「状態更新」と「バッファへの描画」を純粋関数にしてテストする

## 次へ

第14章では **並行処理** を学びます。キー入力を待つスレッドと、一定間隔で tick を送るスレッドを
チャネルでつなぐ — TUI のイベントループの骨格を作ります。
