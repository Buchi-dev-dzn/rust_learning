//! # ループと match
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: `for` / `while` / `loop` / `match` を使い分ける
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第1章「制御フロー」

/// 1 から `n` までの合計を返す (n == 0 なら 0)。`for` と範囲を使おう。
fn sum_to(n: u32) -> u32 {
    // TODO
    todo!()
}

/// 10 進数での桁数を返す。0 は 1 桁とする。
/// (例: digits(0) == 1, digits(7) == 1, digits(42) == 2, digits(1000) == 4)
fn digits(mut n: u64) -> u32 {
    // TODO: while か loop で 10 で割り続ける
    todo!()
}

/// FizzBuzz の 1 項を返す。3 の倍数は "Fizz"、5 の倍数は "Buzz"、
/// 両方の倍数は "FizzBuzz"、それ以外は数字の文字列。
fn fizzbuzz(n: u32) -> String {
    // TODO: `match (n % 3, n % 5) { ... }` と書くと見通しが良い
    //       文字列は "Fizz".to_string() や n.to_string() で作れる
    todo!()
}

/// 幅 `width` の水平線を返す。両端は '+'、間は '-'。
/// (例: hline(5) == "+---+", hline(2) == "++", hline(1) == "+", hline(0) == "")
fn hline(width: usize) -> String {
    let mut s = String::new();
    // TODO: for で 0..width を回し、位置に応じて s.push('+') か s.push('-')
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums() {
        assert_eq!(sum_to(0), 0);
        assert_eq!(sum_to(1), 1);
        assert_eq!(sum_to(10), 55);
    }

    #[test]
    fn counts_digits() {
        assert_eq!(digits(0), 1);
        assert_eq!(digits(7), 1);
        assert_eq!(digits(42), 2);
        assert_eq!(digits(1000), 4);
        assert_eq!(digits(u64::MAX), 20);
    }

    #[test]
    fn fizzbuzzes() {
        assert_eq!(fizzbuzz(1), "1");
        assert_eq!(fizzbuzz(3), "Fizz");
        assert_eq!(fizzbuzz(10), "Buzz");
        assert_eq!(fizzbuzz(30), "FizzBuzz");
    }

    #[test]
    fn draws_hline() {
        assert_eq!(hline(5), "+---+");
        assert_eq!(hline(2), "++");
        assert_eq!(hline(1), "+");
        assert_eq!(hline(0), "");
    }
}
