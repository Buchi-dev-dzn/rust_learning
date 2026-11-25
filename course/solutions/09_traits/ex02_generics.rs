//! # ジェネリクスとトレイト境界
//!
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `clamp`: 値を `lo..=hi` の範囲に収める (カーソル位置の制限によく使う)
//! - `largest`: スライスの最大値。空なら `None`
//! - `join_display`: `Display` を実装した値を区切り文字でつなぐ
//! - `History<T>`: 上限付きの履歴 (古いものから捨てる) をジェネリックに実装する
//! - 関連: 第9章「ジェネリクスとトレイト境界」

use std::fmt::Display;

// PartialOrd があれば < と > で比べられる
pub fn clamp<T: PartialOrd>(value: T, lo: T, hi: T) -> T {
    if value < lo {
        lo
    } else if value > hi {
        hi
    } else {
        value
    }
}

// Copy 境界があるので *x で値を取り出せる
pub fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter();
    let mut max = *iter.next()?;
    for &x in iter {
        if x > max {
            max = x;
        }
    }
    Some(max)
}

pub fn join_display<T: Display>(items: &[T], sep: &str) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(&item.to_string());
    }
    out
}

/// 上限付きの履歴。上限を超えたら最も古い要素を捨てる。
pub struct History<T> {
    items: Vec<T>,
    cap: usize,
}

// impl の後にも <T> が必要
impl<T> History<T> {
    pub fn new(cap: usize) -> Self {
        History { items: Vec::new(), cap }
    }

    pub fn push(&mut self, item: T) {
        if self.items.len() == self.cap {
            self.items.remove(0);
        }
        self.items.push(item);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 最新の要素への参照
    pub fn latest(&self) -> Option<&T> {
        self.items.last()
    }
}

// 境界付きの impl: T が Display のときだけ使えるメソッド
impl<T: Display> History<T> {
    pub fn render(&self) -> String {
        join_display(&self.items, " > ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_works_for_many_types() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-3, 0, 10), 0);
        assert_eq!(clamp(99u16, 0, 79), 79);
        assert_eq!(clamp(1.5, 0.0, 1.0), 1.0);
        assert_eq!(clamp("m", "a", "k"), "k");
    }

    #[test]
    fn largest_works() {
        assert_eq!(largest(&[3, 9, 2]), Some(9));
        assert_eq!(largest(&['a', 'z', 'q']), Some('z'));
        assert_eq!(largest::<i32>(&[]), None);
    }

    #[test]
    fn join_display_works() {
        assert_eq!(join_display(&[1, 2, 3], ", "), "1, 2, 3");
        assert_eq!(join_display(&["a"], "-"), "a");
        assert_eq!(join_display::<i32>(&[], "-"), "");
    }

    #[test]
    fn history_drops_oldest() {
        let mut h = History::new(3);
        for cmd in ["ls", "cd", "vim", "make"] {
            h.push(cmd);
        }
        assert_eq!(h.len(), 3);
        assert_eq!(h.latest(), Some(&"make"));
        assert_eq!(h.render(), "cd > vim > make");
    }

    #[test]
    fn history_with_numbers() {
        let mut h: History<u32> = History::new(2);
        assert_eq!(h.latest(), None);
        h.push(1);
        h.push(2);
        h.push(3);
        assert_eq!(h.render(), "2 > 3");
    }
}
