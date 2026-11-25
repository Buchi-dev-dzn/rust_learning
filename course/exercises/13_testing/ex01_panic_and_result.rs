//! # パニックと Result をテストする
//!
//! テストに書かれた「仕様」を読み取って、関数を実装しよう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `Rect::new(w, h)`: 幅か高さが 0 ならパニックする。メッセージは
//!   `"Rect のサイズは 1 以上: {w}x{h}"` (テストの `expected` を読もう)
//! - `Rect::area`: 面積 (`u32`)
//! - `parse_size("80x24")`: `(80, 24)` を返す。失敗時は `SizeError` のどれか
//!   - `x` がない → `SizeError::MissingX`
//!   - 幅や高さが数値でない → `SizeError::BadNumber(その部分の文字列)`
//!   - 0 を含む → `SizeError::Zero`
//! - 関連: 第13章「異常系のテスト」

#[derive(Debug, PartialEq)]
pub struct Rect {
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(width: u16, height: u16) -> Rect {
        // TODO: 0 ならメッセージ付きでパニックする (assert! か panic! を使う)
        todo!()
    }

    pub fn area(&self) -> u32 {
        // TODO: u16::MAX * 2 でもオーバーフローしないように
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum SizeError {
    MissingX,
    BadNumber(String),
    Zero,
}

pub fn parse_size(s: &str) -> Result<(u16, u16), SizeError> {
    // TODO: split_once('x') と parse::<u16>() と ? 演算子で書く
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_area() {
        let r = Rect::new(80, 24);
        assert_eq!(r, Rect { width: 80, height: 24 });
        assert_eq!(r.area(), 1920);
        assert_eq!(Rect::new(u16::MAX, 2).area(), 131070);
    }

    #[test]
    #[should_panic(expected = "Rect のサイズは 1 以上: 0x10")]
    fn zero_width_panics() {
        Rect::new(0, 10);
    }

    #[test]
    #[should_panic(expected = "1 以上: 5x0")]
    fn zero_height_panics() {
        Rect::new(5, 0);
    }

    #[test]
    fn parse_ok() -> Result<(), SizeError> {
        assert_eq!(parse_size("80x24")?, (80, 24));
        assert_eq!(parse_size("1x1")?, (1, 1));
        Ok(())
    }

    #[test]
    fn parse_errors() {
        assert_eq!(parse_size("80*24"), Err(SizeError::MissingX));
        assert_eq!(parse_size("abcx24"), Err(SizeError::BadNumber("abc".to_string())));
        assert_eq!(parse_size("80x"), Err(SizeError::BadNumber("".to_string())));
        assert_eq!(parse_size("0x24"), Err(SizeError::Zero));
        assert!(matches!(parse_size("99999x1"), Err(SizeError::BadNumber(_))));
    }
}
