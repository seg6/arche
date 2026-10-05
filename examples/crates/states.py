# coding: utf-8
"""Every screen in the game is a State. None of them know about pygame."""

from __future__ import annotations

import textwrap

from arche import Pop, Push, Quit, Set, State, Wait, every, on
from arche import scene as s

import art
from board import Board
from levels import LEVELS

W, H = 32, 18
STEPS = {'u': 'up', 'd': 'down', 'l': 'left', 'r': 'right'}


def stars_for(moves: int, par: int) -> int:
    return 3 if moves <= par else 2 if moves <= par * 1.5 else 1


# building blocks ---------------------------------------------------------

class Menu(State):
    """A vertical list of choices. Pops with the chosen item unless told otherwise."""
    items: list = []
    labels: dict = {}

    def __init__(self):
        self.cursor = 0

    @on('up')
    def up(self, _):
        self.cursor = (self.cursor - 1) % len(self.items)

    @on('down')
    def down(self, _):
        self.cursor = (self.cursor + 1) % len(self.items)

    @on('confirm')
    def confirm(self, _):
        return self.chosen(self.items[self.cursor])

    def chosen(self, item):
        return Pop(item)

    def menu(self, y: int):
        for i, item in enumerate(self.items):
            label = self.labels.get(item, str(item))
            if i == self.cursor:
                yield s.Text(W // 2, y + i, f'> {label} <', art.ACCENT, align='center')
            else:
                yield s.Text(W // 2, y + i, label, art.INK, align='center')


class Say(State):
    """A dialog box with a typewriter effect. Confirm to skip ahead, then to close."""
    overlay = True

    def __init__(self, text: str, who: str = 'night manager'):
        self.text, self.who, self.shown = text, who, 0.0

    def update(self, dt):
        self.shown += dt * 45

    @on('confirm', 'back')
    def next(self, _):
        if self.shown < len(self.text):
            self.shown = len(self.text)
        else:
            return Pop()

    def view(self):
        yield art.panel(1, H - 5, W - 2, 5)
        yield s.Text(2, H - 5, f' {self.who} ', art.ACCENT, art.PANEL)
        budget = int(self.shown)
        for i, line in enumerate(textwrap.wrap(self.text, 54)):
            yield s.Text(2, H - 4 + i, line[:max(0, budget)], art.INK)
            budget -= len(line) + 1
        if self.shown >= len(self.text) and self.app.ticks // 20 % 2:
            yield s.Text(W - 3, H - 2, '>>', art.MUTED)


class Banner(State):
    """A title card that dismisses itself."""
    overlay = True

    def __init__(self, title: str, subtitle: str):
        self.title, self.subtitle = title, subtitle

    def script(self):
        yield Wait(1.4)
        return Pop()

    @on('confirm')
    def skip(self, _):
        return Pop()

    def view(self):
        yield s.Fill(0, H // 2 - 2, W, 4, art.PANEL)
        yield s.Text(W // 2, H // 2 - 1, self.title, art.ACCENT, align='center')
        yield s.Text(W // 2, H // 2, self.subtitle, art.INK, align='center')


# screens -----------------------------------------------------------------

class Title(State):
    DEMOS = [1, 2, 4]   # levels the attract mode replays

    def __init__(self):
        self.t, self.demo = 0.0, 0
        self.load_demo()

    def load_demo(self):
        level = LEVELS[self.DEMOS[self.demo]]
        self.board, self.moves, self.linger = Board.parse(level.map), list(level.solution), 0

    def update(self, dt):
        self.t += dt

    @every(0.2)
    def attract(self):
        if self.moves:
            self.board = self.board.move(STEPS[self.moves.pop(0)])
        elif (linger := self.linger + 1) > 8:
            self.demo = (self.demo + 1) % len(self.DEMOS)
            self.load_demo()
        else:
            self.linger = linger

    @on('confirm')
    def start(self, _):
        return Push(LevelSelect() if self.app.save.get('seen_intro') else Intro())

    @on('back')
    def bye(self, _):
        return Quit()

    def view(self):
        yield s.Clear(art.BG)
        yield from art.logo('CRATES', 4, 1, self.t)
        yield from art.board(self.board, (W - self.board.w) // 2, 8)
        yield s.Text(W // 2, 15, 'attract mode: replaying a solution', art.MUTED, align='center')
        yield s.Text(W // 2, 17, 'enter play  ·  esc quit  ·  f1 debug', art.INK, align='center')


class Intro(State):
    """A cutscene, written top to bottom as a script."""

    def __init__(self):
        self.board, self.pointing = Board.parse(LEVELS[0].map), None

    def script(self):
        self.app.save['seen_intro'] = True
        yield Wait(0.5)

        self.pointing = 'you'
        yield Push(Say('this is you. welcome to the night shift at the crate warehouse.'))
        self.pointing = 'crate'
        yield Push(Say('crates go on the pink spots. you can push them, but you can never pull.'))
        self.pointing = None

        for step in LEVELS[0].solution:
            yield Wait(0.35)
            self.board = self.board.move(STEPS[step])
        yield Wait(0.6)

        yield Push(Say("that's the whole job. it only gets harder from here."))
        return Set(LevelSelect())

    @on('back')
    def skip(self, _):
        return Set(LevelSelect())

    def view(self):
        yield s.Clear(art.BG)
        ox, oy = art.centered(self.board, (W, H - 5))
        yield from art.board(self.board, ox, oy)

        blink = self.app.ticks // 15 % 2
        if self.pointing == 'you' and blink:
            px, py = self.board.player
            yield s.Tile(ox + px, oy + py, 'ring', art.ACCENT)
        if self.pointing == 'crate' and blink:
            for (x, y) in self.board.boxes | self.board.goals:
                yield s.Tile(ox + x, oy + y, 'ring', art.ACCENT)


class LevelSelect(Menu):
    items = list(range(len(LEVELS)))

    def __init__(self):
        super().__init__()
        self.playing = 0

    def unlocked(self, i: int) -> bool:
        return i == 0 or (i - 1) in self.app.save.get('best', {})

    def chosen(self, i):
        if self.unlocked(i):
            self.playing = i
            return Push(Play(i))

    @on('back')
    def back(self, _):
        return Pop()

    def on_resume(self, result):
        if result == 'next' and self.playing + 1 < len(LEVELS):
            self.cursor = self.playing = self.playing + 1
            return Push(Play(self.playing))

    def view(self):
        yield s.Clear(art.BG)
        yield s.Text(W // 2, 1, 'pick a shift', art.ACCENT, align='center')
        best = self.app.save.get('best', {})

        for i, level in enumerate(LEVELS):
            y = 3 + 2 * i
            if i == self.cursor:
                yield s.Fill(3, y, W - 6, 1, art.PANEL)
                yield s.Tile(3, y, 'ball', art.PLAYER)
            if not self.unlocked(i):
                yield s.Text(5, y, f'{i + 1}  locked', art.MUTED)
                continue

            yield s.Text(5, y, f'{i + 1}  {level.name}', art.INK)
            par = len(level.solution)
            earned = stars_for(best[i], par) if i in best else 0
            for k in range(3):
                yield s.Tile(17 + k, y, 'star', art.PLAYER if k < earned else art.FLOOR)
            yield s.Text(21, y, f'{best[i]} / {par}' if i in best else f'- / {par}', art.MUTED)


class Play(State):
    def __init__(self, index: int):
        self.index, self.level = index, LEVELS[index]
        self.reset()

    def reset(self):
        self.board, self.history = Board.parse(self.level.map), []

    @property
    def moves(self) -> int:
        return len(self.history)

    def script(self):
        yield Push(Banner(f'shift {self.index + 1}', self.level.name))

    @on('up', 'down', 'left', 'right')
    def walk(self, direction):
        if (board := self.board.move(direction)) is None:
            return
        self.history.append(self.board)
        self.board = board
        if board.solved:
            best = self.app.save.setdefault('best', {})
            best[self.index] = min(self.moves, best.get(self.index, self.moves))
            return Push(Win(self.moves, len(self.level.solution), last=self.index == len(LEVELS) - 1))

    @on('undo')
    def undo(self, _):
        if self.history:
            self.board = self.history.pop()

    @on('restart')
    def restart(self, _):
        self.reset()

    @on('hint')
    def robot(self, _):
        return Push(Robot(self))

    @on('back')
    def pause(self, _):
        return Push(Pause())

    def on_resume(self, choice):
        match choice:
            case 'restart' | 'retry':
                self.reset()
            case 'robot':
                return Push(Robot(self))
            case 'menu':
                return Pop()
            case 'next':
                return Pop('next')

    def view(self):
        yield s.Clear(art.BG)
        yield from art.board(self.board, *art.centered(self.board, (W, H - 1), top=2))
        yield s.Text(1, 0, f'shift {self.index + 1}: {self.level.name}', art.INK)
        yield s.Text(W - 9, 0, f'moves {self.moves:>3}', art.INK)
        yield s.Text(W - 9, 1, f'par   {len(self.level.solution):>3}', art.MUTED)
        yield s.Text(W // 2, H - 1, 'arrows move · z undo · r restart · tab robot · esc menu',
                     art.MUTED, align='center')


class Robot(State):
    """Takes the controls of a Play state and replays the solution. Any key stops it."""
    overlay = True

    def __init__(self, play: Play):
        self.play = play

    def script(self):
        self.play.reset()
        yield Wait(0.4)
        for step in self.play.level.solution:
            self.play.board = self.play.board.move(STEPS[step])
            yield Wait(0.16)
        yield Wait(0.8)
        self.play.reset()
        return Pop()

    def on_action(self, _):
        self.play.reset()
        return Pop()

    def view(self):
        yield s.Fill(0, H - 1, W, 1, art.PANEL)
        yield s.Text(W // 2, H - 1, 'the robot is showing you how · any key to stop',
                     art.ACCENT, align='center')


class Pause(Menu):
    overlay = True
    items = ['resume', 'restart', 'robot', 'menu']
    labels = {'resume': 'keep going', 'restart': 'start over',
              'robot': 'watch the robot', 'menu': 'back to shifts'}

    @on('back')
    def resume(self, _):
        return Pop('resume')

    def view(self):
        yield s.Dim(0.6)
        yield art.panel(9, 5, 14, 8)
        yield s.Text(W // 2, 6, 'paused', art.ACCENT, align='center')
        yield from self.menu(8)


class Win(Menu):
    overlay = True
    labels = {'next': 'next shift', 'retry': 'try for fewer moves', 'menu': 'back to shifts'}

    def __init__(self, moves: int, par: int, last: bool):
        super().__init__()
        self.moves, self.par, self.last = moves, par, last
        self.items = ['retry', 'menu'] if last else ['next', 'retry', 'menu']
        self.lit = 0

    def script(self):
        for _ in range(stars_for(self.moves, self.par)):
            yield Wait(0.3)
            self.lit += 1

    @on('back')
    def menu_(self, _):
        return Pop('menu')

    def view(self):
        yield s.Dim(0.5)
        yield art.panel(6, 3, 20, 12)
        title = 'every crate is home!' if self.last else 'shift complete!'
        yield s.Text(W // 2, 4, title, art.ACCENT, align='center')
        for k in range(3):
            yield s.Tile(W // 2 - 2 + k, 6, 'star', art.PLAYER if k < self.lit else art.FLOOR)
        yield s.Text(W // 2, 8, f'{self.moves} moves · par {self.par}', art.INK, align='center')
        yield from self.menu(10)
