//! 外部プロセス (rustc / cargo / テストバイナリ) を実行し、出力を HTTP レスポンスにストリームする。
//!
//! レスポンス形式: 生の出力 (ANSI 付き) が順に流れ、最後に `\x1e{"ok":true|false}` が 1 行付く。
//! ブラウザ側 (web/app.js) が ANSI → HTML 変換して表示する。

use axum::body::Body;
use bytes::Bytes;
use std::convert::Infallible;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Debug)]
pub struct Step {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub timeout: Duration,
    /// 表示用のコマンド文字列 (None なら program + args)
    pub display: Option<String>,
    pub env: Vec<(String, String)>,
}

impl Step {
    pub fn new(program: impl Into<String>, args: &[&str], cwd: PathBuf) -> Step {
        Step {
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            cwd,
            timeout: Duration::from_secs(600),
            display: None,
            env: vec![("CARGO_TERM_COLOR".into(), "always".into())],
        }
    }
    pub fn timeout(mut self, secs: u64) -> Step {
        self.timeout = Duration::from_secs(secs);
        self
    }
    pub fn display(mut self, s: impl Into<String>) -> Step {
        self.display = Some(s.into());
        self
    }
}

pub type Tx = mpsc::Sender<Bytes>;

/// 各ステップを順に実行し、途中で失敗したら止める。完了時に `on_done(ok)` を呼ぶ。
pub fn stream(steps: Vec<Step>, on_done: impl FnOnce(bool) + Send + 'static) -> Body {
    let (tx, rx) = mpsc::channel::<Bytes>(64);
    tokio::spawn(async move {
        let mut ok = true;
        for step in steps {
            match run_step(&step, &tx).await {
                Ok(true) => {}
                Ok(false) => {
                    ok = false;
                    break;
                }
                Err(e) => {
                    let _ = tx.send(Bytes::from(format!("\x1b[31m起動に失敗しました: {e}\x1b[0m\n"))).await;
                    ok = false;
                    break;
                }
            }
        }
        on_done(ok);
        let _ = tx.send(Bytes::from(format!("\n\x1e{{\"ok\":{ok}}}\n"))).await;
    });
    Body::from_stream(ReceiverStream::new(rx).map(Ok::<_, Infallible>))
}

pub async fn run_step(step: &Step, tx: &Tx) -> std::io::Result<bool> {
    let shown = step.display.clone().unwrap_or_else(|| {
        let mut s = step.program.clone();
        for a in &step.args {
            s.push(' ');
            s.push_str(a);
        }
        s
    });
    let _ = tx.send(Bytes::from(format!("\x1b[2m$ {shown}\x1b[0m\n"))).await;

    let mut cmd = Command::new(&step.program);
    cmd.args(&step.args)
        .current_dir(&step.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    for (k, v) in &step.env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn()?;
    let out = child.stdout.take().unwrap();
    let err = child.stderr.take().unwrap();
    let t1 = tokio::spawn(pump(out, tx.clone()));
    let t2 = tokio::spawn(pump(err, tx.clone()));

    let status = match tokio::time::timeout(step.timeout, child.wait()).await {
        Ok(s) => s?,
        Err(_) => {
            let _ = child.kill().await;
            let _ = t1.await;
            let _ = t2.await;
            let _ = tx
                .send(Bytes::from(format!(
                    "\n\x1b[31m⏱ {} 秒でタイムアウトしました (無限ループや入力待ちになっていませんか?)\x1b[0m\n",
                    step.timeout.as_secs()
                )))
                .await;
            return Ok(false);
        }
    };
    let _ = t1.await;
    let _ = t2.await;
    if !status.success() {
        let code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "signal".into());
        let _ = tx.send(Bytes::from(format!("\x1b[2m(終了コード: {code})\x1b[0m\n"))).await;
    }
    Ok(status.success())
}

async fn pump(mut r: impl AsyncReadExt + Unpin, tx: Tx) {
    let mut buf = vec![0u8; 8192];
    loop {
        match r.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if tx.send(Bytes::copy_from_slice(&buf[..n])).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// 単体ファイルを rustc でテストビルドする手順
pub fn rustc_test_steps(root: &std::path::Path, file: &std::path::Path, bin: &std::path::Path) -> Vec<Step> {
    let rel = file.strip_prefix(root).unwrap_or(file).display().to_string();
    let f = file.display().to_string();
    let b = bin.display().to_string();
    vec![
        Step::new(
            "rustc",
            &["--edition", "2024", "--test", "-A", "warnings", "--color", "always", &f, "-o", &b],
            root.to_path_buf(),
        )
        .timeout(120)
        .display(format!("rustc --edition 2024 --test {rel}")),
        Step::new(&b, &["--color", "always", "--test-threads", "1"], root.to_path_buf())
            .timeout(20)
            .display("テストを実行"),
    ]
}

/// 単体ファイルを rustc でビルドして実行する手順 (解説ページの ▶ 実行)
pub fn rustc_run_steps(root: &std::path::Path, file: &std::path::Path, bin: &std::path::Path) -> Vec<Step> {
    let f = file.display().to_string();
    let b = bin.display().to_string();
    vec![
        Step::new("rustc", &["--edition", "2024", "--color", "always", &f, "-o", &b], root.to_path_buf())
            .timeout(120)
            .display("rustc --edition 2024 main.rs"),
        Step::new(&b, &[], root.to_path_buf()).timeout(10).display("./main"),
    ]
}
