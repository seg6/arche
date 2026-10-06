//! Rasterize scene nodes to a character grid, two columns per cell. The
//! terminal shell paints this grid; tests and replays print it as plain text.

use crate::scene::{Kind, Node, Rgb};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgb,
    pub bg: Rgb,
}

pub fn glyphs(kind: Kind) -> [char; 2] {
    match kind {
        Kind::Plain => [' ', ' '],
        Kind::Solid => ['█', '█'],
        Kind::Crate => ['[', ']'],
        Kind::Ball => ['◖', '◗'],
        Kind::Dot => ['·', '·'],
        Kind::Ring => ['(', ')'],
        Kind::Star => ['*', '*'],
    }
}

pub fn rasterize(nodes: &[Node], (w, h): (i32, i32)) -> Vec<Vec<Cell>> {
    let black = Rgb(0, 0, 0);
    let blank = Cell { ch: ' ', fg: black, bg: black };
    let mut grid = vec![vec![blank; 2 * w as usize]; h as usize];

    let put = |grid: &mut Vec<Vec<Cell>>, col: i32, row: i32, ch: char, fg: Option<Rgb>, bg: Option<Rgb>| {
        if (0..h).contains(&row) && (0..2 * w).contains(&col) {
            let cell = &mut grid[row as usize][col as usize];
            cell.ch = ch;
            if let Some(fg) = fg {
                cell.fg = fg;
            }
            if let Some(bg) = bg {
                cell.bg = bg;
            }
        }
    };

    for node in nodes {
        match node {
            Node::Clear { bg } => {
                for cell in grid.iter_mut().flatten() {
                    *cell = Cell { ch: ' ', fg: black, bg: *bg };
                }
            }
            Node::Fill { x, y, w, h, bg } => {
                for row in *y..y + h {
                    for col in 2 * x..2 * (x + w) {
                        put(&mut grid, col, row, ' ', None, Some(*bg));
                    }
                }
            }
            Node::Frame { x, y, w, h, fg, bg } => {
                let (left, right, top, bottom) = (2 * x, 2 * (x + w) - 1, *y, y + h - 1);
                for row in top..=bottom {
                    for col in left..=right {
                        let ch = match (col == left, col == right, row == top, row == bottom) {
                            (true, _, true, _) => '╭',
                            (_, true, true, _) => '╮',
                            (true, _, _, true) => '╰',
                            (_, true, _, true) => '╯',
                            (true, ..) | (_, true, ..) => '│',
                            (_, _, true, _) | (.., true) => '─',
                            _ => ' ',
                        };
                        put(&mut grid, col, row, ch, Some(*fg), *bg);
                    }
                }
            }
            Node::Tile { x, y, kind, fg, bg } => {
                for (i, ch) in glyphs(*kind).into_iter().enumerate() {
                    put(&mut grid, 2 * x + i as i32, *y, ch, Some(*fg), *bg);
                }
            }
            Node::Text { x, y, text, fg, bg, align } => {
                let start = Node::start_column(*x, text, *align);
                for (i, ch) in text.chars().enumerate() {
                    put(&mut grid, start + i as i32, *y, ch, Some(*fg), *bg);
                }
            }
            Node::Dim { amount } => {
                for cell in grid.iter_mut().flatten() {
                    cell.fg = cell.fg.darken(*amount);
                    cell.bg = cell.bg.darken(*amount);
                }
            }
        }
    }
    grid
}

/// Plain characters, no color: handy for tests and for pasting into chats.
pub fn to_text(nodes: &[Node], size: (i32, i32)) -> String {
    rasterize(nodes, size)
        .iter()
        .map(|row| row.iter().map(|c| c.ch).collect::<String>().trim_end().to_owned())
        .collect::<Vec<_>>()
        .join("\n")
}
