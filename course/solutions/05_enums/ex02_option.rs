//! # Option を使いこなす
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: 「値がないかもしれない」を `Option` で返し、`match` / `if let` / `let else` /
//!   `unwrap_or` / `map` などで扱う
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第5章「Option: null の代わり」「Option を扱う便利メソッド」「if let / let else」

/// "80x24" のような文字列を (幅, 高さ) に変換する。形式が違えば None。
/// (例: parse_size("80x24") == Some((80, 24)), parse_size("80*24") == None)
fn parse_size(s: &str) -> Option<(u16, u16)> {
    let Some((w, h)) = s.split_once('x') else {
        return None;
    };
    let Some(w) = w.parse::<u16>().ok() else {
        return None;
    };
    let Some(h) = h.parse::<u16>().ok() else {
        return None;
    };
    Some((w, h))
    // 第7章で学ぶ `?` を使うと次のように短く書ける:
    //   let (w, h) = s.split_once('x')?;
    //   Some((w.parse().ok()?, h.parse().ok()?))
}

/// リスト `items` の中で `selected` 番目の要素を返す。
/// 選択なし (None) または範囲外なら "(none)" を返す。
fn selected_label(items: &[String], selected: Option<usize>) -> &str {
    // and_then で「Option を返す処理」をつなぎ、map で &String -> &str に変換。
    selected
        .and_then(|i| items.get(i))
        .map(|s| s.as_str())
        .unwrap_or("(none)")
}

/// 選択位置を 1 つ下に動かした新しい選択位置を返す。
/// - 何も選択されていなければ、要素があれば 0 番目を選ぶ
/// - 末尾にいたら末尾のまま
/// - リストが空なら常に None
fn select_next(selected: Option<usize>, len: usize) -> Option<usize> {
    match (selected, len) {
        (_, 0) => None,
        (None, _) => Some(0),
        (Some(i), len) => Some((i + 1).min(len - 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sizes() {
        assert_eq!(parse_size("80x24"), Some((80, 24)));
        assert_eq!(parse_size("1x1"), Some((1, 1)));
        assert_eq!(parse_size("80*24"), None);
        assert_eq!(parse_size("ax24"), None);
        assert_eq!(parse_size("80x"), None);
        assert_eq!(parse_size("99999x1"), None); // u16 に収まらない
    }

    #[test]
    fn labels() {
        let items = vec!["src".to_string(), "Cargo.toml".to_string()];
        assert_eq!(selected_label(&items, Some(1)), "Cargo.toml");
        assert_eq!(selected_label(&items, Some(5)), "(none)");
        assert_eq!(selected_label(&items, None), "(none)");
    }

    #[test]
    fn selects_next() {
        assert_eq!(select_next(None, 3), Some(0));
        assert_eq!(select_next(Some(0), 3), Some(1));
        assert_eq!(select_next(Some(2), 3), Some(2));
        assert_eq!(select_next(None, 0), None);
        assert_eq!(select_next(Some(4), 0), None);
    }
}
