# 第12章 スマートポインタ (Box, Rc, RefCell)

## この章のゴール

- `Box<T>` で値をヒープに置き、再帰的なデータ構造 (木) を作れる
- `Rc<T>` で 1 つの値を複数の所有者で共有できる
- `RefCell<T>` で「共有しつつ変更する」(内部可変性) ができる
- `Rc<RefCell<T>>` と `Weak<T>` で、親子が互いを参照する UI ツリーを作れる

TUI の画面はたいてい **木構造** です。
「ルート → 縦分割 → (サイドバー, 横分割 → (エディタ, ステータスバー))」のように、
コンテナが子を持ち、子がさらに子を持ちます。
所有権が「1 つの値に所有者は 1 人」のルールを持つ Rust で、こうした構造をどう表すかを学びます。

## スマートポインタとは

「ポインタのように振る舞うが、追加の機能を持つ構造体」です。
すでに使っている `String` や `Vec<T>` もスマートポインタの一種です (ヒープのバッファを所有し、破棄時に解放する)。
共通する 2 つのトレイト:

- `Deref`: `*p` や `p.method()` で中身にアクセスできる
- `Drop`: スコープを抜けたときの後始末 (メモリ解放など)

## `Box<T>`: ヒープに置く

```rust
fn main() {
    let b = Box::new(42);       // 42 はヒープ上に置かれる
    println!("{}", *b + 1);     // Deref で中身にアクセス
    println!("{}", b);          // Display も中身のものが使える
} // b がスコープを抜けるとヒープも解放される
```

単に値をヒープに置くだけなら利点は少ないですが、次の 2 つの場面で必須です。

1. **再帰的な型** (サイズがコンパイル時に決まらない)
2. **トレイトオブジェクト** `Box<dyn Widget>` (第9章)

### 再帰的な型: レイアウト木

```rust
enum Layout {
    Leaf(&'static str),
    // Layout の中に Layout を直接入れるとサイズが無限になる → Box で間接参照にする
    HSplit(Box<Layout>, Box<Layout>),
    VSplit(Box<Layout>, Box<Layout>),
}

use Layout::*;

impl Layout {
    fn count_leaves(&self) -> usize {
        match self {
            Leaf(_) => 1,
            HSplit(a, b) | VSplit(a, b) => a.count_leaves() + b.count_leaves(),
        }
    }

    fn describe(&self, depth: usize, out: &mut Vec<String>) {
        let indent = "  ".repeat(depth);
        match self {
            Leaf(name) => out.push(format!("{indent}{name}")),
            HSplit(a, b) | VSplit(a, b) => {
                let kind = if matches!(self, HSplit(..)) { "HSplit" } else { "VSplit" };
                out.push(format!("{indent}{kind}"));
                a.describe(depth + 1, out);
                b.describe(depth + 1, out);
            }
        }
    }
}

fn main() {
    let ui = VSplit(
        Box::new(Leaf("sidebar")),
        Box::new(HSplit(Box::new(Leaf("editor")), Box::new(Leaf("status")))),
    );
    println!("leaves = {}", ui.count_leaves());
    let mut out = Vec::new();
    ui.describe(0, &mut out);
    println!("{}", out.join("\n"));
}
```

`Box` を外すと次のエラーになります。

```text
error[E0072]: recursive type `Layout` has infinite size
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
```

`Box<Layout>` はポインタ 1 個分のサイズなので、`Layout` 全体のサイズが決まります。

## `Rc<T>`: 共有所有

`Box` は所有者が 1 人です。**複数の場所から同じ値を所有したい** ときは `Rc` (Reference Counted) を使います。

```rust
use std::rc::Rc;

struct Theme {
    name: String,
    accent: &'static str,
}

struct Panel {
    title: String,
    theme: Rc<Theme>, // 複数のパネルが同じテーマを共有
}

fn main() {
    let theme = Rc::new(Theme { name: "dark".into(), accent: "cyan" });
    println!("count = {}", Rc::strong_count(&theme)); // 1

    let left = Panel { title: "Files".into(), theme: Rc::clone(&theme) };
    let right = Panel { title: "Preview".into(), theme: Rc::clone(&theme) };
    println!("count = {}", Rc::strong_count(&theme)); // 3

    for p in [&left, &right] {
        println!("{} uses {} ({})", p.title, p.theme.name, p.theme.accent);
    }

    drop(left);
    println!("count = {}", Rc::strong_count(&theme)); // 2
} // 最後の Rc が消えたときに Theme も解放される
```

- `Rc::clone(&rc)` は中身をコピーせず、**参照カウントを 1 増やすだけ** (安い)
- カウントが 0 になったときに中身が `drop` される
- `Rc<T>` の中身は **読み取り専用** (`&T` しか得られない)
- `Rc` はシングルスレッド専用。スレッド間で共有するなら第14章の `Arc` を使う

## `RefCell<T>`: 内部可変性

`Rc` で共有した値を変更したいことがあります (例: 共有のカウンタ、共有のログ)。
しかし借用ルール「`&mut` は同時に 1 つだけ」をコンパイル時に証明できない場面があります。
`RefCell` は借用チェックを **実行時** に行うことで、`&self` からでも変更を可能にします。

```rust
use std::cell::RefCell;

fn main() {
    let log = RefCell::new(Vec::<String>::new());

    // borrow_mut() で可変参照 (RefMut) を得る。log 自体は mut でなくてよい
    log.borrow_mut().push("started".into());
    log.borrow_mut().push("key: j".into());

    // borrow() で不変参照 (Ref) を得る
    println!("{} entries", log.borrow().len());
    for line in log.borrow().iter() {
        println!("- {line}");
    }
}
```

ルールは通常の借用と同じです。違反するとコンパイルエラーではなく **panic** します。

```rust
use std::cell::RefCell;

fn main() {
    let cell = RefCell::new(5);
    let r1 = cell.borrow();
    // let w = cell.borrow_mut(); // panic: already borrowed (r1 がまだ生きている)
    println!("{}", *r1);
    drop(r1);
    *cell.borrow_mut() += 1; // r1 を drop した後なら OK
    println!("{}", cell.borrow());

    // try_borrow_mut はパニックせず Result を返す
    let r2 = cell.borrow();
    println!("{}", cell.try_borrow_mut().is_err()); // true
    drop(r2);
}
```

**`borrow()` / `borrow_mut()` の戻り値をなるべく短く保持する** のがコツです。
変数に入れっぱなしにせず、1 文の中で使い切るのが安全です。

## `Rc<RefCell<T>>`: 共有して変更する

組み合わせると「複数の所有者が、同じ値を変更できる」ようになります。
UI ツリーのノードの典型的な形です。

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    name: String,
    children: Vec<Rc<RefCell<Node>>>,
}

fn node(name: &str) -> Rc<RefCell<Node>> {
    Rc::new(RefCell::new(Node { name: name.to_string(), children: vec![] }))
}

fn count(n: &Rc<RefCell<Node>>) -> usize {
    1 + n.borrow().children.iter().map(count).sum::<usize>()
}

fn main() {
    let root = node("root");
    let list = node("list");
    root.borrow_mut().children.push(Rc::clone(&list));

    // list への参照を別に持っておき、あとから子を追加できる
    for i in 0..3 {
        list.borrow_mut().children.push(node(&format!("item{i}")));
    }
    list.borrow_mut().name = "file list".into();

    println!("nodes = {}", count(&root));
    println!("first child = {}", root.borrow().children[0].borrow().name);
}
```

## `Weak<T>`: 循環参照を避ける

子から親をたどりたい (例: フォーカス中のノードから親コンテナを探す) とき、
子が親を `Rc` で持つと **循環参照** になり、カウントが 0 にならずメモリリークします。
親への参照は `Weak` (カウントを増やさない弱い参照) にします。

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Node {
    name: String,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn new_node(name: &str) -> Rc<Node> {
    Rc::new(Node {
        name: name.into(),
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    })
}

fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent); // Rc → Weak
    parent.children.borrow_mut().push(child);
}

fn path(node: &Rc<Node>) -> String {
    let mut names = vec![node.name.clone()];
    let mut cur = node.parent.borrow().upgrade(); // Weak → Option<Rc>
    while let Some(p) = cur {
        names.push(p.name.clone());
        cur = p.parent.borrow().upgrade();
    }
    names.reverse();
    names.join("/")
}

fn main() {
    let root = new_node("root");
    let panel = new_node("panel");
    let button = new_node("button");
    add_child(&root, Rc::clone(&panel));
    add_child(&panel, Rc::clone(&button));

    println!("{}", path(&button));
    println!("panel strong = {}, weak = {}", Rc::strong_count(&panel), Rc::weak_count(&panel));
    println!("root has {} child", root.children.borrow().len());
}
```

- `Rc::downgrade(&rc)` で `Weak` を作る
- `weak.upgrade()` は `Option<Rc<T>>`。相手がすでに解放されていれば `None`
- 親 → 子は `Rc` (所有)、子 → 親は `Weak` (参照のみ) が定石

### 使い分けの目安

| 型 | 所有者 | 変更 | スレッド | 用途 |
|---|---|---|---|---|
| `Box<T>` | 1 | 所有者が `mut` なら可 | 可 | 再帰型・`dyn Trait` |
| `Rc<T>` | 複数 | 不可 | 不可 | 読み取り専用の共有 |
| `RefCell<T>` | 1 | `&self` から可 (実行時チェック) | 不可 | 内部可変性 |
| `Rc<RefCell<T>>` | 複数 | 可 | 不可 | 共有 + 変更 (UI ツリー) |
| `Arc<Mutex<T>>` | 複数 | 可 | 可 | 第14章 |

実務の TUI (ratatui など) では、ウィジェットを毎フレーム作り直して木を持たない設計
(**イミディエイトモード**) が主流で、`Rc<RefCell>` はあまり使いません。
しかし自作フレームワークでフォーカス管理や親子関係を持つ場合 (リテインドモード) には必要になります。
「状態は App 構造体に集め、描画は毎回作る」設計を第20章で学ぶので、比較してみてください。

## よくあるコンパイルエラーと読み方

### E0072: recursive type has infinite size

再帰的な型を `Box` なしで定義しています。`help` の通り `Box<T>` で包みます。

### E0594 / E0596: cannot assign / borrow data in an `Rc` as mutable

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
```

`Rc<T>` の中身は変更できません。変更したいなら `Rc<RefCell<T>>` にして `borrow_mut()` を使います。

### 実行時: already borrowed: BorrowMutError

```text
thread 'main' panicked at src/main.rs:5:10:
already borrowed: BorrowMutError
```

`RefCell` の借用ルール違反です。コンパイルは通るので気付きにくい点に注意。
`let r = x.borrow();` を持ったまま同じ値を `borrow_mut()` していないか確認します。
特に `for child in node.borrow().children.iter() { node.borrow_mut()... }` のような
「読みながら書く」パターンで起きがちです。

### E0277: `Rc<...>` cannot be sent between threads safely

`Rc` をスレッドに渡そうとしています。`Arc` (と `Mutex`) を使います (第14章)。

## まとめ

- `Box<T>`: ヒープに置く。再帰型とトレイトオブジェクトに必須
- `Rc<T>`: 参照カウントで共有所有。`Rc::clone` は安い
- `RefCell<T>`: 借用チェックを実行時に。違反は panic
- `Rc<RefCell<T>>` で共有 + 変更、親への参照は `Weak` で循環を防ぐ

## 次へ

第13章では **テストの書き方** を学びます。ここまでの演習で使ってきた `#[test]` を、
自分で設計して書けるようになりましょう。
