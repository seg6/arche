# coding: utf-8
"""Platform-neutral scene nodes.

Everything is laid out on a grid of cells. A cell is square and holds one
tile, or two characters of text. Each shell decides what a cell looks like:
pixels in a window, two columns in a terminal, one LED on a fridge.

Colors are "#rrggbb" strings.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, TypeAlias

Kind = Literal['plain', 'solid', 'crate', 'ball', 'dot', 'ring', 'star']


@dataclass(frozen=True, slots=True)
class Clear:
    bg: str


@dataclass(frozen=True, slots=True)
class Fill:
    x: int
    y: int
    w: int
    h: int
    bg: str


@dataclass(frozen=True, slots=True)
class Frame:
    x: int
    y: int
    w: int
    h: int
    fg: str
    bg: str | None = None


@dataclass(frozen=True, slots=True)
class Tile:
    x: int
    y: int
    kind: Kind
    fg: str
    bg: str | None = None


@dataclass(frozen=True, slots=True)
class Text:
    """`x` is in cells. With align='center', `x` is the center cell."""
    x: int
    y: int
    text: str
    fg: str
    bg: str | None = None
    align: Literal['left', 'center'] = 'left'

    def start_column(self) -> int:
        """Text is measured in half-cells (one character each)."""
        return 2 * self.x - (len(self.text) // 2 if self.align == 'center' else 0)


@dataclass(frozen=True, slots=True)
class Dim:
    """Darken everything drawn before this node."""
    amount: float = 0.5


Node: TypeAlias = Clear | Fill | Frame | Tile | Text | Dim


def rgb(color: str) -> tuple[int, int, int]:
    return int(color[1:3], 16), int(color[3:5], 16), int(color[5:7], 16)


def darken(color: tuple[int, int, int], amount: float) -> tuple[int, int, int]:
    r, g, b = color
    k = 1.0 - amount
    return int(r * k), int(g * k), int(b * k)
