最初のコンパイルエラー E0277 (`Rc<RefCell<Vec<String>>>` cannot be sent between threads safely) は、
`SharedLog` の中身がスレッド安全でないことを指摘しています。
`Rc` → `Arc`、`RefCell` → `Mutex`、`borrow()` / `borrow_mut()` → `lock().unwrap()` に置き換えます。

---

`parallel_count` の形:

```rust
let counter = Arc::new(Mutex::new(0usize));
let mut handles = Vec::new();
for _ in 0..threads {
    let c = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        for _ in 0..per_thread {
            *c.lock().unwrap() += 1;
        }
    }));
}
for h in handles { h.join().unwrap(); }
*counter.lock().unwrap()
```

---

`run_jobs` のワーカー本体:

```rust
loop {
    let job = queue.lock().unwrap().pop(); // この文でロックが解放される
    let Some(job) = job else { break };
    results.lock().unwrap().push(job.to_uppercase());
}
```

`while let Some(job) = queue.lock().unwrap().pop() { ... }` と書くと、ループ本体の間もロックを持ち続ける
ことに注意 (このテストでは動きますが、重い処理だと並列にならない)。
