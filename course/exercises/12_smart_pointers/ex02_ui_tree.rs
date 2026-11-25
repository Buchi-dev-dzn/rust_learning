//! # Rc / RefCell / Weak で UI ノードのツリーを作る
//!
//! 親子をたどれる UI ツリーと、複数の部品で共有するイベントログを作ろう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `Node`: 親への参照は `Weak`、子への参照は `Rc` で持つ
//! - `add_child(parent, child)`: 子の親を設定し、親の子リストに追加する
//! - `path(node)`: ルートからのパス `"root/panel/button"` を返す
//! - `find(root, name)`: 名前でノードを深さ優先で探す
//! - `count(root)`: 自分を含むノード数
//! - `Logger`: `Rc<RefCell<Vec<String>>>` を共有するロガー。clone したロガー同士は同じログに書く
//! - 関連: 第12章「Rc<RefCell<T>>」「Weak<T>」

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Node {
    pub name: String,
    pub parent: RefCell<Weak<Node>>,
    pub children: RefCell<Vec<Rc<Node>>>,
}

pub fn new_node(name: &str) -> Rc<Node> {
    Rc::new(Node {
        name: name.to_string(),
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(Vec::new()),
    })
}

pub fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
    // TODO: child.parent に Rc::downgrade(parent) を入れ、parent.children に child を push する
    todo!()
}

pub fn path(node: &Rc<Node>) -> String {
    // TODO: parent.borrow().upgrade() で親をたどり、名前を集めて逆順に "/" でつなぐ
    todo!()
}

pub fn find(root: &Rc<Node>, name: &str) -> Option<Rc<Node>> {
    // TODO: 自分が一致すれば Rc::clone(root) を返す。違えば子を再帰的に探す
    todo!()
}

pub fn count(root: &Rc<Node>) -> usize {
    // TODO
    todo!()
}

/// clone しても同じログを共有するロガー
#[derive(Clone)]
pub struct Logger {
    lines: Rc<RefCell<Vec<String>>>,
}

impl Logger {
    pub fn new() -> Self {
        Logger { lines: Rc::new(RefCell::new(Vec::new())) }
    }

    pub fn log(&self, msg: &str) {
        // TODO: &self からでも borrow_mut() で書き込める
        todo!()
    }

    pub fn lines(&self) -> Vec<String> {
        // TODO: 中身を clone して返す
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Rc<Node> {
        let root = new_node("root");
        let sidebar = new_node("sidebar");
        let main = new_node("main");
        add_child(&root, Rc::clone(&sidebar));
        add_child(&root, Rc::clone(&main));
        add_child(&main, new_node("editor"));
        add_child(&main, new_node("status"));
        add_child(&sidebar, new_node("tree"));
        root
    }

    #[test]
    fn path_to_root() {
        let root = sample();
        let status = find(&root, "status").expect("status があるはず");
        assert_eq!(path(&status), "root/main/status");
        assert_eq!(path(&root), "root");
        assert_eq!(path(&find(&root, "tree").unwrap()), "root/sidebar/tree");
    }

    #[test]
    fn find_and_count() {
        let root = sample();
        assert_eq!(count(&root), 6);
        assert!(find(&root, "nothing").is_none());
        let main = find(&root, "main").unwrap();
        assert_eq!(count(&main), 3);
    }

    #[test]
    fn parent_is_weak() {
        let root = sample();
        let main = find(&root, "main").unwrap();
        // root を持っているのは変数 root だけ (子からは Weak)
        assert_eq!(Rc::strong_count(&root), 1);
        assert_eq!(Rc::weak_count(&root), 2);
        // main は root の子リストと、変数 main の 2 か所から所有されている
        assert_eq!(Rc::strong_count(&main), 2);
    }

    #[test]
    fn parent_dropped() {
        let child = new_node("orphan");
        {
            let parent = new_node("temp");
            add_child(&parent, Rc::clone(&child));
            assert_eq!(path(&child), "temp/orphan");
        } // parent はここで解放される
        assert!(child.parent.borrow().upgrade().is_none());
        assert_eq!(path(&child), "orphan");
    }

    #[test]
    fn logger_is_shared() {
        let a = Logger::new();
        let b = a.clone();
        a.log("key j");
        b.log("tick");
        a.log("key q");
        assert_eq!(b.lines(), vec!["key j", "tick", "key q"]);
        assert_eq!(a.lines().len(), 3);
        let fresh = Logger::new();
        assert!(fresh.lines().is_empty());
    }
}
