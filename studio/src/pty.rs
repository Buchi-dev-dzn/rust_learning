//! ブラウザ内ターミナル: WebSocket ⇔ 疑似端末 (PTY) の中継。
//!
//! クライアント → サーバ: テキストフレームの JSON
//!   {"type":"input","data":"..."} / {"type":"resize","cols":80,"rows":24}
//! サーバ → クライアント: バイナリフレーム (PTY の出力そのもの)

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::Deserialize;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMsg {
    Input { data: String },
    Resize { cols: u16, rows: u16 },
}

pub async fn session(socket: WebSocket, root: PathBuf, cmd: Option<String>, cols: u16, rows: u16) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    let size = PtySize { rows: rows.max(5), cols: cols.max(20), pixel_width: 0, pixel_height: 0 };
    let pair = match native_pty_system().openpty(size) {
        Ok(p) => p,
        Err(e) => {
            let _ = ws_tx.send(Message::Text(format!("PTY を開けませんでした: {e}\r\n").into())).await;
            return;
        }
    };
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let mut builder = CommandBuilder::new(&shell);
    builder.cwd(&root);
    builder.env("TERM", "xterm-256color");
    builder.env("COLORTERM", "truecolor");
    builder.env("RUST_LEARNING_STUDIO", "1");
    let mut child = match pair.slave.spawn_command(builder) {
        Ok(c) => c,
        Err(e) => {
            let _ = ws_tx.send(Message::Text(format!("シェルを起動できませんでした: {e}\r\n").into())).await;
            return;
        }
    };
    drop(pair.slave);
    let master = pair.master;
    let mut reader = match master.try_clone_reader() {
        Ok(r) => r,
        Err(_) => return,
    };
    let mut writer = match master.take_writer() {
        Ok(w) => w,
        Err(_) => return,
    };

    // PTY 出力 → チャネル (ブロッキング読み込みは専用スレッドで)
    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if out_tx.blocking_send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // 起動コマンドがあればシェルに打ち込む
    if let Some(c) = cmd.filter(|c| !c.trim().is_empty()) {
        let _ = writer.write_all(format!("{}\r", c.trim()).as_bytes());
        let _ = writer.flush();
    }

    loop {
        tokio::select! {
            out = out_rx.recv() => {
                match out {
                    Some(bytes) => {
                        if ws_tx.send(Message::Binary(bytes.into())).await.is_err() { break; }
                    }
                    None => {
                        let _ = ws_tx.send(Message::Text("\r\n[シェルが終了しました]\r\n".into())).await;
                        break;
                    }
                }
            }
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        match serde_json::from_str::<ClientMsg>(t.as_str()) {
                            Ok(ClientMsg::Input { data }) => {
                                let _ = writer.write_all(data.as_bytes());
                                let _ = writer.flush();
                            }
                            Ok(ClientMsg::Resize { cols, rows }) => {
                                let _ = master.resize(PtySize { rows: rows.max(5), cols: cols.max(20), pixel_width: 0, pixel_height: 0 });
                            }
                            Err(_) => {}
                        }
                    }
                    Some(Ok(Message::Binary(b))) => {
                        let _ = writer.write_all(&b);
                        let _ = writer.flush();
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}
