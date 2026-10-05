# coding: utf-8
"""Shells connect a Runtime to a platform: they turn keys into actions and
scene nodes into pixels (or characters, or fridge LEDs).

Keys are named the way pygame names them ("up", "return", "f1", "w"), and
every shell translates its input into those names, so one keymap fits all.
"""

DEFAULT_KEYS: dict[str, str] = {
    'up': 'up', 'w': 'up', 'k': 'up',
    'down': 'down', 's': 'down', 'j': 'down',
    'left': 'left', 'a': 'left', 'h': 'left',
    'right': 'right', 'd': 'right', 'l': 'right',
    'return': 'confirm', 'space': 'confirm',
    'escape': 'back', 'p': 'back', 'q': 'back',
    'z': 'undo', 'u': 'undo', 'backspace': 'undo',
    'r': 'restart',
    '?': 'hint', 'tab': 'hint',
    'f1': 'debug', '`': 'debug',
}
