# 第16章 文字列と Unicode の表示幅

## この章のゴール

- Rust の `String` / `&str` が **UTF-8 のバイト列** であることを理解する
- 「バイト数」「`char` の数」「書記素クラスタ (見た目の 1 文字)」「表示幅 (列数)」の 4 つを区別できる
- 全角文字が端末で 2 列を占める理由 (East Asian Width) を知る
- CJK の範囲から簡易な `char_width` 関数を実装できる
- 文字列を指定の列幅で切り詰めたり、空白で埋めたりできる

TUI では「画面の 10 列目にカーソルを置く」「幅 20 の枠に文字列を収める」といった処理を
いつも行います。このとき `s.len()` を使うと、日本語を含む文字列で表示が必ず崩れます。
この章はそれを防ぐための章です。

## 4 つの「長さ」

```rust
fn main() {
    let s = "Rust言語";
    println!("バイト数   : {}", s.len());            // 4 + 3*2 = 10
    println!("char の数  : {}", s.chars().count());  // 6
    // 表示幅: R u s t = 1 列ずつ、言 語 = 2 列ずつ → 8 列
    for c in s.chars() {
        println!("{c:?} U+{:04X} UTF-8 {} バイト", c as u32, c.len_utf8());
    }
}
```

| 長さ | 意味 | 求め方 | 用途 |
|------|------|--------|------|
| バイト数 | UTF-8 で何バイトか | `s.len()` | スライス `&s[a..b]`、ファイルサイズ |
| char 数 | Unicode スカラー値の数 | `s.chars().count()` | 文字単位の処理 |
| 書記素クラスタ数 | 人間が「1 文字」と感じる単位 | 標準にはない (`unicode-segmentation` クレート) | カーソル移動・削除 |
| 表示幅 | 端末で何列を占めるか | 自作 or `unicode-width` クレート | TUI のレイアウト |

## UTF-8 のしくみ

UTF-8 は 1 つのコードポイントを 1〜4 バイトで表します。

| コードポイント | バイト数 | 先頭バイトのパターン | 例 |
|---|---|---|---|
| U+0000〜U+007F | 1 | `0xxxxxxx` | `A` = `41` |
| U+0080〜U+07FF | 2 | `110xxxxx 10xxxxxx` | `é` = `C3 A9` |
| U+0800〜U+FFFF | 3 | `1110xxxx 10xxxxxx 10xxxxxx` | `あ` = `E3 81 82` |
| U+10000〜 | 4 | `11110xxx 10xxxxxx ...` | `🦀` = `F0 9F A6 80` |

続きのバイトは必ず `10xxxxxx` なので、どのバイトが文字の先頭かが判別できます。
ASCII は 1 バイトのままなので、ANSI エスケープシーケンス (第17章) は UTF-8 の中でも安全に扱えます。

```rust
fn main() {
    for s in ["A", "é", "あ", "🦀"] {
        let bytes: Vec<String> = s.bytes().map(|b| format!("{b:02X}")).collect();
        println!("{s} -> {}", bytes.join(" "));
    }
}
```

### バイト境界に注意

`&s[a..b]` はバイト位置で切り出します。文字の途中で切るとパニックします。

```rust
fn main() {
    let s = "あいう";
    println!("{}", &s[0..3]); // "あ" (OK: 3 バイト目は文字境界)
    println!("is_char_boundary(1) = {}", s.is_char_boundary(1)); // false
    // &s[0..1] はパニック: byte index 1 is not a char boundary

    // 安全に「n 文字目」を扱うには char_indices を使う
    for (byte_pos, c) in s.char_indices() {
        println!("byte {byte_pos}: {c}");
    }
}
```

`char_indices()` は「バイト位置と char」のペアを返します。エディタでカーソルの
char 位置からバイト位置を求めるときの基本道具です。

## 書記素クラスタ

`char` は Unicode の「スカラー値」であり、人間の感じる「1 文字」とは限りません。

```rust
fn main() {
    let composed = "e\u{301}"; // e + 結合アクセント記号 → "é" と表示される
    let family = "👨\u{200D}👩\u{200D}👧"; // ZWJ でつながった家族の絵文字
    let flag = "🇯🇵"; // 地域指示記号 2 つで国旗
    for s in [composed, family, flag] {
        println!("{s}: bytes={} chars={}", s.len(), s.chars().count());
    }
}
```

見た目は 1 文字でも、`chars()` では 2〜5 個になります。エディタで「1 文字削除」したときに
アクセント記号だけが残る、といったバグの原因です。

標準ライブラリには書記素クラスタの分割機能がないため、本格的には `unicode-segmentation`
クレートの `graphemes(true)` を使います。このコースでは「結合文字 (幅 0) は直前の文字にくっつける」
という簡易ルールで十分とします。

## 表示幅と East Asian Width

端末は文字を **セル (マス目)** に並べます。ASCII は 1 セル、漢字やひらがなは 2 セルです。
この「何セル使うか」は Unicode の **East Asian Width** という属性で決まっています。

| 属性 | 意味 | 幅 |
|---|---|---|
| F (Fullwidth) | 全角英数 `Ａ` など | 2 |
| W (Wide) | 漢字・かな・ハングル・多くの絵文字 | 2 |
| H (Halfwidth) | 半角カナ `ｱ` | 1 |
| Na (Narrow) | ASCII | 1 |
| A (Ambiguous) | `○` `※` ギリシャ文字など | 端末設定次第 (普通は 1) |
| N (Neutral) | その他 | 1 |

これに加えて、結合文字やゼロ幅文字 (ZWJ など) は幅 0、制御文字は表示されません。

`unicode-width` クレートは Unicode のデータ表から正確な幅を返してくれますが、
仕組みを知るために、主要な範囲だけを使って自分で書いてみましょう。

```rust
/// 簡易版の表示幅。実用上よく出る範囲だけを扱う。
fn char_width(c: char) -> usize {
    let cp = c as u32;
    // 制御文字
    if cp < 0x20 || (0x7F..0xA0).contains(&cp) {
        return 0;
    }
    // 結合文字・ゼロ幅
    if (0x0300..=0x036F).contains(&cp) || (0x200B..=0x200F).contains(&cp) || cp == 0xFE0F {
        return 0;
    }
    let wide = matches!(cp,
        0x1100..=0x115F     // ハングル字母
        | 0x2E80..=0x303E   // CJK 部首・記号、句読点 (、。「」)
        | 0x3041..=0x33FF   // ひらがな・カタカナ・CJK 互換
        | 0x3400..=0x4DBF   // CJK 拡張 A
        | 0x4E00..=0x9FFF   // CJK 統合漢字
        | 0xA000..=0xA4CF   // イ文字
        | 0xAC00..=0xD7A3   // ハングル音節
        | 0xF900..=0xFAFF   // CJK 互換漢字
        | 0xFE30..=0xFE4F   // CJK 互換形
        | 0xFF00..=0xFF60   // 全角英数・記号
        | 0xFFE0..=0xFFE6   // 全角記号 (￥ など)
        | 0x1F300..=0x1F64F // 絵文字
        | 0x1F900..=0x1F9FF // 絵文字 (補助)
        | 0x20000..=0x3FFFD // CJK 拡張 B 以降
    );
    if wide { 2 } else { 1 }
}

fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

fn main() {
    for s in ["hello", "こんにちは", "Ｒｕｓｔ", "ｱｲｳ", "🦀 crab", "e\u{301}"] {
        println!("{s:<12} len={:2} chars={:2} width={:2}", s.len(), s.chars().count(), str_width(s));
    }
}
```

出力を見ると、`{:<12}` による左寄せが日本語の行だけずれているはずです。`format!` の幅指定は
**char の数** で数えるため、表示幅を考慮しません。TUI では自分で幅を計算して埋める必要があります。

> 半角カナ (U+FF61〜U+FF9F) は 1 列です。上の範囲で `0xFF00..=0xFF60` までにしているのはそのためです。

## 幅に合わせて埋める・切り詰める

表やリストを描画するときは「ちょうど N 列になるように」文字列を整える必要があります。

```rust
fn char_width(c: char) -> usize {
    let cp = c as u32;
    if (0x3041..=0x33FF).contains(&cp) || (0x4E00..=0x9FFF).contains(&cp) || (0xFF01..=0xFF60).contains(&cp) {
        2
    } else {
        1
    }
}

/// 表示幅が max 以下になるように切り詰める
fn truncate(s: &str, max: usize) -> &str {
    let mut w = 0;
    for (i, c) in s.char_indices() {
        let cw = char_width(c);
        if w + cw > max {
            return &s[..i]; // i は char の先頭なので安全にスライスできる
        }
        w += cw;
    }
    s
}

/// 表示幅がちょうど width になるよう、右側を空白で埋める (はみ出す分は切り詰める)
fn pad(s: &str, width: usize) -> String {
    let t = truncate(s, width);
    let w: usize = t.chars().map(char_width).sum();
    format!("{t}{}", " ".repeat(width - w))
}

fn main() {
    let rows = [("main.rs", "メイン"), ("日本語ファイル名.txt", "テキスト"), ("a", "とても長い説明文がここに入る")];
    for (name, desc) in rows {
        println!("|{}|{}|", pad(name, 12), pad(desc, 10));
    }
}
```

ポイント:

- 全角文字が境界をまたぐ場合 (残り 1 列に 2 列の文字) は、その文字を入れずに空白で埋める
- 切り詰めたことを示す `…` を付けるなら、`…` の幅 (1 列) を先に確保してから切る

## TUI でなぜ必須か

第22章で作るセルバッファでは、画面を「列 × 行」の 2 次元配列として持ちます。
`"あ"` を (x=5, y=0) に書くと、セル 5 に `あ`、セル 6 は「前の文字の続き」になります。
ここで幅を 1 と誤ると、次の文字が `あ` の右半分に重なって表示が崩れます。

エディタ (プロジェクト p05) では、さらに 3 つの座標を行き来します。

1. **バイト位置** … `String` の中のどこを編集するか (`insert`, `remove`)
2. **char 位置** … カーソルが何文字目にいるか (左右キーで ±1)
3. **表示列** … 画面の何列目にカーソルを描くか

char 位置から表示列へは「その位置までの `char_width` の合計」、
表示列から char 位置へは「幅を足していき、超えた所」で変換します。

```rust
fn char_width(c: char) -> usize {
    if (c as u32) >= 0x3000 { 2 } else { 1 }
}

fn col_of(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(char_width).sum()
}

fn byte_of(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(s.len())
}

fn main() {
    let line = "let 名前 = 1;";
    for i in [0, 4, 5, 6, 8] {
        println!("char {i} -> byte {} / col {}", byte_of(line, i), col_of(line, i));
    }
}
```

## Ambiguous 幅と端末の違い

`○` `→` `①` などは East Asian Width が A (曖昧) で、端末の設定によって 1 列にも 2 列にもなります。
また絵文字は、端末やフォントによって幅の扱いが異なることがあります。
完全に一致させるのは不可能なので、実用上は

- Ambiguous は 1 列として扱う (多くの端末の既定)
- ボーダーなど重要な部分には Ambiguous 文字を避け、罫線 `─│┌┐` (幅 1) を使う

と割り切るのが一般的です。ratatui も内部で `unicode-width` を使い、同じ方針です。

## よくあるコンパイルエラーと読み方

**`error[E0277]: the type `str` cannot be indexed by `{integer}``**
→ `s[3]` のように 1 文字を添字で取ろうとしています。UTF-8 では「3 番目の文字」を O(1) で
取れないため、Rust は禁止しています。`s.chars().nth(3)` か `s.as_bytes()[3]` を使います。

**`byte index 1 is not a char boundary; it is inside 'あ'`** (実行時パニック)
→ スライスの位置が文字の途中です。`char_indices()` で得たバイト位置を使いましょう。

**`error[E0308]: mismatched types: expected `usize`, found `u32``**
→ `c as u32` の結果をそのまま幅計算に使っています。`as usize` で変換します。

**`error[E0599]: no method named `width` found for type `char``**
→ `unicode-width` クレートの `UnicodeWidthChar` トレイトを `use` していないか、そもそも
標準ライブラリにはこのメソッドがありません。

## まとめ

- `String` は UTF-8。`len()` はバイト数
- 「1 文字」には char と書記素クラスタの 2 種類がある
- 端末の列幅は East Asian Width で決まり、CJK は 2 列、結合文字は 0 列
- `format!` の幅指定は表示幅を考慮しないので、TUI では自前で埋める
- バイト位置・char 位置・表示列の変換が、エディタやセルバッファの基礎になる

## 次へ

第17章から Stage 4 です。ターミナルが実際にどう動いているのか、そして
`\x1b[31m` のような ANSI エスケープシーケンスで色やカーソルを操る方法を学びます。
