//! # Copy な座標型
//!
//! このファイルは **コンパイルが通りません**。
//! ターミナル上の座標 `Pos` を整数のように気軽に渡したいのに、`Pos` が `Copy` でないためムーブが起きている。
//!
//! - ゴール: どんな型なら `Copy` にできるかを理解し、小さな値型に `Copy` を付ける
//! - 直してよいのは `// TODO:` の行だけ。関数本体とテストは変更しないこと。
//! - 関連: 第2章「Copy: スタックだけで完結する型」

// TODO: この 1 行上に derive 属性を追加して、Pos を Copy にする
//       (Copy には Clone も必要。比較・表示のため PartialEq と Debug は付いている)
#[derive(Debug, PartialEq)]
struct Pos {
    x: u16,
    y: u16,
}

fn right(p: Pos) -> Pos {
    Pos { x: p.x + 1, y: p.y }
}

fn down(p: Pos) -> Pos {
    Pos { x: p.x, y: p.y + 1 }
}

/// 元の位置と、右下に 1 つ動いた位置の両方を返す。
fn with_diagonal(p: Pos) -> (Pos, Pos) {
    let moved = down(right(p));
    (p, moved) // p がムーブ済みだとここでエラーになる
}

/// 開始位置から右に n 回動かした位置の一覧 (開始位置を含む) を返す。
fn trail(start: Pos, n: usize) -> Vec<Pos> {
    let mut out = vec![start];
    let mut cur = start;
    for _ in 0..n {
        cur = right(cur);
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal() {
        let (a, b) = with_diagonal(Pos { x: 1, y: 1 });
        assert_eq!(a, Pos { x: 1, y: 1 });
        assert_eq!(b, Pos { x: 2, y: 2 });
    }

    #[test]
    fn trails() {
        let t = trail(Pos { x: 0, y: 5 }, 2);
        assert_eq!(t, vec![Pos { x: 0, y: 5 }, Pos { x: 1, y: 5 }, Pos { x: 2, y: 5 }]);
    }
}
