//! The runtime: a fixed-timestep, deterministic driver for the stack.
//!
//! Shells feed it actions and real elapsed time, and ask it what to show.
//! Input is applied on tick boundaries and logged as (tick, action) pairs, so
//! a run is fully described by its seed and its log: that's a replay.

use std::cell::{Ref, RefCell, RefMut};
use std::collections::VecDeque;
use std::rc::Rc;

use crate::effect::{Effect, spawner};
use crate::rng::Rng;
use crate::scene::{self, Node, hex};
use crate::stack::{Stack, Wait};
use crate::state::State;
use crate::store::Store;

/// What every hook can reach: time, randomness, and shared game data.
pub struct App {
    pub size: (i32, i32),
    pub step: f64,
    pub ticks: u64,
    /// Use this, not a global RNG, to stay replayable.
    pub rng: Rng,
    /// Typed shared data: `app.store.get::<Progress>()`.
    pub store: Store,
}

pub type Log = Vec<(u64, String)>;

#[derive(Clone, Debug)]
pub struct Config {
    pub size: (i32, i32),
    pub step: f64,
    pub seed: u64,
    pub replay: Option<Log>,
}

impl Default for Config {
    fn default() -> Self {
        Config { size: (40, 22), step: 1.0 / 60.0, seed: 0, replay: None }
    }
}

pub struct Runtime {
    app: Rc<RefCell<App>>,
    stack: Stack,
    seed: u64,
    log: Log,
    replay: VecDeque<(u64, String)>,
    queue: Vec<String>,
    acc: f64,
    pub debug: bool,
}

impl Runtime {
    pub fn new(root: impl State, config: Config) -> Self {
        let app = App {
            size: config.size,
            step: config.step,
            ticks: 0,
            rng: Rng::new(config.seed),
            store: Store::default(),
        };
        let mut rt = Runtime {
            app: Rc::new(RefCell::new(app)),
            stack: Stack::default(),
            seed: config.seed,
            log: Vec::new(),
            replay: config.replay.unwrap_or_default().into(),
            queue: Vec::new(),
            acc: 0.0,
            debug: false,
        };
        rt.stack.apply_one(&rt.app, Effect::Push(spawner(root)));
        rt
    }

    pub fn app(&self) -> Ref<'_, App> {
        self.app.borrow()
    }

    pub fn app_mut(&self) -> RefMut<'_, App> {
        self.app.borrow_mut()
    }

    pub fn running(&self) -> bool {
        !self.stack.frames.is_empty()
    }

    /// Live input is ignored until the replay runs out, then you take over.
    pub fn replaying(&self) -> bool {
        !self.replay.is_empty()
    }

    /// How far we are into the next tick, for smooth interpolation.
    pub fn alpha(&self) -> f64 {
        self.acc / self.app().step
    }

    pub fn send(&mut self, action: &str) {
        if action == "debug" {
            self.debug = !self.debug;
        } else if !self.replaying() {
            self.queue.push(action.to_owned());
        }
    }

    /// Run as many fixed ticks as `seconds` of real time pays for.
    pub fn advance(&mut self, seconds: f64) -> bool {
        let step = self.app().step;
        self.acc += seconds.min(0.25); // don't spiral after a hiccup
        while self.acc >= step && self.running() {
            self.acc -= step;
            self.tick();
        }
        self.running()
    }

    pub fn tick(&mut self) {
        let (now, step) = {
            let app = self.app();
            (app.ticks, app.step)
        };
        let mut actions = std::mem::take(&mut self.queue);
        while self.replay.front().is_some_and(|(t, _)| *t <= now) {
            actions.push(self.replay.pop_front().unwrap().1);
        }
        for action in actions {
            self.stack.dispatch(&self.app, &action);
            self.log.push((now, action));
        }
        self.stack.update(&self.app, step);
        self.app_mut().ticks += 1;
    }

    pub fn run_headless(&mut self, max_ticks: u64) {
        let end = self.app().ticks + max_ticks;
        while self.running() && self.app().ticks < end {
            self.tick();
        }
    }

    pub fn view(&self) -> Vec<Node> {
        let mut nodes = Vec::new();
        self.stack.view(&self.app(), &mut nodes);
        if self.debug {
            self.debug_view(&mut nodes);
        }
        nodes
    }

    /// Bottom to top.
    pub fn stack_names(&self) -> Vec<&'static str> {
        self.stack.frames.iter().map(|f| f.state.borrow().name()).collect()
    }

    fn debug_view(&self, out: &mut Vec<Node>) {
        let mut lines = vec![format!(
            "tick {}{}",
            self.app().ticks,
            if self.replaying() { "  (replay)" } else { "" }
        )];
        let names = self.stack_names();
        for (depth, frame) in self.stack.frames.iter().enumerate() {
            let note = match frame.wait {
                Wait::Child(id) => {
                    let child = self.stack.frames.iter().position(|f| f.id == id);
                    format!(" -> {}", child.map_or("?", |i| names[i]))
                }
                Wait::Time(t) => format!(" zz {t:.1}s"),
                Wait::Ready if self.stack.has_script(depth) => " (script)".into(),
                Wait::Ready => String::new(),
            };
            lines.push(format!("{}{}{}", "  ".repeat(depth), names[depth], note));
        }

        let w = (lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i32 + 5) / 2;
        let x = self.app().size.0 - w;
        out.push(scene::frame(x, 0, w, lines.len() as i32 + 2, hex(0xff6ad5), Some(hex(0x1a1023))));
        for (i, line) in lines.into_iter().enumerate() {
            let fg = if i == 0 { hex(0xff6ad5) } else { hex(0xffd6f5) };
            out.push(scene::text(x + 1, i as i32 + 1, line, fg));
        }
    }

    // replays on disk: a seed line, then one "tick action" per line

    pub fn log(&self) -> &Log {
        &self.log
    }

    pub fn save_log(&self) -> String {
        let mut out = format!("seed {}\n", self.seed);
        for (tick, action) in &self.log {
            out += &format!("{tick} {action}\n");
        }
        out
    }

    pub fn load_log(text: &str) -> Option<(u64, Log)> {
        let mut lines = text.lines();
        let seed = lines.next()?.strip_prefix("seed ")?.parse().ok()?;
        let log = lines
            .filter(|l| !l.is_empty())
            .map(|l| {
                let (tick, action) = l.split_once(' ')?;
                Some((tick.parse().ok()?, action.to_owned()))
            })
            .collect::<Option<Log>>()?;
        Some((seed, log))
    }
}
