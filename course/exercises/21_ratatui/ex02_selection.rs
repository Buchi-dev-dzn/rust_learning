//! # StatefulWidget の選択状態
//!
//! リストの選択位置を `Option<usize>` で管理しよう。`None` は未選択。
//! 空のリストでは常に `None`。非空なら `Next` は末尾で先頭へ、`Previous` は先頭で末尾へ循環する。
//! 未選択からの `Next` は先頭、`Previous` は末尾を選ぶ。
//! `Replace(items)` は項目を置換し、選択済みの index は新しい末尾に収める (未選択はそのまま)。
//! `Clear` は選択だけを解除する。描画時に状態を変えず、イベント処理時にだけ変える。
//!
//! - 関連: 第21章「StatefulWidget と ListState」
//! - テストは編集しないこと。

#[derive(Debug)]
struct ListModel { items: Vec<String>, selected: Option<usize> }
enum Msg { Next, Previous, Replace(Vec<String>), Clear }

// TODO: 状態遷移を実装する。
fn update(model: &mut ListModel, msg: Msg) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model(n: usize) -> ListModel {
        ListModel { items: (0..n).map(|i| i.to_string()).collect(), selected: None }
    }
    #[test]
    fn navigation() {
        let mut m = model(3);
        update(&mut m, Msg::Next);
        assert_eq!(m.selected, Some(0));
        update(&mut m, Msg::Previous);
        assert_eq!(m.selected, Some(2));
        update(&mut m, Msg::Next);
        assert_eq!(m.selected, Some(0));
        update(&mut m, Msg::Clear);
        update(&mut m, Msg::Previous);
        assert_eq!(m.selected, Some(2));
    }
    #[test]
    fn empty_and_replace() {
        let mut m = model(0);
        update(&mut m, Msg::Next);
        update(&mut m, Msg::Previous);
        assert_eq!(m.selected, None);
        update(&mut m, Msg::Replace(vec!["a".into(), "b".into()]));
        assert_eq!(m.selected, None);
        update(&mut m, Msg::Previous);
        update(&mut m, Msg::Replace(vec!["c".into()]));
        assert_eq!(m.selected, Some(0));
        update(&mut m, Msg::Replace(vec![]));
        assert_eq!(m.selected, None);
    }
}
