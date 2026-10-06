//! Effects: what a state *wants* to happen, as plain values.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::runtime::App;
use crate::script::{Co, Cx, Script};
use crate::state::State;

/// A value handed back by [`pop_with`]. Read it with [`Reply::take`].
#[derive(Default)]
pub struct Reply(Option<Box<dyn Any>>);

impl Reply {
    pub fn none() -> Self {
        Reply(None)
    }

    pub fn new<T: Any>(value: T) -> Self {
        Reply(Some(Box::new(value)))
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    /// Take the value out if it is a `T`. Anything else stays put.
    pub fn take<T: Any>(&mut self) -> Option<T> {
        match self.0.take()?.downcast::<T>() {
            Ok(value) => Some(*value),
            Err(other) => {
                self.0 = Some(other);
                None
            }
        }
    }
}

impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(if self.0.is_some() { "Reply(..)" } else { "Reply(none)" })
    }
}

pub(crate) struct Spawned {
    pub state: Rc<RefCell<dyn State>>,
    pub co: Rc<Co>,
    pub script: Option<Script>,
}

/// A state waiting to be pushed. Its frame (and script) is built once the
/// stack has it, since that's when the app exists.
pub struct Spawn(Box<Builder>);

type Builder = dyn FnOnce(&Rc<RefCell<App>>) -> Spawned;

impl Spawn {
    pub(crate) fn build(self, app: &Rc<RefCell<App>>) -> Spawned {
        (self.0)(app)
    }
}

pub(crate) fn spawner<S: State>(state: S) -> Spawn {
    Spawn(Box::new(move |app| {
        let me = Rc::new(RefCell::new(state));
        let co = Rc::new(Co::default());
        let cx = Cx { me: me.clone(), app: app.clone(), co: co.clone() };
        let script = me.borrow_mut().script(cx);
        Spawned { state: me, co, script }
    }))
}

pub enum Effect {
    /// Put a state on top.
    Push(Spawn),
    /// Replace the top state.
    Set(Spawn),
    /// Remove the top state, handing a reply to the one underneath.
    Pop(Reply),
    /// Unwind the whole stack.
    Quit,
}

/// What hooks return: usually `None`, sometimes one effect.
pub type Fx = Option<Effect>;

pub fn push<S: State>(state: S) -> Effect {
    Effect::Push(spawner(state))
}

pub fn set<S: State>(state: S) -> Effect {
    Effect::Set(spawner(state))
}

pub fn pop() -> Effect {
    Effect::Pop(Reply::none())
}

pub fn pop_with<T: Any>(value: T) -> Effect {
    Effect::Pop(Reply::new(value))
}

pub fn quit() -> Effect {
    Effect::Quit
}

impl fmt::Debug for Effect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Effect::Push(_) => f.write_str("Push(..)"),
            Effect::Set(_) => f.write_str("Set(..)"),
            Effect::Pop(reply) => write!(f, "Pop({reply:?})"),
            Effect::Quit => f.write_str("Quit"),
        }
    }
}
