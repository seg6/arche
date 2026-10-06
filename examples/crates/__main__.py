# coding: utf-8
"""crates: a tiny Sokoban built on arche.

    python examples/crates                       # a pygame window
    python examples/crates --shell terminal      # the same game, in your terminal
    python examples/crates --record run.json     # save your run as a replay
    python examples/crates --replay run.json     # watch it back (then take over)
    python examples/crates --replay run.json --shell text   # replay headless, print the last frame
"""

import argparse
import sys
from pathlib import Path

try:
    import arche
except ImportError:   # running from a checkout without installing
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
    import arche

from states import H, W, Title


def main() -> None:
    parser = argparse.ArgumentParser(prog='crates', description=__doc__.split('\n')[0])
    parser.add_argument('--shell', choices=['window', 'terminal', 'text'], default='window')
    parser.add_argument('--record', metavar='PATH', help='save your inputs as a replay')
    parser.add_argument('--replay', metavar='PATH', help='play back a recorded run')
    args = parser.parse_args()

    seed, log = arche.Runtime.load_log(args.replay) if args.replay else (0, None)
    rt = arche.Runtime(Title(), size=(W, H), seed=seed, replay=log)

    try:
        match args.shell:
            case 'window':
                from arche.shells import pygame
                pygame.run(rt, title='crates')
            case 'terminal':
                from arche.shells import terminal
                terminal.run(rt)
            case 'text':
                from arche.shells import terminal
                frame, stack = '', []
                while rt.running and rt.replaying:
                    frame, stack = terminal.to_text(rt.view(), rt.size), list(rt.stack.frames)
                    rt.tick()
                if rt.running:
                    rt.run_headless(max_ticks=rt.ticks + 90)   # let the last input play out
                    frame, stack = terminal.to_text(rt.view(), rt.size), list(rt.stack.frames)
                print(frame)
                print(f'\ntick {rt.ticks}, stack: {[f.state for f in stack]}')
    finally:
        if args.record:
            rt.save_log(args.record)
            print(f'saved {len(rt.log)} inputs to {args.record}')


if __name__ == '__main__':
    main()
