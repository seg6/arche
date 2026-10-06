//! crates: a tiny Sokoban built on arche.
//!
//!     cargo run --release                              # a window
//!     cargo run --release -- --shell terminal          # the same game, in your terminal
//!     cargo run --release -- --record run.txt          # save your session (terminal shell)
//!     cargo run --release -- --replay run.txt          # watch it back, then take over
//!     cargo run --release -- --replay run.txt --shell text   # headless, print the last frame

mod art;
mod board;
mod levels;
mod states;

use arche::shells::{DEFAULT_KEYS, text};
use arche::{Config, Runtime};

use states::{H, Title, W};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let shell = flag("--shell").unwrap_or_else(|| "window".into());

    let (seed, replay) = match flag("--replay") {
        Some(path) => {
            let text = std::fs::read_to_string(&path).expect("can't read the replay");
            let (seed, log) = Runtime::load_log(&text).expect("not a replay file");
            (seed, Some(log))
        }
        None => (0, None),
    };
    let mut rt = Runtime::new(Title::new(), Config { size: (W, H), seed, replay, ..Config::default() });

    match shell.as_str() {
        "text" => {
            let (mut frame, mut stack) = (String::new(), vec![]);
            while rt.running() && rt.replaying() {
                frame = text::to_text(&rt.view(), (W, H));
                stack = rt.stack_names();
                rt.tick();
            }
            if rt.running() {
                rt.run_headless(90); // let the last input play out
                frame = text::to_text(&rt.view(), (W, H));
                stack = rt.stack_names();
            }
            println!("{frame}\n\ntick {}, stack: {stack:?}", rt.app().ticks);
        }
        #[cfg(feature = "terminal")]
        "terminal" => {
            if let Err(e) = arche::shells::terminal::run(&mut rt, DEFAULT_KEYS, 30) {
                eprintln!("{e}");
            }
        }
        #[cfg(feature = "window")]
        "window" => {
            let record = flag("--record");
            // a scripted walkthrough that saves screenshots
            let mut tour = flag("--tour").map(|dir| Tour { dir, step: 0, since: 0 });
            arche::shells::window::run(rt, "crates", 36.0, DEFAULT_KEYS, move |rt, frame| {
                if let Some(tour) = &mut tour {
                    tour.step(rt);
                }
                if let Some(path) = &record
                    && (!rt.running() || frame.is_multiple_of(60))
                {
                    let _ = std::fs::write(path, rt.save_log());
                }
            });
            return;
        }
        other => {
            eprintln!("unknown or disabled shell: {other} (try window, terminal or text)");
            std::process::exit(2);
        }
    }

    if let Some(path) = flag("--record") {
        std::fs::write(&path, rt.save_log()).expect("can't write the replay");
        println!("saved {} inputs to {path}", rt.log().len());
    }
}

/// When a tour step fires: some ticks after the previous step, or once a state is on top.
#[cfg(feature = "window")]
enum When {
    After(u64),
    Top(&'static str),
}

/// What it does: press something, save a screenshot, or nothing (just wait).
#[cfg(feature = "window")]
enum Do {
    Send(&'static str),
    Shot(&'static str),
    Nothing,
}

/// A scripted walkthrough of the game, in game time, saving screenshots.
#[cfg(feature = "window")]
const TOUR: &[(When, Do)] = {
    use {Do::*, When::*};
    &[
        (After(90), Shot("1-title")),
        (After(1), Send("confirm")),
        (Top("Say"), Nothing),
        (After(100), Shot("2-intro")),
        (After(1), Send("confirm")),
        (Top("Say"), Nothing),
        (After(100), Send("confirm")),
        (Top("Say"), Nothing),
        (After(80), Send("confirm")),
        (Top("LevelSelect"), Nothing),
        (After(20), Send("confirm")),
        (After(30), Shot("3-banner")),
        (Top("Play"), Nothing),
        (After(10), Send("right")),
        (After(10), Send("right")),
        (After(10), Send("right")),
        (Top("Win"), Nothing),
        (After(70), Shot("4-win")),
        (After(1), Send("confirm")),
        (Top("Play"), Nothing),
        (After(10), Send("up")),
        (After(10), Send("back")),
        (After(5), Shot("5-pause")),
        (After(2), Send("down")),
        (After(2), Send("down")),
        (After(2), Send("confirm")),
        (After(90), Send("debug")),
        (After(1), Shot("6-robot-debug")),
        (After(1), Send("debug")),
        (After(20), Send("back")),
        (After(10), Send("back")),
        (After(2), Send("down")),
        (After(2), Send("down")),
        (After(2), Send("down")),
        (After(2), Send("confirm")),
        (After(30), Shot("7-select")),
        (After(10), Send("back")),
        (After(10), Send("back")),
    ]
};

#[cfg(feature = "window")]
struct Tour {
    dir: String,
    step: usize,
    since: u64,
}

#[cfg(feature = "window")]
impl Tour {
    fn step(&mut self, rt: &mut Runtime) {
        let Some((when, what)) = TOUR.get(self.step) else { return };
        let ticks = rt.app().ticks;
        if self.step > 0 && ticks == self.since {
            return; // one step per tick, so a sent action lands before the next check
        }
        let ready = match when {
            When::After(n) => ticks >= self.since + n,
            When::Top(name) => rt.stack_names().last() == Some(name),
        };
        if !ready {
            return;
        }
        match what {
            Do::Send(action) => rt.send(action),
            Do::Shot(name) => arche::shells::window::screenshot(&format!("{}/{name}.png", self.dir)),
            Do::Nothing => {}
        }
        self.step += 1;
        self.since = ticks;
    }
}
