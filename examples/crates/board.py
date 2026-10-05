# coding: utf-8
"""Sokoban rules as immutable data. No arche, no platform, just puzzles."""

from __future__ import annotations

from collections import deque
from dataclasses import dataclass, replace

Pos = tuple[int, int]

DIRS: dict[str, Pos] = {'up': (0, -1), 'down': (0, 1), 'left': (-1, 0), 'right': (1, 0)}
LETTERS = {'up': 'u', 'down': 'd', 'left': 'l', 'right': 'r'}


@dataclass(frozen=True, slots=True)
class Board:
    w: int
    h: int
    walls: frozenset[Pos]
    goals: frozenset[Pos]
    boxes: frozenset[Pos]
    player: Pos

    @staticmethod
    def parse(text: str) -> Board:
        """Standard notation: # wall, . goal, $ box, * box on goal, @ player, + player on goal."""
        rows = text.strip('\n').split('\n')
        walls, goals, boxes, player = set(), set(), set(), (0, 0)
        for y, row in enumerate(rows):
            for x, ch in enumerate(row):
                if ch == '#':
                    walls.add((x, y))
                if ch in '.*+':
                    goals.add((x, y))
                if ch in '$*':
                    boxes.add((x, y))
                if ch in '@+':
                    player = (x, y)
        return Board(
            max(map(len, rows)), len(rows),
            frozenset(walls), frozenset(goals), frozenset(boxes), player)

    @property
    def solved(self) -> bool:
        return self.boxes == self.goals

    def move(self, direction: str) -> Board | None:
        """The board after stepping, or None if the move is blocked."""
        dx, dy = DIRS[direction]
        px, py = self.player
        step = (px + dx, py + dy)
        if step in self.walls:
            return None
        if step in self.boxes:
            beyond = (px + 2 * dx, py + 2 * dy)
            if beyond in self.walls or beyond in self.boxes:
                return None
            return replace(self, player=step, boxes=self.boxes - {step} | {beyond})
        return replace(self, player=step)

    def floor(self) -> set[Pos]:
        """Cells reachable from the player, ignoring boxes: the warehouse's inside."""
        seen, todo = {self.player}, [self.player]
        while todo:
            x, y = todo.pop()
            for dx, dy in DIRS.values():
                n = (x + dx, y + dy)
                if n not in seen and n not in self.walls and 0 <= n[0] < self.w and 0 <= n[1] < self.h:
                    seen.add(n)
                    todo.append(n)
        return seen


def solve(board: Board, limit: int = 2_000_000) -> str | None:
    """Breadth-first search for a move-optimal solution, as a string of u/d/l/r."""
    start = (board.player, board.boxes)
    paths: dict = {start: ''}
    todo = deque([board])
    while todo and len(paths) < limit:
        b = todo.popleft()
        path = paths[(b.player, b.boxes)]
        for direction, letter in LETTERS.items():
            n = b.move(direction)
            if n is None or (key := (n.player, n.boxes)) in paths:
                continue
            paths[key] = path + letter
            if n.solved:
                return paths[key]
            todo.append(n)
    return None
