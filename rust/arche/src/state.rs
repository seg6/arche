use crate::effect::{Fx, Reply};
use crate::runtime::App;
use crate::scene::Node;
use crate::script::{Cx, Script};

/// One screen of your game. Every method is optional.
#[allow(unused_variables)]
pub trait State: 'static {
    fn name(&self) -> &'static str {
        let full = std::any::type_name::<Self>();
        full.rsplit("::").next().unwrap_or(full)
    }

    /// Draw the states underneath too (dialogs, pause menus).
    fn overlay(&self) -> bool {
        false
    }

    fn on_enter(&mut self, app: &mut App) -> Fx {
        None
    }

    fn on_exit(&mut self, app: &mut App) {}

    fn on_pause(&mut self, app: &mut App) {}

    /// The state above popped. Replies a script is awaiting go to the script
    /// instead, and this gets an empty one.
    fn on_resume(&mut self, app: &mut App, reply: Reply) -> Fx {
        None
    }

    fn on_action(&mut self, app: &mut App, action: &str) -> Fx {
        None
    }

    fn update(&mut self, app: &mut App, dt: f64) -> Fx {
        None
    }

    /// Describe the frame as platform-neutral scene nodes.
    fn view(&self, app: &App, out: &mut Vec<Node>) {}

    /// Started when the state is pushed; polled on ticks while it's on top.
    fn script(&mut self, cx: Cx<Self>) -> Option<Script>
    where
        Self: Sized,
    {
        None
    }
}
