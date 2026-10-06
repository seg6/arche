//! Platform-neutral scene nodes.
//!
//! Everything is laid out on a grid of cells. A cell is square and holds one
//! tile, or two characters of text. Each shell decides what a cell looks like:
//! pixels in a window, two columns in a terminal, one LED on a fridge.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// `hex(0xff6ad5)`
pub const fn hex(v: u32) -> Rgb {
    Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

impl Rgb {
    pub fn darken(self, amount: f32) -> Rgb {
        let k = 1.0 - amount;
        Rgb((self.0 as f32 * k) as u8, (self.1 as f32 * k) as u8, (self.2 as f32 * k) as u8)
    }

    pub fn lighten(self, by: u8) -> Rgb {
        Rgb(self.0.saturating_add(by), self.1.saturating_add(by), self.2.saturating_add(by))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Plain,
    Solid,
    Crate,
    Ball,
    Dot,
    Ring,
    Star,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    /// `x` is the center cell.
    Center,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Clear { bg: Rgb },
    Fill { x: i32, y: i32, w: i32, h: i32, bg: Rgb },
    Frame { x: i32, y: i32, w: i32, h: i32, fg: Rgb, bg: Option<Rgb> },
    Tile { x: i32, y: i32, kind: Kind, fg: Rgb, bg: Option<Rgb> },
    Text { x: i32, y: i32, text: String, fg: Rgb, bg: Option<Rgb>, align: Align },
    /// Darken everything drawn before this node.
    Dim { amount: f32 },
}

impl Node {
    /// Text is measured in half-cells, one character each.
    pub fn start_column(x: i32, text: &str, align: Align) -> i32 {
        match align {
            Align::Left => 2 * x,
            Align::Center => 2 * x - text.chars().count() as i32 / 2,
        }
    }
}

pub fn clear(bg: Rgb) -> Node {
    Node::Clear { bg }
}

pub fn fill(x: i32, y: i32, w: i32, h: i32, bg: Rgb) -> Node {
    Node::Fill { x, y, w, h, bg }
}

pub fn frame(x: i32, y: i32, w: i32, h: i32, fg: Rgb, bg: Option<Rgb>) -> Node {
    Node::Frame { x, y, w, h, fg, bg }
}

pub fn tile(x: i32, y: i32, kind: Kind, fg: Rgb) -> Node {
    Node::Tile { x, y, kind, fg, bg: None }
}

pub fn text(x: i32, y: i32, text: impl Into<String>, fg: Rgb) -> Node {
    Node::Text { x, y, text: text.into(), fg, bg: None, align: Align::Left }
}

pub fn centered(x: i32, y: i32, text: impl Into<String>, fg: Rgb) -> Node {
    Node::Text { x, y, text: text.into(), fg, bg: None, align: Align::Center }
}

pub fn dim(amount: f32) -> Node {
    Node::Dim { amount }
}
