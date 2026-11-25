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

// TODO: フィールドを決める (位置と画面サイズが必要)
struct Cursor {}

impl Cursor {
    // TODO: 上の API をすべて実装する
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
