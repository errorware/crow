//! One terminal session (ERR-93): a PTY running the server's transport, an
//! alacritty `Term` fed by its event loop thread, and a snapshot of the
//! screen as styled runs for the renderer.

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, EventLoopSender, Msg};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::tty;
use alacritty_terminal::vte::ansi::CursorShape;

use super::palette;

/// The grid in cells, and a cell's size in pixels (programs may ask).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSize {
    pub cols: u16,
    pub rows: u16,
    pub cell_w: u16,
    pub cell_h: u16,
}

impl Dimensions for GridSize {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }
    fn screen_lines(&self) -> usize {
        self.rows as usize
    }
    fn columns(&self) -> usize {
        self.cols as usize
    }
}

impl GridSize {
    fn window(self) -> WindowSize {
        WindowSize { num_lines: self.rows, num_cols: self.cols, cell_width: self.cell_w, cell_height: self.cell_h }
    }
}

/// What to run in the PTY.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone)]
struct Listener(mpsc::Sender<Event>);

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        let _ = self.0.send(event);
    }
}

/// A run of cells with one style, starting at `col` and `cols` wide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub col: usize,
    pub cols: usize,
    pub text: String,
    pub fg: u32,
    /// `None` for the default background.
    pub bg: Option<u32>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}

/// The screen as the renderer needs it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub rows: Vec<Vec<Run>>,
    /// (row, col, shape); `None` when hidden or scrolled out of view.
    pub cursor: Option<(usize, usize, CursorShape)>,
    /// Lines scrolled back from the bottom.
    pub scrolled: usize,
}

pub struct Session {
    term: Arc<FairMutex<Term<Listener>>>,
    sender: EventLoopSender,
    events: mpsc::Receiver<Event>,
    size: GridSize,
    pub title: Option<String>,
    /// Why the session ended, once it has.
    pub exited: Option<String>,
    pub bell: bool,
}

impl Session {
    pub fn spawn(launch: &Launch, size: GridSize) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let config = Config { scrolling_history: 10_000, ..Default::default() };
        let term = Arc::new(FairMutex::new(Term::new(config, &size, Listener(tx.clone()))));
        let options = tty::Options {
            shell: Some(tty::Shell::new(launch.program.clone(), launch.args.clone())),
            working_directory: None,
            drain_on_exit: true,
            env: HashMap::from([("TERM".to_string(), "xterm-256color".to_string()), ("COLORTERM".to_string(), "truecolor".to_string())]),
            ..Default::default()
        };
        let pty = tty::new(&options, size.window(), 0)?;
        let event_loop = EventLoop::new(term.clone(), Listener(tx), pty, options.drain_on_exit, false)?;
        let sender = event_loop.channel();
        event_loop.spawn();
        Ok(Self { term, sender, events: rx, size, title: None, exited: None, bell: false })
    }

    pub fn write(&self, bytes: Vec<u8>) {
        if self.exited.is_none() {
            let _ = self.sender.send(Msg::Input(bytes.into()));
        }
    }

    pub fn size(&self) -> GridSize {
        self.size
    }

    pub fn resize(&mut self, size: GridSize) {
        if size == self.size || size.cols == 0 || size.rows == 0 {
            return;
        }
        self.size = size;
        let _ = self.sender.send(Msg::Resize(size.window()));
        self.term.lock().resize(size);
    }

    pub fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    pub fn scroll(&self, lines: i32) {
        self.term.lock().scroll_display(Scroll::Delta(lines));
    }

    /// Back to the live screen (typing does this).
    pub fn scroll_to_bottom(&self) {
        self.term.lock().scroll_display(Scroll::Bottom);
    }

    /// Handles what the event loop reported; true when the screen changed.
    pub fn pump(&mut self) -> bool {
        let mut dirty = false;
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Wakeup | Event::MouseCursorDirty | Event::CursorBlinkingChange => dirty = true,
                Event::Title(t) => {
                    self.title = Some(t);
                    dirty = true;
                }
                Event::ResetTitle => self.title = None,
                Event::Bell => self.bell = true,
                // Replies the terminal owes the program (device status,
                // cursor position, colors, size).
                Event::PtyWrite(s) => self.write(s.into_bytes()),
                Event::ColorRequest(i, format) => {
                    let colors = *self.term.lock().colors();
                    let c = colors[i].unwrap_or_else(|| alacritty_terminal::vte::ansi::Rgb { r: 0, g: 0, b: 0 });
                    self.write(format(c).into_bytes());
                }
                Event::TextAreaSizeRequest(format) => self.write(format(self.size.window()).into_bytes()),
                Event::ChildExit(status) => {
                    self.exited = Some(match status.code() {
                        Some(0) => "session ended".to_string(),
                        Some(code) => format!("session ended (exit status {code})"),
                        None => "session ended (killed by a signal)".to_string(),
                    });
                    dirty = true;
                }
                Event::Exit => {
                    self.exited.get_or_insert_with(|| "session ended".to_string());
                    dirty = true;
                }
                // Clipboard access from programs (OSC 52) is refused.
                Event::ClipboardStore(..) | Event::ClipboardLoad(..) => {}
            }
        }
        dirty
    }

    /// The visible screen as styled runs.
    pub fn frame(&self) -> Frame {
        let term = self.term.lock();
        frame_of(&term)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.sender.send(Msg::Shutdown);
    }
}

fn frame_of<T: EventListener>(term: &Term<T>) -> Frame {
    let content = term.renderable_content();
    let colors = content.colors;
    let offset = content.display_offset as i32;
    let rows_n = term.screen_lines();
    let mut rows: Vec<Vec<Run>> = vec![Vec::new(); rows_n];
    for cell in content.display_iter {
        let row = (cell.point.line.0 + offset) as usize;
        if row >= rows_n || cell.flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }
        let bold = cell.flags.contains(Flags::BOLD);
        let mut fg = palette::resolve(cell.fg, colors, bold);
        let mut bg = palette::resolve(cell.bg, colors, false);
        if cell.flags.contains(Flags::INVERSE) {
            std::mem::swap(&mut fg, &mut bg);
        }
        if cell.flags.contains(Flags::DIM) {
            fg = dim(fg);
        }
        let hidden = cell.flags.contains(Flags::HIDDEN);
        let bg = (bg != palette::to_u32(colors[alacritty_terminal::vte::ansi::NamedColor::Background].unwrap_or(palette::indexed(0))) && bg != palette::BACKGROUND).then_some(bg);
        let wide = cell.flags.contains(Flags::WIDE_CHAR);
        let style = (fg, bg, bold, cell.flags.contains(Flags::ITALIC), cell.flags.intersects(Flags::ALL_UNDERLINES), cell.flags.contains(Flags::STRIKEOUT));
        let col = cell.point.column.0;
        let line = &mut rows[row];
        let c = if cell.c == '\0' || hidden { ' ' } else { cell.c };
        // Wide characters get a run of their own, so the next run starts
        // on its real column.
        match line.last_mut() {
            Some(r) if !wide && r.col + r.cols == col && (r.fg, r.bg, r.bold, r.italic, r.underline, r.strike) == style && !r.text.chars().any(is_wide) => {
                r.text.push(c);
                r.cols += 1;
            }
            _ => line.push(Run { col, cols: if wide { 2 } else { 1 }, text: c.to_string(), fg, bg, bold, italic: style.3, underline: style.4, strike: style.5 }),
        }
        // Combining characters ride on the base character.
        if let (Some(marks), Some(r)) = (cell.zerowidth(), rows[row].last_mut()) {
            r.text.extend(marks.iter());
        }
    }
    // Blanks on the default background draw nothing: trim them off the end
    // of narrow runs, and drop runs that are only blanks.
    for line in &mut rows {
        for r in line.iter_mut().filter(|r| r.bg.is_none() && !r.underline && !r.strike && !r.text.chars().any(is_wide)) {
            let kept = r.text.trim_end_matches(' ').len();
            r.cols -= r.text.len() - kept;
            r.text.truncate(kept);
        }
        line.retain(|r| !r.text.is_empty());
    }
    let cursor = (content.cursor.shape != CursorShape::Hidden && content.display_offset == 0).then(|| {
        let row = (content.cursor.point.line.0 + offset) as usize;
        (row, content.cursor.point.column.0, content.cursor.shape)
    });
    Frame { rows, cursor, scrolled: content.display_offset }
}

fn is_wide(c: char) -> bool {
    unicode_width_2(c)
}

/// Wide (two-cell) characters, by the common East Asian/emoji ranges; the
/// grid already marks them, this only keeps a narrow run from absorbing one.
fn unicode_width_2(c: char) -> bool {
    matches!(c as u32, 0x1100..=0x115F | 0x2E80..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6 | 0x1F300..=0x1F64F | 0x1F900..=0x1F9FF | 0x20000..=0x3FFFD)
}

fn dim(c: u32) -> u32 {
    let f = |v: u32| (v * 2 / 3) & 0xff;
    f(c >> 16) << 16 | f((c >> 8) & 0xff) << 8 | f(c & 0xff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};

    fn screen(bytes: &[u8]) -> Frame {
        let size = GridSize { cols: 20, rows: 4, cell_w: 8, cell_h: 16 };
        let mut term = Term::new(Config::default(), &size, VoidListener);
        let mut parser: Processor<StdSyncHandler> = Processor::new();
        parser.advance(&mut term, bytes);
        frame_of(&term)
    }

    #[test]
    fn text_colors_and_cursor() {
        let f = screen(b"hi \x1b[1;31mred\x1b[0m\r\n\x1b[44mbg\x1b[0m");
        let row0: Vec<_> = f.rows[0].iter().map(|r| (r.col, r.text.as_str(), r.bold)).collect();
        assert_eq!(row0, vec![(0, "hi", false), (3, "red", true)], "blanks trimmed, columns kept");
        assert_eq!(f.rows[0][1].fg, 0xfca5a5, "bold red is bright red");
        assert_eq!(f.rows[1][0].bg, Some(0x60a5fa), "blue background kept");
        assert_eq!(f.cursor, Some((1, 2, CursorShape::Block)));
    }

    #[test]
    fn wide_characters_keep_their_columns() {
        let f = screen("a漢b".as_bytes());
        let row0: Vec<_> = f.rows[0].iter().map(|r| (r.col, r.cols, r.text.as_str())).collect();
        assert_eq!(row0, vec![(0, 1, "a"), (1, 2, "漢"), (3, 1, "b")]);
    }

    #[test]
    fn inverse_and_hidden() {
        let f = screen(b"\x1b[7mX\x1b[0m\x1b[8mY");
        assert_eq!((f.rows[0][0].fg, f.rows[0][0].bg), (palette::BACKGROUND, Some(palette::FOREGROUND)));
        assert_eq!(f.rows[0].len(), 1, "hidden text on the default background draws nothing visible");
    }
}
