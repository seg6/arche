//! Every screen in the game is a State. None of them know about any platform.

use std::collections::BTreeMap;

use arche::scene::{self, Kind, Node};
use arche::*;

use crate::art;
use crate::board::{Board, delta, direction};
use crate::levels::LEVELS;

pub const W: i32 = 32;
pub const H: i32 = 18;

/// What menus and the robot hand back with `pop_with`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Choice {
    Resume,
    Restart,
    Robot,
    Menu,
    Next,
}

/// Shared across screens, kept in `app.store`.
#[derive(Default)]
pub struct Progress {
    pub seen_intro: bool,
    pub best: BTreeMap<usize, usize>,
}

pub fn stars_for(moves: usize, par: usize) -> usize {
    if moves <= par { 3 } else if moves * 2 <= par * 3 { 2 } else { 1 }
}

// building blocks ---------------------------------------------------------

/// A vertical list of choices.
pub struct Menu {
    pub items: Vec<(Choice, &'static str)>,
    pub cursor: usize,
}

impl Menu {
    fn new(items: &[(Choice, &'static str)]) -> Self {
        Menu { items: items.to_vec(), cursor: 0 }
    }

    /// Up/down move; confirm picks.
    fn handle(&mut self, action: &str) -> Option<Choice> {
        let n = self.items.len();
        match action {
            "up" => self.cursor = (self.cursor + n - 1) % n,
            "down" => self.cursor = (self.cursor + 1) % n,
            "confirm" => return Some(self.items[self.cursor].0),
            _ => {}
        }
        None
    }

    fn view(&self, y: i32, out: &mut Vec<Node>) {
        for (i, (_, label)) in self.items.iter().enumerate() {
            let y = y + i as i32;
            if i == self.cursor {
                out.push(scene::centered(W / 2, y, format!("> {label} <"), art::ACCENT));
            } else {
                out.push(scene::centered(W / 2, y, *label, art::INK));
            }
        }
    }
}

/// A dialog box with a typewriter effect. Confirm to skip ahead, then to close.
pub struct Say {
    text: &'static str,
    shown: f64,
}

impl Say {
    pub fn new(text: &'static str) -> Self {
        Say { text, shown: 0.0 }
    }
}

impl State for Say {
    fn overlay(&self) -> bool {
        true
    }

    fn update(&mut self, _: &mut App, dt: f64) -> Fx {
        self.shown += dt * 45.0;
        None
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        if !matches!(action, "confirm" | "back") {
            return None;
        }
        if (self.shown as usize) < self.text.len() {
            self.shown = self.text.len() as f64;
            None
        } else {
            Some(pop())
        }
    }

    fn view(&self, app: &App, out: &mut Vec<Node>) {
        out.push(art::panel(1, H - 5, W - 2, 5));
        out.push(Node::Text {
            x: 2, y: H - 5, text: " night manager ".into(),
            fg: art::ACCENT, bg: Some(art::PANEL), align: scene::Align::Left,
        });
        let mut budget = self.shown as usize;
        for (i, line) in art::wrap(self.text, 54).iter().enumerate() {
            let visible: String = line.chars().take(budget).collect();
            out.push(scene::text(2, H - 4 + i as i32, visible, art::INK));
            budget = budget.saturating_sub(line.len() + 1);
        }
        if self.shown as usize >= self.text.len() && app.ticks / 20 % 2 == 1 {
            out.push(scene::text(W - 3, H - 2, ">>", art::MUTED));
        }
    }
}

/// A title card that dismisses itself.
pub struct Banner {
    title: String,
    subtitle: &'static str,
}

impl State for Banner {
    fn overlay(&self) -> bool {
        true
    }

    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        script(async move {
            cx.wait(1.4).await;
            Some(pop())
        })
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        (action == "confirm").then(pop)
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        out.push(scene::fill(0, H / 2 - 2, W, 4, art::PANEL));
        out.push(scene::centered(W / 2, H / 2 - 1, self.title.clone(), art::ACCENT));
        out.push(scene::centered(W / 2, H / 2, self.subtitle, art::INK));
    }
}

// screens -----------------------------------------------------------------

pub struct Title {
    t: f64,
    demo: usize,
    board: Board,
    moves: Vec<char>,
    linger: u32,
}

impl Title {
    const DEMOS: [usize; 3] = [1, 2, 4]; // levels the attract mode replays

    pub fn new() -> Self {
        let mut title = Title { t: 0.0, demo: 0, board: Board::parse(LEVELS[1].map), moves: vec![], linger: 0 };
        title.load_demo();
        title
    }

    fn load_demo(&mut self) {
        let level = &LEVELS[Self::DEMOS[self.demo]];
        self.board = Board::parse(level.map);
        self.moves = level.solution.chars().rev().collect();
        self.linger = 0;
    }

    fn attract(&mut self) {
        if let Some(letter) = self.moves.pop() {
            self.board = self.board.step(direction(letter)).unwrap();
        } else if self.linger >= 8 {
            self.demo = (self.demo + 1) % Self::DEMOS.len();
            self.load_demo();
        } else {
            self.linger += 1;
        }
    }
}

impl State for Title {
    fn update(&mut self, _: &mut App, dt: f64) -> Fx {
        self.t += dt;
        None
    }

    /// No timer API needed: a script that loops forever is a timer.
    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        script(async move {
            loop {
                cx.wait(0.2).await;
                cx.with(|me, _| me.attract());
            }
        })
    }

    fn on_action(&mut self, app: &mut App, action: &str) -> Fx {
        match action {
            "confirm" if app.store.get::<Progress>().seen_intro => Some(push(LevelSelect::new())),
            "confirm" => Some(push(Intro::new())),
            "back" => Some(quit()),
            _ => None,
        }
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        out.push(scene::clear(art::BG));
        art::logo("CRATES", 4, 1, self.t, out);
        art::board(&self.board, (W - self.board.w) / 2, 8, out);
        out.push(scene::centered(W / 2, 15, "attract mode: replaying a solution", art::MUTED));
        out.push(scene::centered(W / 2, 17, "enter play  ·  esc quit  ·  f1 debug", art::INK));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Pointing {
    Nothing,
    You,
    Crates,
}

/// A cutscene, written top to bottom as a script.
pub struct Intro {
    board: Board,
    pointing: Pointing,
}

impl Intro {
    pub fn new() -> Self {
        Intro { board: Board::parse(LEVELS[0].map), pointing: Pointing::Nothing }
    }
}

impl State for Intro {
    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        script(async move {
            cx.with(|_, app| app.store.get::<Progress>().seen_intro = true);
            cx.wait(0.5).await;

            cx.with(|me, _| me.pointing = Pointing::You);
            cx.push(Say::new("this is you. welcome to the night shift at the crate warehouse.")).await;
            cx.with(|me, _| me.pointing = Pointing::Crates);
            cx.push(Say::new("crates go on the pink spots. you can push them, but you can never pull.")).await;
            cx.with(|me, _| me.pointing = Pointing::Nothing);

            for letter in LEVELS[0].solution.chars() {
                cx.wait(0.35).await;
                cx.with(|me, _| me.board = me.board.step(direction(letter)).unwrap());
            }
            cx.wait(0.6).await;

            cx.push(Say::new("that's the whole job. it only gets harder from here.")).await;
            Some(set(LevelSelect::new()))
        })
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        (action == "back").then(|| set(LevelSelect::new()))
    }

    fn view(&self, app: &App, out: &mut Vec<Node>) {
        out.push(scene::clear(art::BG));
        let (ox, oy) = art::centered(&self.board, (W, H - 5), 0);
        art::board(&self.board, ox, oy, out);

        if app.ticks / 15 % 2 == 1 {
            let spots: Vec<_> = match self.pointing {
                Pointing::You => vec![self.board.player],
                Pointing::Crates => self.board.boxes.iter().chain(self.board.goals.iter()).copied().collect(),
                Pointing::Nothing => vec![],
            };
            for (x, y) in spots {
                out.push(scene::tile(ox + x, oy + y, Kind::Ring, art::ACCENT));
            }
        }
    }
}

pub struct LevelSelect {
    cursor: usize,
    playing: usize,
}

impl LevelSelect {
    pub fn new() -> Self {
        LevelSelect { cursor: 0, playing: 0 }
    }
}

fn unlocked(progress: &Progress, i: usize) -> bool {
    i == 0 || progress.best.contains_key(&(i - 1))
}

impl State for LevelSelect {
    fn on_action(&mut self, app: &mut App, action: &str) -> Fx {
        let n = LEVELS.len();
        match action {
            "up" => self.cursor = (self.cursor + n - 1) % n,
            "down" => self.cursor = (self.cursor + 1) % n,
            "confirm" if unlocked(app.store.get(), self.cursor) => {
                self.playing = self.cursor;
                return Some(push(Play::new(self.cursor)));
            }
            "back" => return Some(pop()),
            _ => {}
        }
        None
    }

    fn on_resume(&mut self, _: &mut App, mut reply: Reply) -> Fx {
        if reply.take::<Choice>() == Some(Choice::Next) && self.playing + 1 < LEVELS.len() {
            self.playing += 1;
            self.cursor = self.playing;
            return Some(push(Play::new(self.playing)));
        }
        None
    }

    fn view(&self, app: &App, out: &mut Vec<Node>) {
        let empty = Progress::default();
        let progress = app.store.peek::<Progress>().unwrap_or(&empty);
        out.push(scene::clear(art::BG));
        out.push(scene::centered(W / 2, 1, "pick a shift", art::ACCENT));

        for (i, level) in LEVELS.iter().enumerate() {
            let y = 3 + 2 * i as i32;
            if i == self.cursor {
                out.push(scene::fill(3, y, W - 6, 1, art::PANEL));
                out.push(scene::tile(3, y, Kind::Ball, art::PLAYER));
            }
            if !unlocked(progress, i) {
                out.push(scene::text(5, y, format!("{}  locked", i + 1), art::MUTED));
                continue;
            }
            out.push(scene::text(5, y, format!("{}  {}", i + 1, level.name), art::INK));
            let par = level.solution.len();
            let best = progress.best.get(&i);
            let earned = best.map_or(0, |&b| stars_for(b, par));
            for k in 0..3 {
                let color = if k < earned { art::PLAYER } else { art::FLOOR };
                out.push(scene::tile(17 + k as i32, y, Kind::Star, color));
            }
            let score = best.map_or(format!("- / {par}"), |b| format!("{b} / {par}"));
            out.push(scene::text(21, y, score, art::MUTED));
        }
    }
}

fn draw_level(index: usize, board: &Board, moves: usize, out: &mut Vec<Node>) {
    let level = &LEVELS[index];
    out.push(scene::clear(art::BG));
    let (ox, oy) = art::centered(board, (W, H - 1), 2);
    art::board(board, ox, oy, out);
    out.push(scene::text(1, 0, format!("shift {}: {}", index + 1, level.name), art::INK));
    out.push(scene::text(W - 9, 0, format!("moves {moves:>3}"), art::INK));
    out.push(scene::text(W - 9, 1, format!("par   {:>3}", level.solution.len()), art::MUTED));
}

pub struct Play {
    pub index: usize,
    pub board: Board,
    pub history: Vec<Board>,
}

impl Play {
    pub fn new(index: usize) -> Self {
        Play { index, board: Board::parse(LEVELS[index].map), history: vec![] }
    }

    fn reset(&mut self) {
        *self = Play::new(self.index);
    }

    pub fn moves(&self) -> usize {
        self.history.len()
    }

    fn walk(&mut self, app: &mut App, direction: &str) -> Fx {
        let next = self.board.step(direction)?;
        self.history.push(std::mem::replace(&mut self.board, next));
        if !self.board.solved() {
            return None;
        }
        let moves = self.moves();
        let best = app.store.get::<Progress>().best.entry(self.index).or_insert(moves);
        *best = (*best).min(moves);
        let par = LEVELS[self.index].solution.len();
        Some(push(Win::new(moves, par, self.index == LEVELS.len() - 1)))
    }
}

impl State for Play {
    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        let (title, subtitle) = (format!("shift {}", self.index + 1), LEVELS[self.index].name);
        script(async move {
            cx.push(Banner { title, subtitle }).await;
            None
        })
    }

    fn on_action(&mut self, app: &mut App, action: &str) -> Fx {
        match action {
            _ if delta(action).is_some() => return self.walk(app, action),
            "undo" => {
                if let Some(board) = self.history.pop() {
                    self.board = board;
                }
            }
            "restart" => self.reset(),
            "hint" => return Some(push(Robot::new(self.index))),
            "back" => return Some(push(Pause::new())),
            _ => {}
        }
        None
    }

    fn on_resume(&mut self, _: &mut App, mut reply: Reply) -> Fx {
        match reply.take::<Choice>()? {
            Choice::Restart => self.reset(),
            Choice::Robot => return Some(push(Robot::new(self.index))),
            Choice::Menu => return Some(pop()),
            Choice::Next => return Some(pop_with(Choice::Next)),
            Choice::Resume => {}
        }
        None
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        draw_level(self.index, &self.board, self.moves(), out);
        out.push(scene::centered(W / 2, H - 1, "arrows move · z undo · r restart · tab robot · esc menu", art::MUTED));
    }
}

/// Plays the level's solution on its own board. Any key stops it; either way
/// the level underneath starts fresh.
pub struct Robot {
    index: usize,
    board: Board,
    moves: usize,
}

impl Robot {
    fn new(index: usize) -> Self {
        Robot { index, board: Board::parse(LEVELS[index].map), moves: 0 }
    }
}

impl State for Robot {
    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        let solution = LEVELS[self.index].solution;
        script(async move {
            cx.wait(0.4).await;
            for letter in solution.chars() {
                cx.with(|me, _| {
                    me.board = me.board.step(direction(letter)).unwrap();
                    me.moves += 1;
                });
                cx.wait(0.16).await;
            }
            cx.wait(0.8).await;
            Some(pop_with(Choice::Restart))
        })
    }

    fn on_action(&mut self, _: &mut App, _: &str) -> Fx {
        Some(pop_with(Choice::Restart))
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        draw_level(self.index, &self.board, self.moves, out);
        out.push(scene::fill(0, H - 1, W, 1, art::PANEL));
        out.push(scene::centered(W / 2, H - 1, "the robot is showing you how · any key to stop", art::ACCENT));
    }
}

pub struct Pause {
    menu: Menu,
}

impl Pause {
    fn new() -> Self {
        Pause {
            menu: Menu::new(&[
                (Choice::Resume, "keep going"),
                (Choice::Restart, "start over"),
                (Choice::Robot, "watch the robot"),
                (Choice::Menu, "back to shifts"),
            ]),
        }
    }
}

impl State for Pause {
    fn overlay(&self) -> bool {
        true
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        if action == "back" {
            return Some(pop_with(Choice::Resume));
        }
        self.menu.handle(action).map(pop_with)
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        out.push(scene::dim(0.6));
        out.push(art::panel(9, 5, 14, 8));
        out.push(scene::centered(W / 2, 6, "paused", art::ACCENT));
        self.menu.view(8, out);
    }
}

pub struct Win {
    pub moves: usize,
    pub par: usize,
    last: bool,
    lit: usize,
    menu: Menu,
}

impl Win {
    fn new(moves: usize, par: usize, last: bool) -> Self {
        let mut items = vec![(Choice::Restart, "try for fewer moves"), (Choice::Menu, "back to shifts")];
        if !last {
            items.insert(0, (Choice::Next, "next shift"));
        }
        Win { moves, par, last, lit: 0, menu: Menu::new(&items) }
    }
}

impl State for Win {
    fn overlay(&self) -> bool {
        true
    }

    /// The stars light up one at a time.
    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        let stars = stars_for(self.moves, self.par);
        script(async move {
            for _ in 0..stars {
                cx.wait(0.3).await;
                cx.with(|me, _| me.lit += 1);
            }
            None
        })
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        if action == "back" {
            return Some(pop_with(Choice::Menu));
        }
        self.menu.handle(action).map(pop_with)
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        out.push(scene::dim(0.5));
        out.push(art::panel(6, 3, 20, 12));
        let title = if self.last { "every crate is home!" } else { "shift complete!" };
        out.push(scene::centered(W / 2, 4, title, art::ACCENT));
        for k in 0..3 {
            let color = if k < self.lit { art::PLAYER } else { art::FLOOR };
            out.push(scene::tile(W / 2 - 2 + k as i32, 6, Kind::Star, color));
        }
        out.push(scene::centered(W / 2, 8, format!("{} moves · par {}", self.moves, self.par), art::INK));
        self.menu.view(10, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arche::shells::text::to_text;

    struct Driver(Runtime);

    impl Driver {
        fn new(config: Config) -> Self {
            Driver(Runtime::new(Title::new(), Config { size: (W, H), ..config }))
        }

        fn top(&self) -> &'static str {
            self.0.stack_names().last().copied().unwrap_or("")
        }

        fn press(&mut self, actions: &[&str]) {
            for action in actions {
                self.0.send(action);
                self.0.tick();
                to_text(&self.0.view(), (W, H)); // every frame must render
            }
        }

        /// Tick (clicking through dialogs) until `name` is on top.
        fn until(&mut self, name: &str) {
            for _ in 0..3000 {
                match self.top() {
                    top if top == name => return,
                    "Say" => self.press(&["confirm"]),
                    _ => self.0.tick(),
                }
            }
            panic!("never reached {name}: {:?}", self.0.stack_names());
        }

        fn best(&self) -> Vec<(usize, usize)> {
            let app = self.0.app();
            app.store.peek::<Progress>().map_or(vec![], |p| p.best.clone().into_iter().collect())
        }
    }

    fn first_shift_then_next(d: &mut Driver) {
        d.press(&["confirm"]);
        d.until("LevelSelect");
        d.press(&["confirm"]);
        d.until("Banner");
        d.press(&["confirm"]);
        d.press(&["right", "left", "right", "right", "right"]); // 5 moves, par 3
        d.until("Win");
        d.press(&["confirm"]); // next shift
        d.until("Play");
    }

    #[test]
    fn a_whole_session() {
        let mut d = Driver::new(Config::default());
        first_shift_then_next(&mut d);
        assert_eq!(d.best(), [(0, 5)]);
        assert_eq!(d.0.stack_names(), ["Title", "LevelSelect", "Play"]); // the banner has come and gone

        let moves: Vec<&str> = LEVELS[1].solution.chars().map(direction).collect();
        d.press(&moves);
        d.until("Win");
        d.press(&["down", "confirm"]); // try for fewer moves
        d.press(&["back", "up", "confirm"]); // pause, back to shifts
        d.until("LevelSelect");
        assert_eq!(d.best(), [(0, 5), (1, 12)]);
        d.press(&["back", "back"]);
        assert!(!d.0.running());
    }

    #[test]
    fn the_robot_solves_it_then_hands_back_a_fresh_board() {
        let mut d = Driver::new(Config::default());
        first_shift_then_next(&mut d);
        d.press(&["right", "hint"]);
        d.0.run_headless(60 * 2); // the solution takes ~3s to play
        assert_eq!(d.top(), "Robot");
        d.until("Play");
        let text = to_text(&d.0.view(), (W, H));
        assert!(text.contains("moves   0"), "{text}");
    }

    #[test]
    fn replays_reproduce_a_session() {
        let mut live = Driver::new(Config::default());
        first_shift_then_next(&mut live);

        let (seed, log) = Runtime::load_log(&live.0.save_log()).unwrap();
        let mut again = Driver::new(Config { seed, replay: Some(log), ..Config::default() });
        let ticks = live.0.app().ticks;
        again.0.run_headless(ticks);

        assert_eq!(again.best(), live.best());
        assert_eq!(to_text(&again.0.view(), (W, H)), to_text(&live.0.view(), (W, H)));
    }
}
