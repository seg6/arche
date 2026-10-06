//! Sokoban rules as immutable data. No arche, no platform, just puzzles.

use std::collections::BTreeSet;
#[cfg(test)]
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

pub type Pos = (i32, i32);

pub const DIRS: [(&str, char, Pos); 4] =
    [("up", 'u', (0, -1)), ("down", 'd', (0, 1)), ("left", 'l', (-1, 0)), ("right", 'r', (1, 0))];

pub fn delta(direction: &str) -> Option<Pos> {
    DIRS.iter().find(|(name, ..)| *name == direction).map(|(.., d)| *d)
}

pub fn direction(letter: char) -> &'static str {
    DIRS.iter().find(|(_, l, _)| *l == letter).map(|(name, ..)| *name).expect("u/d/l/r")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub w: i32,
    pub h: i32,
    pub walls: Rc<BTreeSet<Pos>>,   // shared between every board of a level
    pub goals: Rc<BTreeSet<Pos>>,
    pub boxes: BTreeSet<Pos>,
    pub player: Pos,
}

impl Board {
    /// Standard notation: # wall, . goal, $ box, * box on goal, @ player, + player on goal.
    pub fn parse(text: &str) -> Board {
        let rows: Vec<&str> = text.trim_matches('\n').lines().collect();
        let (mut walls, mut goals, mut boxes, mut player) =
            (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), (0, 0));
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let p = (x as i32, y as i32);
                if ch == '#' {
                    walls.insert(p);
                }
                if ".*+".contains(ch) {
                    goals.insert(p);
                }
                if "$*".contains(ch) {
                    boxes.insert(p);
                }
                if "@+".contains(ch) {
                    player = p;
                }
            }
        }
        Board {
            w: rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32,
            h: rows.len() as i32,
            walls: Rc::new(walls),
            goals: Rc::new(goals),
            boxes,
            player,
        }
    }

    pub fn solved(&self) -> bool {
        self.boxes.iter().eq(self.goals.iter())
    }

    /// The board after stepping, or None if the move is blocked.
    pub fn step(&self, direction: &str) -> Option<Board> {
        let (dx, dy) = delta(direction)?;
        let (px, py) = self.player;
        let next = (px + dx, py + dy);
        if self.walls.contains(&next) {
            return None;
        }
        let mut board = self.clone();
        if self.boxes.contains(&next) {
            let beyond = (px + 2 * dx, py + 2 * dy);
            if self.walls.contains(&beyond) || self.boxes.contains(&beyond) {
                return None;
            }
            board.boxes.remove(&next);
            board.boxes.insert(beyond);
        }
        board.player = next;
        Some(board)
    }

    /// Cells reachable from the player, ignoring boxes: the warehouse's inside.
    pub fn floor(&self) -> BTreeSet<Pos> {
        let mut seen = BTreeSet::from([self.player]);
        let mut todo = vec![self.player];
        while let Some((x, y)) = todo.pop() {
            for (.., (dx, dy)) in DIRS {
                let n = (x + dx, y + dy);
                let inside = (0..self.w).contains(&n.0) && (0..self.h).contains(&n.1);
                if inside && !self.walls.contains(&n) && seen.insert(n) {
                    todo.push(n);
                }
            }
        }
        seen
    }
}

/// Breadth-first search for a move-optimal solution, as a string of u/d/l/r.
#[cfg(test)]
pub fn solve(board: &Board) -> Option<String> {
    let key = |b: &Board| (b.player, b.boxes.clone());
    let mut paths = HashMap::from([(key(board), String::new())]);
    let mut todo = VecDeque::from([board.clone()]);
    while let Some(b) = todo.pop_front() {
        let path = paths[&key(&b)].clone();
        for (name, letter, _) in DIRS {
            let Some(n) = b.step(name) else { continue };
            if paths.contains_key(&key(&n)) {
                continue;
            }
            let p = format!("{path}{letter}");
            if n.solved() {
                return Some(p);
            }
            paths.insert(key(&n), p);
            todo.push_back(n);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::LEVELS;

    #[test]
    fn stored_solutions_solve_their_level() {
        for level in LEVELS {
            let mut board = Board::parse(level.map);
            assert_eq!(board.boxes.len(), board.goals.len(), "{}", level.name);
            for letter in level.solution.chars() {
                board = board.step(direction(letter)).unwrap_or_else(|| panic!("{} blocked", level.name));
            }
            assert!(board.solved(), "{}", level.name);
        }
    }

    #[test]
    fn stored_solutions_are_optimal() {
        for level in LEVELS {
            let found = solve(&Board::parse(level.map)).unwrap();
            assert_eq!(found.len(), level.solution.len(), "{}", level.name);
        }
    }
}
