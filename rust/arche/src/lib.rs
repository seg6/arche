//! arche: state management for small games, with the platform kept at arm's length.
//!
//! A game is a stack of [`State`]s. States never touch the stack, the screen or
//! the keyboard. They return [`Effect`]s, hand results back with [`pop_with`],
//! describe frames as [`scene::Node`]s, and run scripts: `async` blocks that
//! can [`Cx::wait`] for game time or [`Cx::push`] a state and await its reply.
//!
//! ```
//! use arche::*;
//!
//! struct Ask;
//! impl State for Ask {
//!     fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
//!         (action == "confirm").then(|| pop_with(42))
//!     }
//! }
//!
//! #[derive(Default)]
//! struct Shop { answer: Option<i32> }
//! impl State for Shop {
//!     fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
//!         script(async move {
//!             cx.wait(0.5).await;
//!             let answer = cx.push(Ask).await.take::<i32>();
//!             cx.with(|me, _| me.answer = answer);
//!             None
//!         })
//!     }
//! }
//!
//! let mut rt = Runtime::new(Shop::default(), Config::default());
//! rt.run_headless(60);
//! assert_eq!(rt.stack_names(), ["Shop", "Ask"]);
//! rt.send("confirm");
//! rt.tick();
//! ```

mod effect;
mod rng;
mod runtime;
pub mod scene;
mod script;
pub mod shells;
mod stack;
mod state;
mod store;

pub use effect::{Effect, Fx, Reply, Spawn, pop, pop_with, push, quit, set};
pub use rng::Rng;
pub use runtime::{App, Config, Log, Runtime};
pub use script::{Cx, Script, script};
pub use stack::Wait;
pub use state::State;
pub use store::Store;
