`Node` のフィールドは `RefCell` に包まれているので、`&Rc<Node>` からでも
`node.children.borrow_mut().push(...)` や `*node.parent.borrow_mut() = ...` で変更できます。

`Logger` も同じで、`self.lines.borrow_mut().push(msg.to_string())`、
読むときは `self.lines.borrow().clone()` です。

---

`path` は解説の例とほぼ同じです。

```rust
let mut names = vec![node.name.clone()];
let mut cur = node.parent.borrow().upgrade(); // Option<Rc<Node>>
while let Some(p) = cur {
    names.push(p.name.clone());
    cur = p.parent.borrow().upgrade();
}
names.reverse();
names.join("/")
```

`count` は `1 + root.children.borrow().iter().map(count).sum::<usize>()` と 1 行で書けます。

---

```rust
pub fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent);
    parent.children.borrow_mut().push(child);
}

pub fn find(root: &Rc<Node>, name: &str) -> Option<Rc<Node>> {
    if root.name == name {
        return Some(Rc::clone(root));
    }
    for child in root.children.borrow().iter() {
        if let Some(found) = find(child, name) {
            return Some(found);
        }
    }
    None
}
```
