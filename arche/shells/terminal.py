# coding: utf-8
"""A terminal shell: truecolor ANSI, two columns per cell. Unix terminals only."""

from __future__ import annotations

import os
import select
import shutil
import sys
import termios
import time
import tty

from .. import scene as s
from ..runtime import Runtime
from . import DEFAULT_KEYS

GLYPHS = {
    'plain': '  ', 'solid': '██', 'crate': '[]',
    'ball': '◖◗', 'dot': '··', 'ring': '()', 'star': '**'}

Cell = list   # [char, fg, bg]


def rasterize(nodes: list[s.Node], size: tuple[int, int]) -> list[list[Cell]]:
    """Scene nodes to a grid of [char, fg, bg], two columns per cell."""
    w, h = size
    black = (0, 0, 0)
    grid = [[[' ', black, black] for _ in range(2 * w)] for _ in range(h)]

    def put(col, row, ch, fg=None, bg=None):
        if 0 <= row < h and 0 <= col < 2 * w:
            cell = grid[row][col]
            cell[0] = ch
            if fg is not None:
                cell[1] = fg
            if bg is not None:
                cell[2] = bg

    for node in nodes:
        match node:
            case s.Clear(bg):
                for row in grid:
                    for cell in row:
                        cell[:] = [' ', black, s.rgb(bg)]

            case s.Fill(x, y, cw, ch, bg):
                for row in range(y, y + ch):
                    for col in range(2 * x, 2 * (x + cw)):
                        put(col, row, ' ', bg=s.rgb(bg))

            case s.Frame(x, y, cw, ch, fg, bg):
                fg, bg = s.rgb(fg), bg and s.rgb(bg)
                left, right, top, bottom = 2 * x, 2 * (x + cw) - 1, y, y + ch - 1
                for row in range(top, bottom + 1):
                    for col in range(left, right + 1):
                        edge_x, edge_y = col in (left, right), row in (top, bottom)
                        ch_ = (('╭' if col == left else '╮') if row == top else
                               ('╰' if col == left else '╯')) if edge_x and edge_y else \
                            '│' if edge_x else '─' if edge_y else ' '
                        put(col, row, ch_, fg, bg)

            case s.Tile(x, y, kind, fg, bg):
                for i, ch_ in enumerate(GLYPHS[kind]):
                    put(2 * x + i, y, ch_, s.rgb(fg), bg and s.rgb(bg))

            case s.Text(x, y, text, fg, bg):
                for i, ch_ in enumerate(text, node.start_column()):
                    put(i, y, ch_, s.rgb(fg), bg and s.rgb(bg))

            case s.Dim(amount):
                for row in grid:
                    for cell in row:
                        cell[1], cell[2] = s.darken(cell[1], amount), s.darken(cell[2], amount)
    return grid


def to_ansi(grid: list[list[Cell]]) -> str:
    out, last = ['\x1b[H'], None
    for r, row in enumerate(grid):
        out.append(f'\x1b[{r + 1};1H')
        for ch, fg, bg in row:
            if (fg, bg) != last:
                out.append('\x1b[38;2;%d;%d;%dm\x1b[48;2;%d;%d;%dm' % (*fg, *bg))
                last = (fg, bg)
            out.append(ch)
    out.append('\x1b[0m')
    return ''.join(out)


def to_text(nodes: list[s.Node], size: tuple[int, int]) -> str:
    """Plain characters, no color: handy for tests and for pasting into chats."""
    return '\n'.join(''.join(c[0] for c in row).rstrip() for row in rasterize(nodes, size))


SEQUENCES = {
    '\x1b[A': 'up', '\x1b[B': 'down', '\x1b[C': 'right', '\x1b[D': 'left',
    '\x1bOA': 'up', '\x1bOB': 'down', '\x1bOC': 'right', '\x1bOD': 'left',
    '\x1bOP': 'f1', '\x1b[11~': 'f1',
    '\r': 'return', '\n': 'return', ' ': 'space', '\t': 'tab',
    '\x7f': 'backspace', '\x08': 'backspace', '\x1b': 'escape'}


def read_keys(fd: int) -> list[str]:
    data = b''
    while select.select([fd], [], [], 0)[0]:
        if not (chunk := os.read(fd, 64)):
            break
        data += chunk
    text, keys = data.decode(errors='ignore'), []
    while text:
        for seq in sorted(SEQUENCES, key=len, reverse=True):
            if text.startswith(seq):
                keys.append(SEQUENCES[seq])
                text = text[len(seq):]
                break
        else:
            if text[0] == '\x03':
                raise KeyboardInterrupt
            keys.append(text[0].lower())
            text = text[1:]
    return keys


def run(rt: Runtime, *, fps: int = 30, keys: dict[str, str] = DEFAULT_KEYS) -> None:
    w, h = rt.size
    cols, rows = shutil.get_terminal_size()
    if cols < 2 * w or rows < h:
        sys.exit(f'this game wants a {2 * w}x{h} terminal, yours is {cols}x{rows}')

    fd = sys.stdin.fileno()
    saved = termios.tcgetattr(fd)
    write = sys.stdout.write
    write('\x1b[?1049h\x1b[?25l\x1b[2J')
    try:
        tty.setcbreak(fd)
        last, frame = time.perf_counter(), ''
        while rt.running:
            for key in read_keys(fd):
                if action := keys.get(key):
                    rt.send(action)

            now = time.perf_counter()
            rt.advance(now - last)
            last = now

            if (new := to_ansi(rasterize(rt.view(), rt.size))) != frame:
                write(frame := new)
                sys.stdout.flush()
            time.sleep(max(0.0, 1 / fps - (time.perf_counter() - now)))
    except KeyboardInterrupt:
        pass
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, saved)
        write('\x1b[0m\x1b[?25h\x1b[?1049l')
        sys.stdout.flush()
