# coding: utf-8
"""The runtime: a fixed-timestep, deterministic driver for the stack.

Shells feed it actions and real elapsed time, and ask it what to show.
Input is applied on tick boundaries and logged as (tick, action) pairs, so
a run is fully described by its seed and its log: that's a replay.
"""

from __future__ import annotations

import json
import random
from collections import deque
from pathlib import Path
from typing import Any, Iterable

from . import scene as s
from .effects import Push
from .stack import StateStack
from .state import State

Log = list[tuple[int, str]]


class Runtime:
    def __init__(
            self, root: State, *,
            size: tuple[int, int] = (40, 22),
            step: float = 1 / 60,
            seed: int = 0,
            replay: Log | None = None,
            save: dict[str, Any] | None = None):
        self.size = size
        self.step = step
        self.seed = seed
        self.rng = random.Random(seed)   # use this, not `random`, to stay replayable
        self.save = save if save is not None else {}

        self.ticks = 0
        self.debug = False
        self.log: Log = []
        self._replay = deque(replay or ())
        self._queue: list[str] = []
        self._acc = 0.0

        self.stack = StateStack(self)
        self.stack.apply(Push(root))

    @property
    def running(self) -> bool:
        return bool(self.stack)

    @property
    def replaying(self) -> bool:
        """Live input is ignored until the replay runs out, then you take over."""
        return bool(self._replay)

    @property
    def alpha(self) -> float:
        """How far we are into the next tick, for smooth interpolation."""
        return self._acc / self.step

    def send(self, action: str) -> None:
        if action == 'debug':
            self.debug = not self.debug
        elif not self.replaying:
            self._queue.append(action)

    def advance(self, seconds: float) -> bool:
        """Run as many fixed ticks as `seconds` of real time pays for."""
        self._acc += min(seconds, 0.25)   # don't spiral after a hiccup
        while self._acc >= self.step and self.running:
            self._acc -= self.step
            self.tick()
        return self.running

    def tick(self) -> None:
        actions, self._queue = self._queue, []
        while self._replay and self._replay[0][0] <= self.ticks:
            actions.append(self._replay.popleft()[1])

        for action in actions:
            self.log.append((self.ticks, action))
            self.stack.dispatch(action)

        self.stack.update(self.step)
        self.ticks += 1

    def run_headless(self, max_ticks: int = 1_000_000) -> None:
        while self.running and self.ticks < max_ticks:
            self.tick()

    def view(self) -> list[s.Node]:
        nodes = self.stack.view()
        if self.debug:
            nodes += self._debug_view()
        return nodes

    def _debug_view(self) -> Iterable[s.Node]:
        lines = [f'tick {self.ticks}' + ('  (replay)' if self.replaying else '')]
        for depth, frame in enumerate(self.stack.frames):
            note = ''
            if isinstance(frame.wait, State):
                note = f' -> {frame.wait!r}'
            elif frame.wait is not None:
                note = f' zz {frame.wait:.1f}s'
            elif frame.script:
                note = ' (script)'
            lines.append(f'{"  " * depth}{frame.state!r}{note}')

        w = (max(map(len, lines)) + 5) // 2
        x = self.size[0] - w
        yield s.Frame(x, 0, w, len(lines) + 2, '#ff6ad5', '#1a1023')
        for i, line in enumerate(lines):
            yield s.Text(x + 1, i + 1, line, '#ffd6f5' if i else '#ff6ad5')

    # replays on disk

    def save_log(self, path: str | Path) -> None:
        Path(path).write_text(json.dumps({'seed': self.seed, 'log': self.log}))

    @staticmethod
    def load_log(path: str | Path) -> tuple[int, Log]:
        data = json.loads(Path(path).read_text())
        return data['seed'], [(t, a) for t, a in data['log']]
