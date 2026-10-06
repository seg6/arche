//! Shells connect a Runtime to a platform: they turn keys into actions and
//! scene nodes into pixels (or characters, or fridge LEDs).
//!
//! Keys are named the same way everywhere ("up", "return", "f1", "w"), so one
//! keymap fits every shell.

pub mod text;

#[cfg(feature = "terminal")]
pub mod terminal;

#[cfg(feature = "window")]
pub mod window;

pub const DEFAULT_KEYS: &[(&str, &str)] = &[
    ("up", "up"), ("w", "up"), ("k", "up"),
    ("down", "down"), ("s", "down"), ("j", "down"),
    ("left", "left"), ("a", "left"), ("h", "left"),
    ("right", "right"), ("d", "right"), ("l", "right"),
    ("return", "confirm"), ("space", "confirm"),
    ("escape", "back"), ("p", "back"), ("q", "back"),
    ("z", "undo"), ("u", "undo"), ("backspace", "undo"),
    ("r", "restart"),
    ("?", "hint"), ("tab", "hint"),
    ("f1", "debug"), ("`", "debug"),
];

pub fn action_for<'a>(keys: &[(&str, &'a str)], key: &str) -> Option<&'a str> {
    keys.iter().find(|(k, _)| *k == key).map(|(_, a)| *a)
}
