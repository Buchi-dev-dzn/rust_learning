//! # 差分描画の更新列
//!
//! 前フレームと次フレームを行優先で比較し、変わったセルだけ `Change` にする。
//! 端末のカーソル移動はこの座標を使う。寸法が異なる場合は `None` を返し、呼び出し側が全描画する。
//! 同じセルでも色が変われば更新が必要。空のバッファなら変更はない。
//!
//! - 関連: 第22章「差分レンダラ」
//! - テストは編集しないこと。

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cell { ch: char, color: u8 }
#[derive(Debug)]
struct Buffer { width: usize, height: usize, cells: Vec<Cell> }
#[derive(Debug, PartialEq, Eq)]
struct Change { x: usize, y: usize, cell: Cell }

// TODO: 寸法が異なれば None、同じなら変更セルの列を返す。
fn diff(old: &Buffer, next: &Buffer) -> Option<Vec<Change>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn buffer(w: usize, h: usize) -> Buffer {
        Buffer { width: w, height: h, cells: vec![Cell { ch: ' ', color: 0 }; w * h] }
    }
    #[test]
    fn changed_cells_in_row_order() {
        let old = buffer(3, 2);
        let mut next = buffer(3, 2);
        next.cells[2].ch = 'x';
        next.cells[3].color = 4;
        assert_eq!(diff(&old, &next), Some(vec![
            Change { x: 2, y: 0, cell: Cell { ch: 'x', color: 0 } },
            Change { x: 0, y: 1, cell: Cell { ch: ' ', color: 4 } },
        ]));
    }
    #[test]
    fn unchanged_and_resize() {
        assert_eq!(diff(&buffer(2, 1), &buffer(2, 1)), Some(vec![]));
        assert_eq!(diff(&buffer(0, 0), &buffer(0, 0)), Some(vec![]));
        assert_eq!(diff(&buffer(2, 1), &buffer(1, 2)), None);
    }
}
