//! libc の termios / poll を使う端末イベントループ。外部 TUI クレートは使わない。
use f7_file_manager::{Buffer, FileManager};
use std::io::{self, Write};
use std::sync::Mutex;
use std::time::Duration;

static ORIGINAL: Mutex<Option<libc::termios>> = Mutex::new(None);
fn restore() {
    let _ = io::stdout().write_all(b"\x1b[0m\x1b[?25h\x1b[?1049l");
    let _ = io::stdout().flush();
    if let Ok(mut saved) = ORIGINAL.lock() {
        if let Some(term) = saved.take() { unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &term); } }
    }
}
struct Screen;
impl Drop for Screen { fn drop(&mut self) { restore(); } }
fn screen() -> io::Result<Screen> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| { restore(); hook(info); }));
    let mut old = unsafe { std::mem::zeroed::<libc::termios>() };
    if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut old) } != 0 { return Err(io::Error::last_os_error()); }
    let mut raw = old;
    raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::ISIG | libc::IEXTEN);
    raw.c_iflag &= !(libc::ICRNL | libc::IXON);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
    if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw) } != 0 { return Err(io::Error::last_os_error()); }
    *ORIGINAL.lock().unwrap() = Some(old);
    io::stdout().write_all(b"\x1b[?1049h\x1b[?25l\x1b[2J")?;
    io::stdout().flush()?;
    Ok(Screen)
}
fn size() -> (u16, u16) {
    let mut ws = unsafe { std::mem::zeroed::<libc::winsize>() };
    if unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        (ws.ws_col, ws.ws_row)
    } else { (80, 24) }
}
fn read_key() -> io::Result<Option<u8>> {
    let mut fd = libc::pollfd { fd: libc::STDIN_FILENO, events: libc::POLLIN, revents: 0 };
    let count = unsafe { libc::poll(&mut fd, 1, Duration::from_millis(200).as_millis() as i32) };
    if count < 0 { let e = io::Error::last_os_error(); return if e.kind() == io::ErrorKind::Interrupted { Ok(None) } else { Err(e) }; }
    if count == 0 { return Ok(None); }
    let mut bytes = [0u8; 16];
    let n = unsafe { libc::read(libc::STDIN_FILENO, bytes.as_mut_ptr().cast(), bytes.len()) };
    if n < 0 { return Err(io::Error::last_os_error()); }
    if n == 0 { return Ok(Some(b'q')); }
    if bytes[0] == 0x1b && n >= 3 && bytes[1] == b'[' {
        return Ok(Some(match bytes[2] { b'A' => b'k', b'B' => b'j', b'D' => b'h', b'C' => b'\r', _ => 0 }));
    }
    Ok(Some(bytes[0]))
}
fn main() -> io::Result<()> {
    let path = std::env::args_os().nth(1).unwrap_or_else(|| ".".into());
    let mut app = FileManager::open(path)?;
    let _guard = screen()?;
    let mut previous = Buffer::new(0, 0);
    loop {
        let (w, h) = size();
        app.scroll(h.saturating_sub(2) as usize);
        let mut frame = Buffer::new(w, h);
        app.render(&mut frame);
        io::stdout().write_all(frame.diff(&previous).as_bytes())?;
        io::stdout().flush()?;
        previous = frame;
        match read_key()? {
            Some(b'q' | 3 | 4) => break,
            Some(b'j') => app.down(),
            Some(b'k') => app.up(),
            Some(b'h' | 0x7f) => if let Err(e) = app.parent() { app.message = e.to_string(); },
            Some(b'\r' | b'\n' | b'l') => if let Err(e) = app.enter() { app.message = e.to_string(); },
            Some(b'r') => if let Err(e) = app.refresh() { app.message = e.to_string(); },
            _ => {}
        }
    }
    Ok(())
}
