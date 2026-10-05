# coding: utf-8

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'examples' / 'crates'))

from arche import Runtime                     # noqa: E402
from arche.shells.terminal import to_text     # noqa: E402
from board import Board, solve                # noqa: E402
from levels import LEVELS                     # noqa: E402
from states import (H, W, Banner, LevelSelect, Play, Say, Title,   # noqa: E402
                    Win, STEPS)


@pytest.mark.parametrize('level', LEVELS, ids=lambda l: l.name)
def test_stored_solutions_solve_their_level(level):
    board = Board.parse(level.map)
    assert len(board.boxes) == len(board.goals)
    for step in level.solution:
        board = board.move(STEPS[step])
        assert board is not None
    assert board.solved


@pytest.mark.parametrize('level', LEVELS[:5], ids=lambda l: l.name)
def test_stored_solutions_are_optimal(level):
    assert len(solve(Board.parse(level.map))) == len(level.solution)


class Driver:
    def __init__(self, **kw):
        self.rt = Runtime(Title(), size=(W, H), **kw)

    @property
    def top(self):
        return self.rt.stack.top.state

    def press(self, *actions):
        for action in actions:
            self.rt.send(action)
            self.rt.tick()
            to_text(self.rt.view(), self.rt.size)   # every frame must render

    def until(self, kind, limit=3000):
        for _ in range(limit):
            if isinstance(self.top, kind):
                return self.top
            if isinstance(self.top, Say):
                self.press('confirm')
            else:
                self.rt.tick()
        raise AssertionError(f'never reached {kind.__name__}: {self.rt.stack.frames}')


def play_first_two_shifts(d: Driver):
    d.press('confirm')
    d.until(LevelSelect)
    d.press('confirm')
    d.until(Banner)
    d.press('confirm')
    d.press('right', 'left', 'right', 'right', 'right')   # a wasted step: 5 moves vs par 3
    win = d.until(Win)
    assert (win.moves, win.par) == (5, 3)
    d.press('confirm')                                  # next shift
    play = d.until(Play)
    assert play.index == 1


def test_a_whole_session():
    d = Driver()
    play_first_two_shifts(d)
    assert d.rt.save == {'seen_intro': True, 'best': {0: 5}}

    d.until(Play)
    d.rt.run_headless(max_ticks=d.rt.ticks + 120)   # let the banner go
    d.press(*[STEPS[c] for c in LEVELS[1].solution])
    d.until(Win)
    d.press('down', 'confirm')                      # retry for fewer moves
    assert d.top.moves == 0
    d.press('back', 'up', 'confirm')                # pause, back to shifts
    select = d.until(LevelSelect)
    assert select.unlocked(2) and not select.unlocked(3)
    d.press('back', 'back')
    assert not d.rt.running


def test_robot_shows_the_way_and_hands_back_a_fresh_board():
    d = Driver()
    play_first_two_shifts(d)
    d.rt.run_headless(max_ticks=d.rt.ticks + 120)
    d.press('hint')
    d.rt.run_headless(max_ticks=d.rt.ticks + 30)
    assert d.top.play.moves == 0 and d.top.play.board != Board.parse(LEVELS[1].map)
    d.until(Play)
    assert d.top.board == Board.parse(LEVELS[1].map)


def test_replays_reproduce_a_session(tmp_path):
    live = Driver()
    play_first_two_shifts(live)
    live.rt.save_log(tmp_path / 'run.json')

    seed, log = Runtime.load_log(tmp_path / 'run.json')
    again = Driver(seed=seed, replay=log)
    again.rt.run_headless(max_ticks=live.rt.ticks)
    assert again.rt.save == live.rt.save
    assert to_text(again.rt.view(), again.rt.size) == to_text(live.rt.view(), live.rt.size)
