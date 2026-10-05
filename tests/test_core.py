# coding: utf-8

import pytest

from arche import Pop, Push, Quit, Runtime, Set, State, Wait, every, on, scene


class Ask(State):
    overlay = True

    def __init__(self, answer):
        self.answer = answer

    @on('confirm')
    def confirm(self, action):
        return Pop(self.answer)

    def view(self):
        yield scene.Text(0, 0, 'ask?', '#ffffff')


class Root(State):
    def __init__(self):
        self.events = []

    def on_resume(self, result):
        self.events.append(('resume', result))

    @on('ask')
    def ask(self, action):
        return Push(Ask(42))

    @on('bye')
    def bye(self, action):
        return Quit()

    def view(self):
        yield scene.Clear('#000000')


def test_pop_hands_result_to_on_resume():
    rt = Runtime(root := Root())
    for action in ['ask', 'confirm']:
        rt.send(action)
        rt.tick()
    assert root.events == [('resume', 42)]
    assert len(rt.stack) == 1


def test_overlay_view_includes_state_underneath():
    rt = Runtime(Root())
    rt.send('ask')
    rt.tick()
    assert [type(n) for n in rt.view()] == [scene.Clear, scene.Text]


def test_quit_unwinds_everything():
    rt = Runtime(Root())
    for action in ['ask', 'bye']:   # 'bye' goes to Ask, which ignores it
        rt.send(action)
        rt.tick()
    assert rt.running
    rt.stack.apply(Quit())
    assert not rt.running


def test_script_waits_and_receives_pop_results():
    seen = []

    class Scripted(State):
        def script(self):
            yield Wait(0.5)
            seen.append('waited')
            answer = yield Push(Ask('yes'))
            seen.append(answer)
            return Pop()

    rt = Runtime(Root(), step=0.1)
    rt.stack.apply(Push(Scripted()))
    for _ in range(4):
        rt.tick()
    assert seen == []
    rt.tick()
    assert seen == ['waited']
    assert repr(rt.stack.top.state) == 'Ask'

    rt.send('confirm')
    rt.tick()   # Ask pops, Scripted resumes with 'yes' and pops itself
    assert seen == ['waited', 'yes']
    assert repr(rt.stack.top.state) == 'Root'


def test_set_inherits_a_pending_push():
    class Caller(State):
        got = None

        def script(self):
            Caller.got = yield Push(Ask(1))

    rt = Runtime(Caller())
    rt.tick()
    rt.stack.apply(Set(Ask(2)))
    rt.send('confirm')
    rt.tick()
    assert Caller.got == 2


def test_timers_only_tick_on_top():
    class Ticker(State):
        n = 0

        @every(0.25)
        def bump(self):
            self.n += 1

    rt = Runtime(t := Ticker(), step=0.05)
    for _ in range(10):
        rt.tick()
    assert t.n == 2
    rt.stack.apply(Push(Ask(0)))
    for _ in range(10):
        rt.tick()
    assert t.n == 2


def test_wait_outside_script_is_an_error():
    with pytest.raises(TypeError):
        Runtime(Root()).stack.apply(Wait(1))


def test_replay_reproduces_a_run_and_then_hands_back_control():
    def play(rt, inputs):
        for tick in range(30):
            if tick in inputs:
                rt.send(inputs[tick])
            rt.tick()
        return rt

    live = play(Runtime(Root()), {3: 'ask', 9: 'confirm', 20: 'ask'})
    again = play(Runtime(root := Root(), replay=live.log), {5: 'confirm'})
    assert again.log[:3] == live.log
    assert root.events == [('resume', 42)]
    assert not again.replaying
