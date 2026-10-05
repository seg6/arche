# coding: utf-8
"""Effects: what a state *wants* to happen, as plain values.

States never touch the stack. Hooks, timers and scripts return (or yield)
effects, and the stack interprets them.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Iterable, TypeAlias

if TYPE_CHECKING:
    from .state import State


@dataclass(frozen=True, slots=True)
class Push:
    """Put `state` on top. A script yielding this resumes with the pop result."""
    state: State


@dataclass(frozen=True, slots=True)
class Pop:
    """Remove the top state, handing `result` to the one underneath."""
    result: Any = None


@dataclass(frozen=True, slots=True)
class Set:
    """Replace the top state with `state`."""
    state: State


@dataclass(frozen=True, slots=True)
class Quit:
    """Unwind the whole stack."""


@dataclass(frozen=True, slots=True)
class Wait:
    """Scripts only: pause for `seconds` of game time."""
    seconds: float


Effect: TypeAlias = Push | Pop | Set | Quit | Wait
Effects: TypeAlias = Effect | Iterable[Effect] | None


def flatten(effects: Effects) -> list[Effect]:
    match effects:
        case None:
            return []
        case Push() | Pop() | Set() | Quit() | Wait():
            return [effects]
        case _:
            return [e for eff in effects for e in flatten(eff)]
