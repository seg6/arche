//! A terminal shell (crossterm): truecolor, two columns per cell.

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::{cursor, execute, terminal};

use super::action_for;
use super::text::{Cell, rasterize};
use crate::runtime::Runtime;

fn key_name(code: KeyCode) -> Option<String> {
    Some(match code {
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Enter => "return".into(),
        KeyCode::Esc => "escape".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::Backspace => "backspace".into(),
        KeyCode::F(n) => format!("f{n}"),
        KeyCode::Char(' ') => "space".into(),
        KeyCode::Char(c) => c.to_lowercase().to_string(),
        _ => return None,
    })
}

fn paint(grid: &[Vec<Cell>]) -> String {
    let mut out = String::new();
    let mut last = None;
    for (row, cells) in grid.iter().enumerate() {
        out += &format!("\x1b[{};1H", row + 1);
        for cell in cells {
            if last != Some((cell.fg, cell.bg)) {
                let (f, b) = (cell.fg, cell.bg);
                out += &format!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m", f.0, f.1, f.2, b.0, b.1, b.2);
                last = Some((cell.fg, cell.bg));
            }
            out.push(cell.ch);
        }
    }
    out + "\x1b[0m"
}

/// Run until the game quits or you press ctrl-c.
pub fn run(rt: &mut Runtime, keys: &[(&str, &str)], fps: u32) -> io::Result<()> {
    let (w, h) = rt.app().size;
    let (cols, rows) = terminal::size()?;
    if (cols as i32) < 2 * w || (rows as i32) < h {
        return Err(io::Error::other(format!(
            "this game wants a {}x{h} terminal, yours is {cols}x{rows}",
            2 * w
        )));
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let result = (|| -> io::Result<()> {
        let frame_time = Duration::from_secs_f64(1.0 / fps as f64);
        let (mut last, mut shown) = (Instant::now(), String::new());
        while rt.running() {
            while event::poll(Duration::ZERO)? {
                let Event::Key(key) = event::read()? else { continue };
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(());
                }
                if let Some(action) = key_name(key.code).and_then(|k| action_for(keys, &k)) {
                    rt.send(action);
                }
            }

            let now = Instant::now();
            rt.advance((now - last).as_secs_f64());
            last = now;

            let frame = paint(&rasterize(&rt.view(), (w, h)));
            if frame != shown {
                stdout.write_all(frame.as_bytes())?;
                stdout.flush()?;
                shown = frame;
            }
            std::thread::sleep(frame_time.saturating_sub(now.elapsed()));
        }
        Ok(())
    })();

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    result
}
