//! # キーバインド設定を HashMap で読む
//!
//! 設定ファイル風のテキストからキーバインド表を作り、検索・集計しよう。
//!
//! - ゴール: `HashMap` の insert / get / entry を使い、結果をソートして決まった順序で返す
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第6章「HashMap<K, V>」「entry API で集計」
//!
//! 設定の書式:
//! ```text
//! # コメント行 (先頭が #) と空行は無視
//! q = quit
//! ctrl-s = save
//! j=down          # = の前後の空白は無いこともある (行末コメントは考えなくてよい)
//! ```
//! - `=` を含まない行は無視する。
//! - キーと値は前後の空白を取り除く。キーかコマンドが空なら無視する。
//! - 同じキーが複数回出たら、**後のものが勝つ**。

use std::collections::HashMap;

fn parse_bindings(config: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    // TODO: config.lines() で 1 行ずつ処理する。split_once('=') が便利
    map
}

/// キーを押したときのコマンドを返す。未割り当てなら None。
fn command_for<'a>(map: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    // TODO: map.get(key) は Option<&String>。as_str に変換する
    todo!()
}

/// コマンドごとに、割り当てられているキーの一覧を作る。
/// 結果は (コマンド, キー一覧) の Vec で、コマンド名の辞書順。キー一覧も辞書順。
fn keys_by_command(map: &HashMap<String, String>) -> Vec<(String, Vec<String>)> {
    // TODO: HashMap<String, Vec<String>> を entry(..).or_default() で作り、
    //       最後に Vec に変換してソートする (HashMap の順序は不定なので必ずソートする)
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "
# my keys
q = quit
ctrl-s = save
j=down
k = up
down-arrow = down

this line has no equals
 = nothing
esc = quit
q = force-quit
";

    #[test]
    fn parses() {
        let m = parse_bindings(CONFIG);
        assert_eq!(m.len(), 6);
        assert_eq!(m.get("ctrl-s").map(|s| s.as_str()), Some("save"));
        assert_eq!(m.get("j").map(|s| s.as_str()), Some("down"));
        assert_eq!(m.get("q").map(|s| s.as_str()), Some("force-quit"));
        assert!(!m.contains_key(""));
    }

    #[test]
    fn looks_up() {
        let m = parse_bindings(CONFIG);
        assert_eq!(command_for(&m, "k"), Some("up"));
        assert_eq!(command_for(&m, "x"), None);
    }

    #[test]
    fn groups() {
        let m = parse_bindings(CONFIG);
        let g = keys_by_command(&m);
        let expected: Vec<(String, Vec<String>)> = vec![
            ("down".into(), vec!["down-arrow".into(), "j".into()]),
            ("force-quit".into(), vec!["q".into()]),
            ("quit".into(), vec!["esc".into()]),
            ("save".into(), vec!["ctrl-s".into()]),
            ("up".into(), vec!["k".into()]),
        ];
        assert_eq!(g, expected);
    }

    #[test]
    fn empty_config() {
        assert!(parse_bindings("").is_empty());
        assert!(keys_by_command(&HashMap::new()).is_empty());
    }
}
