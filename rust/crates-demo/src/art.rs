//! Palette and drawing helpers. Everything here produces scene nodes.

use arche::scene::{self, Kind, Node, Rgb, hex};

use crate::board::Board;

pub const BG: Rgb = hex(0x1b1626);
pub const FLOOR: Rgb = hex(0x2a2238);
pub const WALL: Rgb = hex(0x5b4d7a);
pub const GOAL: Rgb = hex(0xff6ad5);
pub const CRATE: Rgb = hex(0xe0a458);
pub const CRATE_HOME: Rgb = hex(0x7ee081);
pub const PLAYER: Rgb = hex(0xffd166);
pub const INK: Rgb = hex(0xf4eefa);
pub const MUTED: Rgb = hex(0x9a8fb0);
pub const ACCENT: Rgb = hex(0xff6ad5);
pub const PANEL: Rgb = hex(0x241c33);

const LOGO_COLORS: [Rgb; 6] =
    [hex(0xff6ad5), hex(0xffd166), hex(0x7ee081), hex(0x6ad5ff), hex(0xe0a458), hex(0xc58cff)];

/// A 3x5 tile font, just the letters we need.
fn glyph(letter: char) -> [&'static str; 5] {
    match letter {
        'C' => ["###", "#..", "#..", "#..", "###"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'S' => ["###", "#..", "###", "..#", "###"],
        _ => ["...", "...", "...", "...", "..."],
    }
}

pub fn board(b: &Board, ox: i32, oy: i32, out: &mut Vec<Node>) {
    for (x, y) in b.floor() {
        out.push(Node::Tile { x: ox + x, y: oy + y, kind: Kind::Plain, fg: FLOOR, bg: Some(FLOOR) });
    }
    for &(x, y) in b.walls.iter() {
        out.push(scene::tile(ox + x, oy + y, Kind::Solid, WALL));
    }
    for &(x, y) in b.goals.iter() {
        out.push(scene::tile(ox + x, oy + y, Kind::Dot, GOAL));
    }
    for &(x, y) in &b.boxes {
        let color = if b.goals.contains(&(x, y)) { CRATE_HOME } else { CRATE };
        out.push(scene::tile(ox + x, oy + y, Kind::Crate, color));
    }
    out.push(scene::tile(ox + b.player.0, oy + b.player.1, Kind::Ball, PLAYER));
}

/// Top-left corner that centers `b` in the area from row `top` to `h`.
pub fn centered(b: &Board, (w, h): (i32, i32), top: i32) -> (i32, i32) {
    ((w - b.w) / 2, top + (h - top - b.h) / 2)
}

pub fn logo(word: &str, x: i32, y: i32, t: f64, out: &mut Vec<Node>) {
    for (i, letter) in word.chars().enumerate() {
        // a little hop runs along the word
        let bob = ((t * 2.2 - i as f64 * 0.35).rem_euclid(3.0) < 0.35) as i32;
        for (row, bits) in glyph(letter).iter().enumerate() {
            for (col, bit) in bits.chars().enumerate() {
                if bit == '#' {
                    let (tx, ty) = (x + i as i32 * 4 + col as i32, y + row as i32 - bob);
                    out.push(scene::tile(tx, ty, Kind::Crate, LOGO_COLORS[i % LOGO_COLORS.len()]));
                }
            }
        }
    }
}

pub fn panel(x: i32, y: i32, w: i32, h: i32) -> Node {
    scene::frame(x, y, w, h, ACCENT, Some(PANEL))
}

/// Greedy word wrap.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    for word in text.split(' ') {
        match lines.last_mut() {
            Some(line) if line.len() + 1 + word.len() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}
