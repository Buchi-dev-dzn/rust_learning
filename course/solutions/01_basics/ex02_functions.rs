//! # 関数と式
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: 引数・戻り値のある関数を書き、`if` を式として使う
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第1章「関数」「式と文」「整数のオーバーフロー」

/// `value` を `min..=max` の範囲に収めて返す。
/// (例: clamp(15, 0, 10) == 10, clamp(-3, 0, 10) == 0)
/// 標準の `i32::clamp` は使わずに `if` で書いてみよう。
fn clamp(value: i32, min: i32, max: i32) -> i32 {
    // if 全体が式なので、その値がそのまま戻り値になる。
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// カーソルの列 `col` を左に `n` 動かした結果を返す。0 より左には行かない。
/// (例: move_left(5, 2) == 3, move_left(1, 4) == 0)
fn move_left(col: u16, n: u16) -> u16 {
    // 0 で止まる引き算。
    col.saturating_sub(n)
}

/// 幅 `width` の中に長さ `len` の文字列を中央寄せするとき、左に入れる空白の数を返す。
/// 余りが出るときは左を少なくする。`len` が `width` より大きければ 0。
/// (例: center_pad(10, 4) == 3, center_pad(10, 3) == 3, center_pad(3, 8) == 0)
fn center_pad(width: usize, len: usize) -> usize {
    // 整数の割り算は切り捨てなので、余りは自動的に右側に回る。
    width.saturating_sub(len) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps() {
        assert_eq!(clamp(15, 0, 10), 10);
        assert_eq!(clamp(-3, 0, 10), 0);
        assert_eq!(clamp(7, 0, 10), 7);
        assert_eq!(clamp(0, 0, 10), 0);
    }

    #[test]
    fn moves_left_without_underflow() {
        assert_eq!(move_left(5, 2), 3);
        assert_eq!(move_left(1, 4), 0);
        assert_eq!(move_left(0, 1), 0);
    }

    #[test]
    fn centers() {
        assert_eq!(center_pad(10, 4), 3);
        assert_eq!(center_pad(10, 3), 3);
        assert_eq!(center_pad(3, 8), 0);
        assert_eq!(center_pad(80, 80), 0);
    }
}
