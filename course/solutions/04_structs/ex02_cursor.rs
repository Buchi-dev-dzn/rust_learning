//! # Cursor を設計する
//!
//! 画面サイズの範囲内を動くカーソル `Cursor` を **自分で設計** しよう。
//! 構造体のフィールドは自由に決めてよい。テストが使う API だけ満たせば合格。
//!
//! - ゴール: 状態 (フィールド) を決め、`&self` / `&mut self` のメソッドで操作を提供する
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第4章「メソッドと impl」「derive でよく使うトレイト」
//!
//! 必要な API:
//! - `Cursor::new(cols, rows)` — 画面サイズ cols x rows、カーソルは (0, 0)
//! - `col()` / `row()` — 現在位置 (u16)
//! - `up()` / `down()` / `left()` / `right()` — 1 マス動く。画面外には出ない
//! - `home()` — 行頭 (col = 0) へ、`end()` — 行末 (col = cols - 1) へ
//! - `resize(cols, rows)` — 画面サイズ変更。カーソルが画面外になったら内側に寄せる
//!
//! 画面サイズが 0 のときは常に (0, 0) とする。

// フィールドは非公開にして、メソッド経由でしか変えられないようにする。
// こうすると「カーソルは必ず画面内」という不変条件を Cursor 自身が守れる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cursor {
    col: u16,
    row: u16,
    cols: u16,
    rows: u16,
}

impl Cursor {
    fn new(cols: u16, rows: u16) -> Self {
        Self { col: 0, row: 0, cols, rows }
    }

    fn col(&self) -> u16 {
        self.col
    }

    fn row(&self) -> u16 {
        self.row
    }

    /// 列の最大値 (サイズ 0 でも溢れないよう saturating_sub)
    fn max_col(&self) -> u16 {
        self.cols.saturating_sub(1)
    }

    fn max_row(&self) -> u16 {
        self.rows.saturating_sub(1)
    }

    fn up(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    fn down(&mut self) {
        self.row = (self.row + 1).min(self.max_row());
    }

    fn left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }

    fn right(&mut self) {
        self.col = (self.col + 1).min(self.max_col());
    }

    fn home(&mut self) {
        self.col = 0;
    }

    fn end(&mut self) {
        self.col = self.max_col();
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.col = self.col.min(self.max_col());
        self.row = self.row.min(self.max_row());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_origin() {
        let c = Cursor::new(80, 24);
        assert_eq!((c.col(), c.row()), (0, 0));
    }

    #[test]
    fn moves_and_stops_at_edges() {
        let mut c = Cursor::new(3, 2);
        c.left();
        c.up();
        assert_eq!((c.col(), c.row()), (0, 0));
        c.right();
        c.right();
        c.right();
        c.right();
        assert_eq!(c.col(), 2);
        c.down();
        c.down();
        assert_eq!(c.row(), 1);
        c.left();
        assert_eq!((c.col(), c.row()), (1, 1));
    }

    #[test]
    fn home_and_end() {
        let mut c = Cursor::new(10, 5);
        c.end();
        assert_eq!(c.col(), 9);
        c.home();
        assert_eq!(c.col(), 0);
    }

    #[test]
    fn resize_clamps() {
        let mut c = Cursor::new(10, 10);
        c.end();
        for _ in 0..20 {
            c.down();
        }
        assert_eq!((c.col(), c.row()), (9, 9));
        c.resize(4, 3);
        assert_eq!((c.col(), c.row()), (3, 2));
        c.resize(40, 30);
        assert_eq!((c.col(), c.row()), (3, 2));
    }

    #[test]
    fn zero_size() {
        let mut c = Cursor::new(0, 0);
        c.right();
        c.down();
        c.end();
        assert_eq!((c.col(), c.row()), (0, 0));
    }
}
