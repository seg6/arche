//! The state stack: a tiny interpreter for effects. No platform code here.

use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use crate::effect::{Effect, Fx, Reply, Spawn};
use crate::runtime::App;
use crate::scene::Node;
use crate::script::{Co, Request, Script};
use crate::state::State;

/// What a state's script is waiting for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Wait {
    Ready,
    Time(f64),
    /// The id of the frame it pushed.
    Child(u64),
}

pub(crate) struct Frame {
    pub id: u64,
    pub state: Rc<RefCell<dyn State>>,
    pub wait: Wait,
    co: Rc<Co>,
    script: Option<Script>,
}

#[derive(Default)]
pub(crate) struct Stack {
    pub frames: Vec<Frame>,
    next_id: u64,
}

type AppRc = Rc<RefCell<App>>;

impl Stack {
    fn index(&self, id: u64) -> Option<usize> {
        self.frames.iter().position(|f| f.id == id)
    }

    fn top_id(&self) -> Option<u64> {
        self.frames.last().map(|f| f.id)
    }

    pub fn apply(&mut self, app: &AppRc, fx: Fx) {
        if let Some(effect) = fx {
            self.apply_one(app, effect);
        }
    }

    pub fn apply_one(&mut self, app: &AppRc, effect: Effect) {
        match effect {
            Effect::Push(spawn) => {
                if let Some(top) = self.frames.last() {
                    let state = top.state.clone();
                    state.borrow_mut().on_pause(&mut app.borrow_mut());
                }
                self.enter(app, spawn);
            }

            Effect::Set(spawn) => {
                let old = self.leave(app);
                // a replacement inherits a pending `cx.push(old)`
                if let Some(parent) = self.frames.last_mut()
                    && parent.wait == Wait::Child(old)
                {
                    parent.wait = Wait::Child(self.next_id);
                }
                self.enter(app, spawn);
            }

            Effect::Pop(reply) => {
                let popped = self.leave(app);
                let Some(parent) = self.frames.last_mut() else { return };
                let reply = if parent.wait == Wait::Child(popped) {
                    parent.wait = Wait::Ready;
                    *parent.co.reply.borrow_mut() = Some(reply);
                    Reply::none()
                } else {
                    reply
                };
                let state = parent.state.clone();
                let fx = state.borrow_mut().on_resume(&mut app.borrow_mut(), reply);
                self.apply(app, fx);
            }

            Effect::Quit => {
                while !self.frames.is_empty() {
                    self.leave(app);
                }
            }
        }
    }

    fn enter(&mut self, app: &AppRc, spawn: Spawn) {
        let id = self.next_id;
        self.next_id += 1;
        let spawned = spawn.build(app);
        let state = spawned.state.clone();
        self.frames.push(Frame {
            id,
            state: spawned.state,
            wait: Wait::Ready,
            co: spawned.co,
            script: spawned.script,
        });
        let fx = state.borrow_mut().on_enter(&mut app.borrow_mut());
        self.apply(app, fx);
    }

    fn leave(&mut self, app: &AppRc) -> u64 {
        let frame = self.frames.pop().expect("pop from an empty state stack");
        frame.state.borrow_mut().on_exit(&mut app.borrow_mut());
        frame.id // dropping the frame drops its script
    }

    // per tick, only the top state is live

    pub fn dispatch(&mut self, app: &AppRc, action: &str) {
        let Some(top) = self.frames.last() else { return };
        let state = top.state.clone();
        let fx = state.borrow_mut().on_action(&mut app.borrow_mut(), action);
        self.apply(app, fx);
    }

    pub fn update(&mut self, app: &AppRc, dt: f64) {
        let Some(top) = self.frames.last() else { return };
        let (id, state) = (top.id, top.state.clone());
        let fx = state.borrow_mut().update(&mut app.borrow_mut(), dt);
        self.apply(app, fx);
        if self.top_id() == Some(id) {
            self.run_script(app, dt);
        }
    }

    fn run_script(&mut self, app: &AppRc, dt: f64) {
        let frame = self.frames.last_mut().unwrap();
        match frame.wait {
            Wait::Child(_) => return,
            Wait::Time(t) if t - dt > 1e-9 => {
                frame.wait = Wait::Time(t - dt);
                return;
            }
            _ => frame.wait = Wait::Ready,
        }
        let Some(mut script) = frame.script.take() else { return };
        let (id, co) = (frame.id, frame.co.clone());

        // no borrows are held while the script runs, so `cx.with` is free to borrow
        let poll = script.as_mut().poll(&mut Context::from_waker(Waker::noop()));
        let emitted = co.effects.take();
        let request = co.request.take();

        if poll.is_pending() {
            self.frames.last_mut().unwrap().script = Some(script);
        }
        for effect in emitted {
            self.apply_one(app, effect);
        }

        match poll {
            Poll::Ready(fx) => self.apply(app, fx),
            Poll::Pending => {
                let Some(i) = self.index(id) else { return };
                match request {
                    Some(Request::Wait(seconds)) => {
                        self.frames[i].wait = Wait::Time(seconds - dt); // this tick counts too
                    }
                    Some(Request::Push(spawn)) => {
                        self.frames[i].wait = Wait::Child(self.next_id);
                        self.apply_one(app, Effect::Push(spawn));
                    }
                    None => self.frames[i].wait = Wait::Time(0.0), // awaited something else: next tick
                }
            }
        }
    }

    /// The top state, plus everything visible through overlays.
    pub fn view(&self, app: &App, out: &mut Vec<Node>) {
        let mut first = self.frames.len().saturating_sub(1);
        while first > 0 && self.frames[first].state.borrow().overlay() {
            first -= 1;
        }
        for frame in self.frames.iter().skip(first) {
            frame.state.borrow().view(app, out);
        }
    }

    pub fn has_script(&self, i: usize) -> bool {
        self.frames[i].script.is_some()
    }
}
