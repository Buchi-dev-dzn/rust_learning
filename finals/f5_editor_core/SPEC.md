# F5: エディタのコア

テキストエディタの「中身」= **テキストバッファ + カーソル + 編集操作 + Undo/Redo + 検索** を作る。
画面描画はしない (それは P05 や F4 の仕事)。ここではロジックだけを、テストで厳密に確かめる。

この課題で身につくこと: 文字単位とバイト単位の違い (UTF-8 の `String` を文字位置で編集する)、
行の分割・結合の境界条件、単語移動の定義、**Undo のグループ化**という「仕様を決めて守る」設計。

外部クレートは使わない (標準ライブラリのみ)。

## 1. 公開 API (`src/lib.rs`)

```rust
#[derive(Debug, Clone)]
pub struct Editor { /* 自由 */ }

impl Default for Editor { /* Editor::new() */ }

impl Editor {
    pub fn new() -> Editor;                       // 空のバッファ (1 行・空文字列)
    pub fn from_text(text: &str) -> Editor;       // カーソルは (0, 0)
    pub fn text(&self) -> String;
    pub fn line(&self, row: usize) -> &str;       // 範囲外はパニックしてよい
    pub fn line_count(&self) -> usize;
    pub fn cursor(&self) -> (usize, usize);       // (row, col)
    pub fn set_cursor(&mut self, row: usize, col: usize);

    pub fn insert_char(&mut self, c: char);
    pub fn insert_newline(&mut self);
    pub fn insert_str(&mut self, s: &str);
    pub fn backspace(&mut self) -> bool;          // 何か変わったら true
    pub fn delete(&mut self) -> bool;             // 何か変わったら true

    pub fn move_left(&mut self);
    pub fn move_right(&mut self);
    pub fn move_up(&mut self);
    pub fn move_down(&mut self);
    pub fn move_home(&mut self);
    pub fn move_end(&mut self);
    pub fn move_word_left(&mut self);
    pub fn move_word_right(&mut self);

    pub fn undo(&mut self) -> bool;               // 何か戻したら true
    pub fn redo(&mut self) -> bool;

    pub fn find(&self, query: &str, from: (usize, usize)) -> Option<(usize, usize)>;
    pub fn find_all(&self, query: &str) -> Vec<(usize, usize)>;
}
```

## 2. バッファとカーソル

- テキストは**行の列**。行は常に 1 行以上ある。`from_text` は `'\n'` で分割する
  (`""` → 1 行 `[""]`、`"a\n"` → 2 行 `["a", ""]`)。`text()` は行を `'\n'` で連結 (= `from_text` の逆)。
- カーソル `(row, col)` の `col` は**文字 (char) 単位**。`0 <= col <= その行の文字数`。
  `"あいう"` の末尾は col = 3。
- `set_cursor(row, col)`: `row` を最終行に、`col` をその行の文字数に**丸めて**設定する。

## 3. 編集

| 操作 | 動作 |
|---|---|
| `insert_char(c)` | カーソル位置に `c` を挿入し、col を 1 進める。`c == '\n'` なら `insert_newline()` と同じ。 |
| `insert_newline()` | カーソル位置で行を分割。カーソルは新しい行の先頭 `(row+1, 0)`。 |
| `insert_str(s)` | `s` を順に挿入 (`'\n'` は改行)。カーソルは挿入したテキストの直後。`s` が空なら何もしない。 |
| `backspace()` | col > 0: カーソルの左の 1 文字を消し col を 1 減らす。col == 0 かつ row > 0: この行を前の行の末尾に結合し、カーソルは結合点 (前の行の元の長さ)。`(0, 0)` では何もせず `false`。 |
| `delete()` | col < 行の長さ: カーソル位置の 1 文字を消す (カーソルは動かない)。行末かつ次の行がある: 次の行をこの行に結合。最終行の行末では何もせず `false`。 |

## 4. 移動

- `move_left`: col > 0 なら 1 戻る。行頭なら前の行の行末へ。`(0, 0)` では動かない。
- `move_right`: 行末でなければ 1 進む。行末なら次の行の行頭へ。最終行の行末では動かない。
- `move_home` / `move_end`: 行頭 (col 0) / 行末。
- `move_up` / `move_down`: 1 行上/下へ。**目標列 (goal column)** を使う:
  - 縦移動を始めるとき、その時点の col を目標列として覚える。
  - 縦移動のたびに `col = min(目標列, 移動先の行の文字数)`。
  - 目標列は、縦移動**以外**のあらゆる操作 (横移動・編集・`set_cursor`・undo/redo) で忘れる。
  - 先頭行での `move_up`、最終行での `move_down` は何もしない (目標列も保持したまま)。

  例: 行 `["abcdef", "ab", "abcdef"]`、(0, 5) から down → (1, 2)、down → (2, 5)。

### 単語移動

文字の種類を 3 つに分ける: **空白** (`char::is_whitespace`)、**単語** (`is_alphanumeric()` または `'_'`)、**記号** (それ以外)。

- `move_word_right`:
  - 行末なら次の行の行頭へ (最終行なら動かない)。
  - そうでなければ: カーソル位置の文字が空白でなければ、**同じ種類の文字が続く間**進む。その後、**空白が続く間**進む。
- `move_word_left`:
  - 行頭なら前の行の行末へ (先頭行なら動かない)。
  - そうでなければ: 左の文字が**空白の間**戻る。その後、行頭でなければ、左の文字と**同じ種類が続く間**戻る。

例: `"foo.bar  baz"` で col 0 から `move_word_right` を繰り返すと 3 → 4 → 9 → 12。
col 12 から `move_word_left` を繰り返すと 9 → 4 → 3 → 0。

## 5. Undo / Redo

**グループ**単位で戻す・やり直す。undo は「グループ開始直前のテキストとカーソル」に戻し、
redo は「undo した時点のテキストとカーソル」に戻す。

グループ化の規則:

1. `insert_char` (改行以外) は、**直前の操作も `insert_char` で、直前に挿入した文字が空白でない**とき、直前のグループに合流する。
   それ以外は新しいグループを始める。
   (つまり `"hello world"` と打つと `"hello "` と `"world"` の 2 グループ。空白の直後で切れる)
2. `backspace` は直前の操作も (実際に何かを消した) `backspace` なら合流。`delete` も同様に `delete` 同士で合流。
3. `insert_newline` (`insert_char('\n')` を含む) と `insert_str` は、それ単独で 1 グループ。次の編集とも合流しない。
4. **移動系の操作** (`move_*`・`set_cursor`) を呼ぶと、カーソルが実際に動いたかどうかに関わらず、現在のグループは終わる。
   `undo` / `redo` もグループを終わらせる。
5. 何も変えなかった操作 (`(0,0)` での `backspace`、空の `insert_str` など) は、履歴にもグループ化にも影響しない。
6. 新しい編集をすると redo の履歴は消える。
7. 戻すものがなければ `undo()` は `false`、やり直すものがなければ `redo()` は `false`。

## 6. 検索

- `find(query, from)`: `from = (row, col)` の位置**以降** (その位置から始まる一致を含む) で最初に現れる `query` の
  開始位置 `(row, col)` を返す。最後まで見つからなければ**先頭に戻って** `from` の手前まで探す (ラップアラウンド)。
  - 一致は行をまたがない。`query` が空、または `'\n'` を含むなら `None`。
  - `from.0` は最終行に、`from.1` はその行の長さに丸めて扱う。
  - 返す col は**文字単位**。
- `find_all(query)`: 全一致の開始位置を先頭から順に返す。同じ行の中では重ならないように探す
  (一致の直後から次を探す。`"aaaa"` から `"aa"` → `[(0,0), (0,2)]`)。空クエリなら空。

## 7. `src/main.rs` (テスト対象外・starter に同梱済み)

一連の操作を行い、カーソルを `|` で示したテキストを表示する小さなデモ。
`cargo run -p f5_editor_core` で、完成した lib.rs の動きを目で確かめられる。

## 合格条件

`cargo test -p f5_editor_core` がすべて成功すること。
