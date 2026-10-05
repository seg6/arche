# coding: utf-8
"""Palette and drawing helpers. Everything here returns scene nodes."""

from __future__ import annotations

from typing import Iterator

from arche import scene as s

from board import Board

BG = '#1b1626'
FLOOR = '#2a2238'
WALL = '#5b4d7a'
GOAL = '#ff6ad5'
CRATE = '#e0a458'
CRATE_HOME = '#7ee081'
PLAYER = '#ffd166'
INK = '#f4eefa'
MUTED = '#9a8fb0'
ACCENT = '#ff6ad5'
PANEL = '#241c33'

LOGO_COLORS = ['#ff6ad5', '#ffd166', '#7ee081', '#6ad5ff', '#e0a458', '#c58cff']

FONT = {   # 3x5 tile font, just the letters we need
    'C': ['###', '#..', '#..', '#..', '###'],
    'R': ['##.', '#.#', '##.', '#.#', '#.#'],
    'A': ['.#.', '#.#', '###', '#.#', '#.#'],
    'T': ['###', '.#.', '.#.', '.#.', '.#.'],
    'E': ['###', '#..', '##.', '#..', '###'],
    'S': ['###', '#..', '###', '..#', '###'],
}


def board(b: Board, ox: int, oy: int) -> Iterator[s.Node]:
    for (x, y) in b.floor():
        yield s.Tile(ox + x, oy + y, 'plain', FLOOR, FLOOR)
    for (x, y) in b.walls:
        yield s.Tile(ox + x, oy + y, 'solid', WALL)
    for (x, y) in b.goals:
        yield s.Tile(ox + x, oy + y, 'dot', GOAL)
    for (x, y) in b.boxes:
        yield s.Tile(ox + x, oy + y, 'crate', CRATE_HOME if (x, y) in b.goals else CRATE)
    yield s.Tile(ox + b.player[0], oy + b.player[1], 'ball', PLAYER)


def centered(b: Board, size: tuple[int, int], top: int = 0) -> tuple[int, int]:
    w, h = size
    return (w - b.w) // 2, top + (h - top - b.h) // 2


def logo(word: str, x: int, y: int, t: float) -> Iterator[s.Node]:
    for i, letter in enumerate(word):
        bob = 1 if (t * 2.2 - i * 0.35) % 3 < 0.35 else 0   # a little hop runs along the word
        for row, bits in enumerate(FONT[letter]):
            for col, bit in enumerate(bits):
                if bit == '#':
                    yield s.Tile(x + i * 4 + col, y + row - bob, 'crate', LOGO_COLORS[i % len(LOGO_COLORS)])


def panel(x: int, y: int, w: int, h: int) -> s.Node:
    return s.Frame(x, y, w, h, ACCENT, PANEL)
