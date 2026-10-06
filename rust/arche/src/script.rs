//! Scripts: `async` blocks that the stack polls once per tick.
//!
//! There's no async runtime here. Awaiting [`Cx::wait`] or [`Cx::push`] leaves a
//! request for the stack and suspends; the stack resumes the script when the
//! time has passed or the pushed state pops.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::effect::{Effect, Fx, Reply, Spawn, spawner};
use crate::runtime::App;
use crate::state::State;

pub type Script = Pin<Box<dyn Future<Output = Fx>>>;

/// Wrap an async block as a state's script: `script(async move { .. })`.
pub fn script(body: impl Future<Output = Fx> + 'static) -> Option<Script> {
    Some(Box::pin(body))
}

pub(crate) enum Request {
    Wait(f64),
    Push(Spawn),
}

/// The mailbox between one script and the stack.
#[derive(Default)]
pub(crate) struct Co {
    pub request: RefCell<Option<Request>>,
    pub reply: RefCell<Option<Reply>>,
    pub effects: RefCell<Vec<Effect>>,
}

/// A script's handle on its own state, the app, and the stack.
pub struct Cx<S> {
    pub(crate) me: Rc<RefCell<S>>,
    pub(crate) app: Rc<RefCell<App>>,
    pub(crate) co: Rc<Co>,
}

impl<S> Clone for Cx<S> {
    fn clone(&self) -> Self {
        Cx { me: self.me.clone(), app: self.app.clone(), co: self.co.clone() }
    }
}

impl<S: State> Cx<S> {
    /// Borrow your state and the app for a moment. Don't hold on across an `.await`.
    pub fn with<R>(&self, f: impl FnOnce(&mut S, &mut App) -> R) -> R {
        f(&mut self.me.borrow_mut(), &mut self.app.borrow_mut())
    }

    /// Suspend for `seconds` of game time. Pausing (pushing on top) pauses this too.
    pub fn wait(&self, seconds: f64) -> impl Future<Output = ()> + 'static {
        let suspend = Suspend { co: self.co.clone(), request: Some(Request::Wait(seconds)) };
        async move {
            suspend.await;
        }
    }

    /// Push a state, and resume with whatever it pops with.
    pub fn push<T: State>(&self, state: T) -> impl Future<Output = Reply> + 'static {
        Suspend { co: self.co.clone(), request: Some(Request::Push(spawner(state))) }
    }

    /// Apply an effect without waiting, e.g. `cx.emit(set(Next))`. Takes effect
    /// when the script next suspends; if it removes this state, the script ends there.
    pub fn emit(&self, effect: Effect) {
        self.co.effects.borrow_mut().push(effect);
    }

    pub fn ticks(&self) -> u64 {
        self.app.borrow().ticks
    }
}

struct Suspend {
    co: Rc<Co>,
    request: Option<Request>,
}

impl Future for Suspend {
    type Output = Reply;

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Reply> {
        let this = self.get_mut();
        match this.request.take() {
            Some(request) => {
                *this.co.request.borrow_mut() = Some(request);
                Poll::Pending
            }
            None => Poll::Ready(this.co.reply.borrow_mut().take().unwrap_or_default()),
        }
    }
}
