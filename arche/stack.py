# coding: utf-8
"""The state stack: a tiny interpreter for effects. No platform code here."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

from .effects import Effects, Pop, Push, Quit, Set, Wait, flatten
from .state import Script, State

if TYPE_CHECKING:
    from .runtime import Runtime
    from .scene import Node


@dataclass(eq=False)
class Frame:
    """A state on the stack, plus the bookkeeping the state shouldn't care about."""
    state: State
    script: Script | None = None
    wait: float | State | None = None   # seconds left, or the child we pushed
    send: Any = None                    # value for the script's next resume
    clocks: dict[str, float] = field(default_factory=dict)


class StateStack:
    def __init__(self, app: Runtime):
        self.app = app
        self.frames: list[Frame] = []

    def __len__(self) -> int:
        return len(self.frames)

    @property
    def top(self) -> Frame | None:
        return self.frames[-1] if self.frames else None

    # effects

    def apply(self, effects: Effects) -> None:
        for effect in flatten(effects):
            match effect:
                case Push(state):
                    if self.frames:
                        self.frames[-1].state.on_pause()
                    self._enter(state)

                case Set(state):
                    old = self._leave()
                    self._enter(state)
                    # a replacement inherits a pending `yield Push(old)`
                    if len(self.frames) > 1 and self.frames[-2].wait is old:
                        self.frames[-2].wait = state

                case Pop(result):
                    popped = self._leave()
                    if parent := self.top:
                        if parent.wait is popped:
                            parent.wait, parent.send = None, result
                        self.apply(parent.state.on_resume(result))

                case Quit():
                    while self.frames:
                        self._leave()

                case Wait():
                    raise TypeError('Wait() only makes sense when yielded from a script')

    def _enter(self, state: State) -> None:
        state.app = self.app
        frame = Frame(state)
        self.frames.append(frame)
        frame.script = state.script()
        self.apply(state.on_enter())

    def _leave(self) -> State:
        if not self.frames:
            raise IndexError('pop from an empty state stack')
        frame = self.frames.pop()
        frame.state.on_exit()
        if frame.script:
            frame.script.close()
        return frame.state

    # per tick, only the top state is live

    def dispatch(self, action: str) -> None:
        if not (frame := self.top):
            return
        state = frame.state
        if name := state._actions.get(action):
            self.apply(getattr(state, name)(action))
        else:
            self.apply(state.on_action(action))

    def update(self, dt: float) -> None:
        if not (frame := self.top):
            return
        state = frame.state

        self.apply(state.update(dt))

        for name, period in state._timers.items():
            acc = frame.clocks.get(name, 0.0) + dt
            while acc >= period and self.top is frame:
                acc -= period
                self.apply(getattr(state, name)())
            frame.clocks[name] = acc

        if self.top is frame:
            self._run_script(frame, dt)

    def _run_script(self, frame: Frame, dt: float) -> None:
        if isinstance(frame.wait, float | int):
            frame.wait -= dt
            if frame.wait > 1e-9:
                return
            frame.wait = None

        while frame.script and frame.wait is None and self.top is frame:
            try:
                effect = frame.script.send(frame.send)
            except StopIteration as stop:
                frame.script = None
                self.apply(stop.value)
                return

            frame.send = None
            match effect:
                case Wait(seconds):
                    frame.wait = seconds - dt   # this tick counts too, like timers
                case Push(child):
                    frame.wait = child
                    self.apply(effect)
                case _:
                    self.apply(effect)

    # drawing: the top state, plus everything visible through overlays

    def view(self) -> list[Node]:
        first = len(self.frames) - 1
        while first > 0 and self.frames[first].state.overlay:
            first -= 1
        return [node for frame in self.frames[first:] for node in frame.state.view()]
