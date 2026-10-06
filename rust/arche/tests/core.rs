use std::cell::RefCell;
use std::rc::Rc;

use arche::scene::{self, Node, hex};
use arche::*;

struct Ask(i32);

impl State for Ask {
    fn overlay(&self) -> bool {
        true
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        (action == "confirm").then(|| pop_with(self.0))
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        out.push(scene::text(0, 0, "ask?", hex(0xffffff)));
    }
}

#[derive(Default)]
struct Root {
    events: Rc<RefCell<Vec<Option<i32>>>>,
}

impl State for Root {
    fn on_resume(&mut self, _: &mut App, mut reply: Reply) -> Fx {
        self.events.borrow_mut().push(reply.take::<i32>());
        None
    }

    fn on_action(&mut self, _: &mut App, action: &str) -> Fx {
        match action {
            "ask" => Some(push(Ask(42))),
            "bye" => Some(quit()),
            _ => None,
        }
    }

    fn view(&self, _: &App, out: &mut Vec<Node>) {
        out.push(scene::clear(hex(0)));
    }
}

fn press(rt: &mut Runtime, actions: &[&str]) {
    for action in actions {
        rt.send(action);
        rt.tick();
    }
}

#[test]
fn pop_hands_its_reply_to_on_resume() {
    let root = Root::default();
    let events = root.events.clone();
    let mut rt = Runtime::new(root, Config::default());
    press(&mut rt, &["ask", "confirm"]);
    assert_eq!(*events.borrow(), [Some(42)]);
    assert_eq!(rt.stack_names(), ["Root"]);
}

#[test]
fn overlays_show_the_state_underneath() {
    let mut rt = Runtime::new(Root::default(), Config::default());
    press(&mut rt, &["ask"]);
    let view = rt.view();
    assert!(matches!(view[..], [Node::Clear { .. }, Node::Text { .. }]));
}

#[test]
fn only_the_top_state_hears_actions_and_quit_unwinds() {
    let mut rt = Runtime::new(Root::default(), Config::default());
    press(&mut rt, &["ask", "bye"]); // Ask ignores "bye"
    assert!(rt.running());
    press(&mut rt, &["confirm", "bye"]);
    assert!(!rt.running());
}

#[derive(Default)]
struct Scripted {
    seen: Rc<RefCell<Vec<String>>>,
}

impl State for Scripted {
    fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
        let seen = self.seen.clone();
        script(async move {
            cx.wait(0.5).await;
            seen.borrow_mut().push("waited".into());
            let answer = cx.push(Ask(7)).await.take::<i32>();
            seen.borrow_mut().push(format!("got {answer:?}"));
            Some(pop())
        })
    }
}

#[test]
fn scripts_wait_and_receive_replies() {
    let state = Scripted::default();
    let seen = state.seen.clone();
    // something underneath, so popping the scripted state leaves the game running
    struct Pusher(Option<Scripted>);
    impl State for Pusher {
        fn on_enter(&mut self, _: &mut App) -> Fx {
            Some(push(self.0.take().unwrap()))
        }
    }
    let mut rt2 = Runtime::new(Pusher(Some(state)), Config { step: 0.1, ..Config::default() });

    rt2.run_headless(4);
    assert!(seen.borrow().is_empty());
    rt2.tick();
    assert_eq!(*seen.borrow(), ["waited"]);
    assert_eq!(rt2.stack_names(), ["Pusher", "Scripted", "Ask"]);

    press(&mut rt2, &["confirm"]); // Ask pops; the reply goes to the script
    assert_eq!(*seen.borrow(), ["waited", "got Some(7)"]);
    assert_eq!(rt2.stack_names(), ["Pusher"]);
}

#[test]
fn a_set_replacement_inherits_the_pending_push() {
    struct Caller(Rc<RefCell<Option<i32>>>);
    impl State for Caller {
        fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
            let got = self.0.clone();
            script(async move {
                *got.borrow_mut() = cx.push(Swapper).await.take::<i32>();
                None
            })
        }
    }
    struct Swapper;
    impl State for Swapper {
        fn on_action(&mut self, _: &mut App, _: &str) -> Fx {
            Some(set(Ask(2)))
        }
    }

    let got = Rc::new(RefCell::new(None));
    let mut rt = Runtime::new(Caller(got.clone()), Config::default());
    rt.tick();
    press(&mut rt, &["swap", "confirm"]);
    rt.tick();
    assert_eq!(*got.borrow(), Some(2));
}

#[test]
fn scripts_pause_while_something_is_on_top() {
    struct Ticker(Rc<RefCell<u32>>);
    impl State for Ticker {
        fn on_action(&mut self, _: &mut App, _: &str) -> Fx {
            Some(push(Ask(0)))
        }
        fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
            let n = self.0.clone();
            script(async move {
                loop {
                    cx.wait(0.25).await;
                    *n.borrow_mut() += 1;
                }
            })
        }
    }

    let n = Rc::new(RefCell::new(0));
    let mut rt = Runtime::new(Ticker(n.clone()), Config { step: 0.05, ..Config::default() });
    rt.run_headless(10);
    assert_eq!(*n.borrow(), 2);
    press(&mut rt, &["pause"]);
    rt.run_headless(20);
    assert_eq!(*n.borrow(), 2);
}

#[test]
fn emit_applies_effects_mid_script() {
    struct Hop;
    impl State for Hop {
        fn script(&mut self, cx: Cx<Self>) -> Option<Script> {
            script(async move {
                cx.emit(push(Ask(1)));
                cx.wait(0.0).await;
                None
            })
        }
    }
    let mut rt = Runtime::new(Hop, Config::default());
    rt.tick();
    assert_eq!(rt.stack_names(), ["Hop", "Ask"]);
}

#[test]
fn replays_reproduce_a_run_then_hand_back_control() {
    fn play(rt: &mut Runtime, inputs: &[(u64, &str)]) {
        for tick in 0..30 {
            if let Some((_, action)) = inputs.iter().find(|(t, _)| *t == tick) {
                rt.send(action);
            }
            rt.tick();
        }
    }

    let mut live = Runtime::new(Root::default(), Config::default());
    play(&mut live, &[(3, "ask"), (9, "confirm"), (20, "ask")]);

    let (seed, log) = Runtime::load_log(&live.save_log()).unwrap();
    let root = Root::default();
    let events = root.events.clone();
    let mut again = Runtime::new(root, Config { seed, replay: Some(log), ..Config::default() });
    play(&mut again, &[(5, "confirm"), (25, "confirm")]);

    assert_eq!(&again.log()[..3], &live.log()[..]);
    assert_eq!(*events.borrow(), [Some(42), Some(42)]);
    assert!(!again.replaying());
}

#[test]
fn rng_and_store_are_deterministic_services() {
    let mut a = Rng::new(9);
    let mut b = Rng::new(9);
    assert!((0..100).all(|_| a.next_u64() == b.next_u64()));

    #[derive(Default)]
    struct Progress(u32);
    let mut store = Store::default();
    store.get::<Progress>().0 += 3;
    assert_eq!(store.peek::<Progress>().unwrap().0, 3);
}
