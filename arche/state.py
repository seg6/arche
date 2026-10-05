# coding: utf-8
"""States, plus the `@on` and `@every` decorators."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Callable, ClassVar, Generator, Iterable

from .effects import Effect, Effects

if TYPE_CHECKING:
    from .runtime import Runtime
    from .scene import Node

Script = Generator[Effect, Any, Effects]


def on(*actions: str) -> Callable:
    """Handle one or more actions: `@on("left", "right")`."""
    def mark(fn):
        fn.__arche_on__ = (*getattr(fn, '__arche_on__', ()), *actions)
        return fn
    return mark


def every(seconds: float) -> Callable:
    """Call a method every `seconds` of game time while its state is on top."""
    def mark(fn):
        fn.__arche_every__ = seconds
        return fn
    return mark


class State:
    overlay: ClassVar[bool] = False     # draw the states underneath too
    app: Runtime                        # set by the stack on push

    _actions: ClassVar[dict[str, str]] = {}
    _timers: ClassVar[dict[str, float]] = {}

    def __init_subclass__(cls, **kw):
        super().__init_subclass__(**kw)
        cls._actions, cls._timers = {}, {}
        for klass in reversed(cls.__mro__):
            for name, fn in vars(klass).items():
                for action in getattr(fn, '__arche_on__', ()):
                    cls._actions[action] = name
                if (secs := getattr(fn, '__arche_every__', None)) is not None:
                    cls._timers[name] = secs

    def __repr__(self) -> str:
        return type(self).__name__

    # lifecycle (all optional, all may return effects)
    def on_enter(self) -> Effects: ...
    def on_exit(self) -> None: ...
    def on_pause(self) -> None: ...
    def on_resume(self, result: Any) -> Effects: ...

    # per tick
    def on_action(self, action: str) -> Effects: ...    # fallback for unbound actions
    def update(self, dt: float) -> Effects: ...

    # a generator yielding effects; it starts when the state is pushed
    def script(self) -> Script | None:
        return None

    # what to show: a platform-neutral list of scene nodes
    def view(self) -> Iterable[Node]:
        return ()
