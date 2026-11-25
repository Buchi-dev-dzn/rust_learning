# 第22章 自作 TUI フレームワーク (セルバッファ・差分描画・レイアウト)

## この章のゴール

- 端末の 1 マスを `Cell`、画面を `Buffer` として表現する
- 旧フレームと新フレームを比べ、変更したセルだけ出力する
- 矩形の分割とクリッピングを行い、Widget に描画領域を渡す
- 入力、描画、端末の開始・終了を分離し、安全に後片付けする

第21章の ratatui が内部で扱っていたものを自分で作ります。目的は完全な互換品ではなく、
セル・レイアウト・差分描画の関係を理解し、小さな TUI を自分で組み立てることです。
端末と関係ない計算を純粋な関数にしておけば、テストは普通の `rustc --test` で走ります。

## 描画の流れ

```text
Model ──view──> 新しい Buffer ──diff(古い Buffer)──> 変更セル
  ▲                                                  │
  │                                                 ANSI
update <── Message <── 入力パーサ <── 端末              ▼
                                                   stdout
```

毎フレーム新しいバッファを空白で初期化し、すべての Widget をそこに描きます。
古いバッファは端末に実際に表示されている内容の記録です。出力と flush が成功してから
古いバッファを新しいものと交換します。途中でエラーになったら表示との差が不明になるため、
次回は全画面の再描画を選ぶのが安全です。

## セルとバッファ

最小のセルは文字とスタイルを持ちます。色だけ変わった場合も差分として扱うため、
`PartialEq` で両方を比較します。

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
struct Cell { ch: char, color: u8 }

struct Buffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Buffer {
    fn new(width: usize, height: usize) -> Self {
        let len = width.checked_mul(height).expect("dimensions overflow");
        Self { width, height, cells: vec![Cell { ch: ' ', color: 0 }; len] }
    }
    fn put(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }
    fn at(&self, x: usize, y: usize) -> Option<&Cell> {
        (x < self.width && y < self.height).then(|| &self.cells[y * self.width + x])
    }
}

fn main() {
    let mut screen = Buffer::new(4, 2);
    screen.put(2, 1, Cell { ch: '#', color: 2 });
    screen.put(100, 0, Cell { ch: '!', color: 0 });
    println!("{:?}", screen.at(2, 1));
}
```

`y * width + x` は行優先の添字です。範囲外の座標を無視する `put` なら Widget が
小さな端末でパニックしにくくなります。ただし巨大な `width * height` はオーバーフローするので
構築時に確認します。実際の端末寸法は小さくても、この境界条件を忘れないでください。

## 文字幅は「文字数」ではない

上の `put` は ASCII の 1 文字 = 1 セルを仮定します。第16章のように全角文字は通常 2 セル、
結合文字は 0 セルになることがあります。完全な実装には grapheme cluster と端末の幅規則が必要です。

- 幅 2 の文字を描くときは次のセルを「継続セル」として予約する
- 右端に半分だけ残る場合は描かないか代替文字を使う
- そのセルを後で上書きするときは、前後の継続セルも消す
- 幅 0 の結合文字は前の表示単位に付ける。単独セルとして進めない

演習 ex01 は ASCII に限定し、範囲チェックとクリッピングに集中します。
日本語を含む完成版では幅処理を無視しないでください。

## Rect とクリッピング

Widget は自分が描いてよい矩形だけ受け取ります。矩形の右端と下端は含みません。
親と子の矩形が重なるかを確認するには、加算のオーバーフローを防いで比較します。

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect { x: usize, y: usize, width: usize, height: usize }

impl Rect {
    fn contains(self, x: usize, y: usize) -> bool {
        x >= self.x && y >= self.y
            && x - self.x < self.width && y - self.y < self.height
    }
}

fn main() {
    let area = Rect { x: 3, y: 2, width: 4, height: 2 };
    assert!(area.contains(6, 3));
    assert!(!area.contains(7, 3));
    println!("right edge excluded");
}
```

`x - self.x` は `x >= self.x` が真のときだけ評価されるので安全です。
座標を比較するとき `x + width` を無条件に計算すると上限であふれます。
描画時は Rect に加え、Buffer の幅・高さでもクリップします。

## 縦方向のレイアウト

タイトルを 1 行、フッターを 1 行にし、残りを本文にする例です。
端末が 1 行だけならタイトルを優先する、など不足時の規則を決めます。

```rust
#[derive(Debug, PartialEq, Eq)]
struct Rows { header: usize, body: usize, footer: usize }

fn split(height: usize) -> Rows {
    let header = height.min(1);
    let footer = (height - header).min(1);
    Rows { header, body: height - header - footer, footer }
}

fn main() {
    assert_eq!(split(1), Rows { header: 1, body: 0, footer: 0 });
    assert_eq!(split(5), Rows { header: 1, body: 3, footer: 1 });
    println!("{:?}", split(5));
}
```

水平方向も同じ考え方です。割合で分割するときは丸めで合計幅がずれないように、
最後のペインに残りを割り当てます。長さ 0 の領域に描く Widget は何もしません。

## Widget をトレイトにする

状態を読み、指定領域内の Buffer だけを書き換える契約を作ります。
端末への I/O を Widget に入れないことが重要です。そうすれば Widget を実際の
ターミナルなしでテストできます。

```rust
struct Buffer { rows: Vec<String> }
struct Rect { y: usize, height: usize }

trait Widget {
    fn render(&self, area: Rect, buffer: &mut Buffer);
}

struct Label<'a> { text: &'a str }
impl Widget for Label<'_> {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        if area.height == 0 { return; }
        if let Some(row) = buffer.rows.get_mut(area.y) {
            *row = self.text.to_owned();
        }
    }
}

fn main() {
    let mut b = Buffer { rows: vec![String::new()] };
    Label { text: "ready" }.render(Rect { y: 0, height: 1 }, &mut b);
    assert_eq!(b.rows[0], "ready");
}
```

この例はトレイトの形を示すための簡略版です。実装では横幅や重なりもクリップします。
アプリ状態の変化は第20章の update に戻し、Widget は表示専用にします。

## 差分を作る

バッファを順番に比較して、変わったセルだけを `(x, y, cell)` で返します。
行優先の添字 `i` から `x = i % width`、`y = i / width` に戻せます。
幅 0 の場合、セルが空であればループに入らず 0 除算しません。

```rust
fn changes(old: &[char], next: &[char], width: usize) -> Vec<(usize, usize, char)> {
    assert_eq!(old.len(), next.len());
    let mut out = Vec::new();
    for (i, (&a, &b)) in old.iter().zip(next).enumerate() {
        if a != b {
            out.push((i % width, i / width, b));
        }
    }
    out
}

fn main() {
    let old = ['a', 'b', 'c', 'd'];
    let next = ['a', 'X', 'c', 'Y'];
    assert_eq!(changes(&old, &next, 2), vec![(1, 0, 'X'), (1, 1, 'Y')]);
    println!("{:?}", changes(&old, &next, 2));
}
```

実際には文字だけでなく色・装飾・継続セルも比較します。端末のリサイズで寸法が変われば
旧セルとの 1 対 1 比較は無効です。全面クリアして新しい画面を描画し、古いバッファを作り直します。
演習 ex02 はこの「寸法の違い」と「色の違い」をテストします。

## 差分を ANSI に変換する

端末への出力は第17章の ANSI を使えます。移動コマンド `ESC [ row ; column H` は 1 始まり。
Buffer の `(x, y)` は 0 始まりなので、それぞれに 1 を足します。

```rust
fn move_to(x: usize, y: usize) -> String {
    format!("\x1b[{};{}H", y + 1, x + 1)
}

fn main() {
    assert_eq!(move_to(3, 1), "\x1b[2;4H");
    println!("{:?}", move_to(3, 1)); // 実端末のカーソルは動かさない
}
```

本物のレンダラでは `queue!` や `BufWriter` を使って出力をまとめ、最後に `flush` します。
連続して変わったセルなら毎セル移動する必要はなく、直前の出力位置を追跡して連続出力できます。
色も直前のスタイルと同じなら SGR を省けます。ただし省略の最適化は正しい描画の後に行います。

## 安全なイベントループ

```text
端末セットアップ (raw / alternate / cursor hide)
loop {
    Model から Buffer を構築
    前フレームとの差分を書き込み、flush
    入力を待つ (キー、リサイズ、タイマー)
    Message に変換して update
    quit なら終了
}
復元 (cursor show / alternate leave / raw disable)
```

入力解析は第18章のエスケープ処理、端末の開始・終了は第19章のガードを再利用します。
stdout への書き込み失敗時にも復元処理を呼ぶため、ガードを開始直後に作ります。
リサイズ時には寸法を取り直し、旧バッファを無効にします。

## よくあるコンパイルエラーと読み方

- `attempt to divide by zero`: 幅 0 のバッファで添字から座標を逆算していないか確認します。
  通常は空セルなのでループに入りません。壊れた寸法のバッファを作らないようにします。
- `index out of bounds`: `put` の境界条件を `<=` ではなく `<` にしているか確認します。
- `cannot borrow ... as mutable more than once`: 旧バッファと新バッファを同時に変更しないでください。
  新バッファを作り、比較してから交換する順番にします。
- `the trait Clone is not implemented`: `vec![cell; n]` は複製が必要です。`Cell` に `Clone` を付けます。

## まとめ

Buffer は「次に表示したいもの」、旧 Buffer は「最後に表示したもの」です。
Widget は矩形にクリップして Buffer へ描き、Renderer は両者の差分を端末に送ります。
入力と後片付けを分離すれば、端末不要のテストと安全な終了を両立できます。

## 次へ

p07 でこの小さな仕組みをアプリとして動かし、卒業課題 F4 / F7 で差分レンダラと
自作フレームワークに挑戦します。Unicode の表示幅、リサイズ、I/O エラーまで含めて確認しましょう。
