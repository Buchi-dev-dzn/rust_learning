// Rust 学習スタジオ — フロントエンド (ビルド不要の素の JS)
'use strict';

const Studio = (() => {
  // ------------------------------------------------------------ 共通
  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));
  const isDark = () => document.documentElement.dataset.theme !== 'light';

  function initCommon() {
    const t = $('#theme-toggle');
    if (t) t.onclick = () => {
      const next = isDark() ? 'light' : 'dark';
      document.documentElement.dataset.theme = next;
      localStorage.setItem('theme', next);
      for (const cm of editors) cm.setOption('theme', cmTheme());
      syncHlTheme();
    };
    $$('time[data-ts]').forEach(el => {
      const d = new Date(Number(el.dataset.ts) * 1000);
      el.textContent = d.toLocaleString('ja-JP', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' });
    });
    initCodeBlocks();
    syncHlTheme();
    $$('[data-copy]').forEach(b => b.onclick = () => copy(b.dataset.copy, b));
  }

  function syncHlTheme() {
    $$('pre.cm-hl').forEach(p => {
      p.classList.toggle('cm-s-material-darker', isDark());
      p.classList.toggle('cm-s-default', !isDark());
    });
  }

  function copy(text, btn) {
    navigator.clipboard?.writeText(text).then(() => {
      if (!btn) return;
      const old = btn.textContent; btn.textContent = 'コピーしました'; setTimeout(() => btn.textContent = old, 1200);
    });
  }

  // ------------------------------------------------------------ ANSI → HTML
  const PALETTE = ['#000000', '#cd3131', '#0dbc79', '#e5e510', '#2472c8', '#bc3fbc', '#11a8cd', '#e5e5e5',
    '#666666', '#f14c4c', '#23d18b', '#f5f543', '#3b8eea', '#d670d6', '#29b8db', '#ffffff'];
  function color256(n) {
    if (n < 16) return `var(--ansi-${n})`;
    if (n < 232) { n -= 16; const c = [0, 95, 135, 175, 215, 255]; return `rgb(${c[Math.floor(n / 36)]},${c[Math.floor(n / 6) % 6]},${c[n % 6]})`; }
    const g = 8 + (n - 232) * 10; return `rgb(${g},${g},${g})`;
  }
  const esc = s => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

  class AnsiRenderer {
    constructor(el) { this.el = el; this.pending = ''; this.st = {}; }
    style() {
      const s = this.st, css = [];
      let fg = s.fg, bg = s.bg;
      if (s.inv) [fg, bg] = [bg || 'var(--out-bg)', fg || 'var(--out-fg)'];
      if (fg) css.push(`color:${fg}`);
      if (bg) css.push(`background:${bg}`);
      if (s.bold) css.push('font-weight:700');
      if (s.dim) css.push('opacity:.65');
      if (s.italic) css.push('font-style:italic');
      if (s.under) css.push('text-decoration:underline');
      return css.join(';');
    }
    sgr(params) {
      const p = params.length ? params.split(';').map(x => Number(x || 0)) : [0];
      for (let i = 0; i < p.length; i++) {
        const n = p[i];
        if (n === 0) this.st = {};
        else if (n === 1) this.st.bold = true;
        else if (n === 2) this.st.dim = true;
        else if (n === 3) this.st.italic = true;
        else if (n === 4) this.st.under = true;
        else if (n === 7) this.st.inv = true;
        else if (n === 22) { this.st.bold = false; this.st.dim = false; }
        else if (n === 23) this.st.italic = false;
        else if (n === 24) this.st.under = false;
        else if (n === 27) this.st.inv = false;
        else if (n >= 30 && n <= 37) this.st.fg = `var(--ansi-${n - 30})`;
        else if (n >= 90 && n <= 97) this.st.fg = `var(--ansi-${n - 90 + 8})`;
        else if (n === 39) this.st.fg = null;
        else if (n >= 40 && n <= 47) this.st.bg = `var(--ansi-${n - 40})`;
        else if (n >= 100 && n <= 107) this.st.bg = `var(--ansi-${n - 100 + 8})`;
        else if (n === 49) this.st.bg = null;
        else if ((n === 38 || n === 48) && p[i + 1] === 5) { this.st[n === 38 ? 'fg' : 'bg'] = color256(p[i + 2]); i += 2; }
        else if ((n === 38 || n === 48) && p[i + 1] === 2) { this.st[n === 38 ? 'fg' : 'bg'] = `rgb(${p[i + 2]},${p[i + 3]},${p[i + 4]})`; i += 4; }
      }
    }
    write(text) {
      text = this.pending + text; this.pending = '';
      let html = '', i = 0, buf = '';
      const flush = () => {
        if (!buf) return;
        const css = this.style();
        html += css ? `<span style="${css}">${esc(buf)}</span>` : esc(buf);
        buf = '';
      };
      while (i < text.length) {
        const ch = text[i];
        if (ch === '\x1b' && text[i + 1] === ']') { // OSC (\x1b] ... BEL) 
          const end = text.indexOf('\x07', i);
          if (end < 0) { this.pending = text.slice(i); break; }
          flush(); i = end + 1; continue;
        }
        if (ch === '\x1b') {
          const m = /^\x1b\[([0-9;?]*)([A-Za-z])/.exec(text.slice(i, i + 32));
          if (!m) {
            if (text.length - i < 32 && !/[A-Za-z]/.test(text.slice(i + 2))) { this.pending = text.slice(i); break; }
            i++; continue;
          }
          flush();
          if (m[2] === 'm') this.sgr(m[1]);
          i += m[0].length; continue;
        }
        if (ch === '\r') { if (text[i + 1] !== '\n' && i + 1 < text.length) { /* 進捗表示の上書き: 改行に置換 */ buf += '\n'; } i++; continue; }
        if (ch < ' ' && ch !== '\n' && ch !== '\t') { i++; continue; } // SI(\x0f) などの制御文字は捨てる
        buf += ch; i++;
      }
      flush();
      if (html) {
        const stick = this.el.scrollHeight - this.el.scrollTop - this.el.clientHeight < 40;
        this.el.insertAdjacentHTML('beforeend', html);
        if (stick) this.el.scrollTop = this.el.scrollHeight;
      }
    }
  }

  /** サーバのストリーム API を呼び、出力を el に描画。結果 {ok} を返す */
  async function runStream(url, body, el) {
    el.hidden = false;
    el.innerHTML = '';
    el.classList.remove('ok', 'ng');
    el.classList.add('running');
    const r = new AnsiRenderer(el);
    let ok = false;
    try {
      const res = await fetch(url, { method: 'POST', body: body ?? '', headers: { 'content-type': 'text/plain; charset=utf-8' } });
      if (!res.ok) { r.write(`\x1b[31mHTTP ${res.status}\x1b[0m\n` + await res.text()); return { ok: false }; }
      const reader = res.body.getReader();
      const dec = new TextDecoder();
      let tail = '';
      for (;;) {
        const { value, done } = await reader.read();
        if (done) break;
        let text = tail + dec.decode(value, { stream: true });
        tail = '';
        const idx = text.lastIndexOf('\x1e');
        if (idx >= 0) {
          const control = text.slice(idx);
          const m = /^\x1e(\{"ok":(?:true|false)\})\n/.exec(control);
          if (m) {
            ok = JSON.parse(m[1]).ok;
          } else if (!control.includes('\n')) {
            tail = control; // JSON がネットワークのチャンク境界で分割されても保持する
          }
          text = text.slice(0, idx);
        }
        r.write(text);
      }
    } catch (e) {
      r.write(`\x1b[31m通信エラー: ${e}\x1b[0m\n(サーバが止まっていませんか？)`);
    } finally {
      el.classList.remove('running');
      el.classList.add(ok ? 'ok' : 'ng');
      r.write(ok ? '\n\x1b[1;32m✔ 成功\x1b[0m\n' : '\n\x1b[1;31m✘ 失敗\x1b[0m\n');
    }
    return { ok };
  }

  // ------------------------------------------------------------ 解説ページのコードブロック
  function initCodeBlocks() {
    $$('.codeblock').forEach(block => {
      const codeEl = $('code', block);
      const code = codeEl.textContent;
      if (window.CodeMirror?.runMode && block.dataset.lang === 'rust') {
        CodeMirror.runMode(code, 'rust', codeEl);
        $('pre', block).classList.add('cm-hl');
      }
      const c = $('.copy', block); if (c) c.onclick = () => copy(code, c);
      const run = $('.run', block);
      if (run) run.onclick = async () => {
        run.disabled = true;
        await runStream('/api/playground', code, $('.run-output', block));
        run.disabled = false;
      };
    });
  }

  // ------------------------------------------------------------ エディタ
  const editors = [];
  const cmTheme = () => isDark() ? 'material-darker' : 'default';
  function makeEditor(textarea, opts = {}) {
    if (!window.CodeMirror) return null;
    const cm = CodeMirror.fromTextArea(textarea, {
      mode: 'rust', theme: cmTheme(), lineNumbers: true, indentUnit: 4, tabSize: 4,
      indentWithTabs: false, matchBrackets: true, autoCloseBrackets: true, viewportMargin: opts.readOnly ? Infinity : 10,
      readOnly: !!opts.readOnly,
      extraKeys: {
        Tab: cm => cm.somethingSelected() ? cm.indentSelection('add') : cm.replaceSelection('    ', 'end'),
        'Shift-Tab': cm => cm.indentSelection('subtract'),
        ...(opts.keys || {}),
      },
    });
    editors.push(cm);
    return cm;
  }

  function readonlyCode() {
    $$('textarea[data-readonly]').forEach(t => makeEditor(t, { readOnly: true }));
  }

  function hintsAndSolution() {
    const more = $('#more-hint');
    if (more) more.onclick = () => {
      const hidden = $$('.hint[hidden]');
      if (hidden.length) { hidden[0].hidden = false; hidden[0].open = true; }
      if (hidden.length <= 1) more.hidden = true;
    };
    $$('.hint').forEach((h, i) => { if (i === 0) h.addEventListener('toggle', () => {}); });
    const sol = $('#solution-link');
    if (sol) sol.onclick = e => { if (!confirm('模範解答を見ますか？\nまずはヒントを全部見てから、もう少しだけ粘ってみるのもおすすめです。')) e.preventDefault(); };
  }

  /** SSE でファイル変更を購読 */
  function onFileChange(cb) {
    const es = new EventSource('/api/events');
    es.addEventListener('change', e => cb(e.data));
    return es;
  }

  function setStatus(ok) {
    const s = $('#ex-status');
    if (ok && s) { s.textContent = '✅ 合格済み'; s.classList.add('ok'); }
    const b = $('#pass-banner');
    if (ok && b) b.hidden = false;
  }

  // ------------------------------------------------------------ 演習ページ
  function exercisePage(cfg) {
    hintsAndSolution();
    const out = $('#output');
    const dirtyEl = $('#dirty');
    let running = false;
    let saved = $('#code').value;
    const cm = makeEditor($('#code'), {
      keys: { 'Ctrl-S': () => save(), 'Cmd-S': () => save(), 'Ctrl-Enter': () => test(), 'Cmd-Enter': () => test() },
    });
    const get = () => cm ? cm.getValue() : $('#code').value;
    const set = v => { cm ? cm.setValue(v) : ($('#code').value = v); saved = v; markDirty(); };
    const markDirty = () => { dirtyEl.hidden = get() === saved; };
    cm?.on('change', markDirty);

    async function save() {
      const v = get();
      const r = await fetch(cfg.sourceUrl, { method: 'PUT', body: v });
      if (r.ok) { saved = v; markDirty(); }
    }
    async function test() {
      if (running) return;
      running = true; $('#btn-test').disabled = true;
      const v = get();
      saved = v; markDirty();
      const { ok } = await runStream(cfg.testUrl, v, out);
      setStatus(ok);
      running = false; $('#btn-test').disabled = false;
    }
    $('#btn-test').onclick = test;
    $('#btn-save').onclick = save;
    $('#btn-reset').onclick = async () => {
      if (!confirm('演習ファイルを初期状態に戻します。あなたの変更は消えます。よろしいですか？')) return;
      const r = await fetch(cfg.resetUrl, { method: 'POST' });
      if (r.ok) set(await r.text());
    };
    window.addEventListener('beforeunload', e => { if (get() !== saved) { e.preventDefault(); e.returnValue = ''; } });

    onFileChange(async path => {
      if (path !== cfg.file || running) return;
      const text = await (await fetch(cfg.sourceUrl)).text();
      if (text === get()) return;
      if (get() !== saved) return; // ブラウザ側に未保存の変更があるときは上書きしない
      set(text);
      if ($('#auto').checked) test();
    });
  }

  // ------------------------------------------------------------ 複数ファイル編集 (卒業課題 / starter)
  function multiFileEditor({ select, textarea, dirtyEl, onSaved }) {
    let current = select.value;
    let saved = '';
    const cm = makeEditor(textarea, {
      keys: { 'Ctrl-S': () => save(), 'Cmd-S': () => save() },
    });
    const get = () => cm ? cm.getValue() : textarea.value;
    const set = v => { cm ? cm.setValue(v) : (textarea.value = v); saved = v; mark(); cm?.clearHistory(); };
    const mark = () => { if (dirtyEl) dirtyEl.hidden = get() === saved; };
    cm?.on('change', mark);
    const url = p => '/api/files?path=' + encodeURIComponent(p);
    async function load(p) {
      const r = await fetch(url(p));
      set(r.ok ? await r.text() : `// 読み込めませんでした: ${p}`);
    }
    async function save() {
      if (get() === saved) return true;
      const v = get();
      const r = await fetch(url(current), { method: 'PUT', body: v });
      if (r.ok) { saved = v; mark(); onSaved?.(); }
      return r.ok;
    }
    select.onchange = async () => {
      const next = select.value;
      select.value = current;
      await save();
      current = next; select.value = next;
      load(current);
    };
    if (current) load(current);
    return {
      save, reload: () => load(current),
      get current() { return current; },
      isDirty: () => get() !== saved,
      refreshIfClean: async p => {
        if (p !== current || get() !== saved) return false;
        const r = await fetch(url(p)); const t = await r.text();
        if (t === get()) return false;
        set(t); return true;
      },
      refresh: () => cm?.refresh(),
    };
  }

  function finalPage(cfg) {
    hintsAndSolution();
    const out = $('#output');
    const ed = multiFileEditor({ select: $('#file-select'), textarea: $('#code'), dirtyEl: $('#dirty') });
    let running = false;
    async function cargo(action) {
      if (running) return;
      running = true;
      await ed.save();
      const { ok } = await runStream(`/api/cargo/final/${cfg.id}/${action}`, '', out);
      if (action === 'test') setStatus(ok);
      running = false;
    }
    $('#btn-test').onclick = () => cargo('test');
    $('#btn-build').onclick = () => cargo('build');
    $('#btn-save').onclick = () => ed.save();
    $('#btn-reset').onclick = async () => {
      if (!confirm('src/ のファイルを初期状態 (starter) に戻します。あなたの変更は消えます。よろしいですか？')) return;
      const r = await fetch(`/api/finals/${cfg.id}/reset`, { method: 'POST' });
      alert(await r.text());
      ed.reload();
    };
    document.addEventListener('keydown', e => { if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); cargo('test'); } });
    onFileChange(async p => {
      if (!p.startsWith(`finals/${cfg.id}/`) || running) return;
      await ed.refreshIfClean(p);
      if ($('#auto').checked && !ed.isDirty()) cargo('test');
    });
  }

  // ------------------------------------------------------------ プロジェクトページ
  function toolsPage() {
    const out = $('#output');
    let projectEditor = null;
    let running = false;
    $$('[data-cargo]').forEach(b => b.onclick = async () => {
      if (running) return;
      running = true; $$('[data-cargo]').forEach(x => x.disabled = true);
      if (projectEditor) await projectEditor.save();
      await runStream(b.dataset.cargo, '', out);
      running = false; $$('[data-cargo]').forEach(x => x.disabled = false);
    });
    const step = $('#step-done');
    if (step) step.onchange = () => fetch(step.dataset.url + '?done=' + (step.checked ? 1 : 0), { method: 'POST' }).then(() => {
      const li = $('.steps-nav li.current'); if (li) li.classList.toggle('done', step.checked);
    });
    const fe = $('.file-editor');
    if (fe) {
      const files = fe.dataset.files.split(',').filter(Boolean);
      fe.innerHTML = `<div class="ide-bar"><select>${files.map(f => `<option>${esc(f)}</option>`).join('')}</select>
        <span class="dirty" hidden>● 未保存</span><span class="spacer"></span><button class="btn tiny" type="button">保存</button></div>
        <div class="editor-wrap short"><textarea></textarea></div>`;
      const ed = multiFileEditor({ select: $('select', fe), textarea: $('textarea', fe), dirtyEl: $('.dirty', fe) });
      projectEditor = ed;
      $('button', fe).onclick = () => ed.save();
      fe.closest('details')?.addEventListener('toggle', () => ed.refresh());
      onFileChange(p => ed.refreshIfClean(p));
    }
  }

  // ------------------------------------------------------------ 解答ページ
  function lineDiff(a, b) {
    a = a.split('\n'); b = b.split('\n');
    const n = a.length, m = b.length;
    const dp = Array.from({ length: n + 1 }, () => new Uint16Array(m + 1));
    for (let i = n - 1; i >= 0; i--) for (let j = m - 1; j >= 0; j--)
      dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    const out = []; let i = 0, j = 0;
    while (i < n && j < m) {
      if (a[i] === b[j]) { out.push([' ', a[i]]); i++; j++; }
      else if (dp[i + 1][j] >= dp[i][j + 1]) out.push(['-', a[i++]]);
      else out.push(['+', b[j++]]);
    }
    while (i < n) out.push(['-', a[i++]]);
    while (j < m) out.push(['+', b[j++]]);
    return out;
  }
  function solutionPage() {
    readonlyCode();
    const diff = lineDiff($('#mine').value, $('#sol').value);
    $('#diff').innerHTML = '<div class="legend"><span class="del">- あなたのコード</span> <span class="add">+ 模範解答</span></div>' +
      diff.map(([k, l]) => `<div class="${k === '+' ? 'add' : k === '-' ? 'del' : ''}">${k} ${esc(l)}</div>`).join('');
    $$('#sol-tabs [data-tab]').forEach(b => b.onclick = () => {
      $$('#sol-tabs [data-tab]').forEach(x => x.classList.toggle('active', x === b));
      $('#tab-solution').hidden = b.dataset.tab !== 'solution';
      $('#tab-diff').hidden = b.dataset.tab !== 'diff';
      editors.forEach(e => e.refresh());
    });
  }

  // ------------------------------------------------------------ ターミナル
  function terminalPage(cfg) {
    const el = $('#term');
    const css = getComputedStyle(document.documentElement);
    const term = new Terminal({
      cursorBlink: true, fontSize: 14, scrollback: 5000, allowProposedApi: true,
      fontFamily: '"JetBrains Mono", "SF Mono", Menlo, Consolas, "Hiragino Sans", monospace',
      theme: { background: css.getPropertyValue('--term-bg').trim() || '#0b0e14', foreground: '#e6e6e6' },
    });
    const fit = new FitAddon.FitAddon();
    term.loadAddon(fit);
    term.open(el);
    fit.fit();
    let ws = null;
    const send = obj => { if (ws && ws.readyState === 1) ws.send(JSON.stringify(obj)); };
    function connect(cmd) {
      if (ws) { ws.onclose = null; ws.close(); }
      term.reset();
      const q = new URLSearchParams({ cols: term.cols, rows: term.rows });
      if (cmd) q.set('cmd', cmd);
      ws = new WebSocket(`ws://${location.host}/api/pty?${q}`);
      ws.binaryType = 'arraybuffer';
      ws.onmessage = e => term.write(typeof e.data === 'string' ? e.data : new Uint8Array(e.data));
      ws.onclose = () => term.write('\r\n\x1b[2m[切断されました — 「再起動」で再接続]\x1b[0m\r\n');
      term.focus();
    }
    term.onData(d => send({ type: 'input', data: d }));
    term.onResize(({ cols, rows }) => send({ type: 'resize', cols, rows }));
    new ResizeObserver(() => { try { fit.fit(); } catch (_) {} }).observe(el);
    $('#term-restart').onclick = () => connect('');
    $('#term-ctrlc').onclick = () => { send({ type: 'input', data: '\x03' }); term.focus(); };
    $('#preset').onchange = e => {
      const c = e.target.value; e.target.value = '';
      if (c) { send({ type: 'input', data: '\x03' }); setTimeout(() => send({ type: 'input', data: c + '\r' }), 150); term.focus(); }
    };
    connect(cfg.cmd);
  }

  document.addEventListener('DOMContentLoaded', initCommon);
  return { exercisePage, finalPage, toolsPage, solutionPage, terminalPage, readonlyCode, runStream };
})();
