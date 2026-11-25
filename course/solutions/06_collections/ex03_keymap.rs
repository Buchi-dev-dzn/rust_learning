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
    for line in config.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, cmd)) = line.split_once('=') else {
            continue;
        };
        let (key, cmd) = (key.trim(), cmd.trim());
        if key.is_empty() || cmd.is_empty() {
            continue;
        }
        // insert は既存の値を上書きするので「後のものが勝つ」が自然に実現できる。
        map.insert(key.to_string(), cmd.to_string());
    }
    map
}

/// キーを押したときのコマンドを返す。未割り当てなら None。
fn command_for<'a>(map: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    // String のキーを &str で引ける (Borrow トレイトのおかげ)。
    map.get(key).map(|s| s.as_str())
}

/// コマンドごとに、割り当てられているキーの一覧を作る。
/// 結果は (コマンド, キー一覧) の Vec で、コマンド名の辞書順。キー一覧も辞書順。
fn keys_by_command(map: &HashMap<String, String>) -> Vec<(String, Vec<String>)> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for (key, cmd) in map {
        groups.entry(cmd.clone()).or_default().push(key.clone());
    }
    let mut out: Vec<(String, Vec<String>)> = groups.into_iter().collect();
    for (_, keys) in &mut out {
        keys.sort();
    }
    out.sort(); // タプルは先頭要素 (コマンド名) から順に比較される
    out
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
