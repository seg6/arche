# coding: utf-8
"""A pygame-ce window shell."""

from __future__ import annotations

import math
import os

os.environ.setdefault('PYGAME_HIDE_SUPPORT_PROMPT', '1')
import pygame as pg

from .. import scene as s
from ..runtime import Runtime
from . import DEFAULT_KEYS


class Renderer:
    def __init__(self, cell: int = 36):
        self.cell = cell
        self.font = self.make_font(cell)
        self.glyphs: dict[tuple[str, tuple], pg.Surface] = {}

    @staticmethod
    def make_font(cell: int) -> pg.font.Font:
        """A monospace font if there is one, sized so a character fills half a cell."""
        path = pg.font.match_font('dejavusansmono,menlo,consolas,liberationmono,monospace')
        probe = pg.font.Font(path, cell)
        return pg.font.Font(path, int(cell * cell * 0.46 / probe.size('M')[0]))

    def rect(self, x, y, w=1, h=1) -> pg.Rect:
        c = self.cell
        return pg.Rect(x * c, y * c, w * c, h * c)

    def draw(self, surf: pg.Surface, nodes: list[s.Node]) -> None:
        c = self.cell
        for node in nodes:
            match node:
                case s.Clear(bg):
                    surf.fill(s.rgb(bg))

                case s.Fill(x, y, w, h, bg):
                    pg.draw.rect(surf, s.rgb(bg), self.rect(x, y, w, h))

                case s.Frame(x, y, w, h, fg, bg):
                    r = self.rect(x, y, w, h)
                    if bg:
                        pg.draw.rect(surf, s.rgb(bg), r.inflate(-c // 4, -c // 4), border_radius=c // 3)
                    pg.draw.rect(surf, s.rgb(fg), r.inflate(-c + 4, -c + 4), 3, border_radius=c // 4)

                case s.Tile(x, y, kind, fg, bg):
                    r = self.rect(x, y)
                    if bg:
                        pg.draw.rect(surf, s.rgb(bg), r)
                    self.tile(surf, r, kind, s.rgb(fg))

                case s.Text(x, y, text, fg, bg):
                    col, half = node.start_column(), c // 2
                    if bg:
                        pg.draw.rect(surf, s.rgb(bg), (col * half, y * c, len(text) * half, c))
                    for i, ch in enumerate(text):   # monospace, whatever the font
                        glyph = self.glyph(ch, s.rgb(fg))
                        surf.blit(glyph, glyph.get_rect(center=((col + i) * half + half // 2, y * c + c // 2)))

                case s.Dim(amount):
                    shade = pg.Surface(surf.get_size(), pg.SRCALPHA)
                    shade.fill((0, 0, 0, int(255 * amount)))
                    surf.blit(shade, (0, 0))

    def glyph(self, ch: str, color: tuple) -> pg.Surface:
        if (key := (ch, color)) not in self.glyphs:
            self.glyphs[key] = self.font.render(ch, True, color)
        return self.glyphs[key]

    def tile(self, surf: pg.Surface, r: pg.Rect, kind: str, fg: tuple) -> None:
        c = self.cell
        light = tuple(min(255, v + 40) for v in fg)
        dark = s.darken(fg, 0.35)
        match kind:
            case 'solid':
                pg.draw.rect(surf, dark, r)
                pg.draw.rect(surf, fg, r.inflate(-4, -4).move(0, -2), border_radius=4)
            case 'crate':
                box = r.inflate(-c // 6, -c // 6)
                pg.draw.rect(surf, dark, box.move(0, 3), border_radius=5)
                pg.draw.rect(surf, fg, box, border_radius=5)
                inner = box.inflate(-c // 4, -c // 4)
                pg.draw.rect(surf, dark, inner, 2, border_radius=3)
                pg.draw.line(surf, dark, inner.topleft, inner.bottomright, 2)
                pg.draw.line(surf, dark, inner.topright, inner.bottomleft, 2)
            case 'ball':
                pg.draw.circle(surf, dark, (r.centerx, r.centery + 3), c * 0.36)
                pg.draw.circle(surf, fg, r.center, c * 0.36)
                pg.draw.circle(surf, light, (r.centerx - c * 0.12, r.centery - c * 0.12), c * 0.09)
            case 'dot':
                pg.draw.circle(surf, fg, r.center, c * 0.13)
            case 'ring':
                pg.draw.circle(surf, fg, r.center, c * 0.3, 3)
            case 'star':
                pts = [
                    (r.centerx + math.sin(i * math.pi / 5) * c * (0.42 if i % 2 == 0 else 0.18),
                     r.centery - math.cos(i * math.pi / 5) * c * (0.42 if i % 2 == 0 else 0.18))
                    for i in range(10)]
                pg.draw.polygon(surf, fg, pts)


def screenshot(rt: Runtime, path: str, cell: int = 36) -> None:
    """Render the current view to an image file, no window needed."""
    pg.display.init()
    pg.font.init()
    w, h = rt.size
    surf = pg.Surface((w * cell, h * cell))
    Renderer(cell).draw(surf, rt.view())
    pg.image.save(surf, path)


def run(rt: Runtime, *, title: str = 'arche', cell: int = 36, fps: int = 60,
        keys: dict[str, str] = DEFAULT_KEYS) -> None:
    pg.display.init()
    pg.font.init()
    w, h = rt.size
    screen = pg.display.set_mode((w * cell, h * cell), vsync=1)
    pg.display.set_caption(title)
    renderer, clock = Renderer(cell), pg.time.Clock()

    try:
        while rt.running:
            for event in pg.event.get():
                if event.type == pg.QUIT:
                    return
                if event.type == pg.KEYDOWN:
                    name = event.unicode if event.unicode in keys else pg.key.name(event.key)
                    if action := keys.get(name):
                        rt.send(action)

            rt.advance(clock.tick(fps) / 1000)
            renderer.draw(screen, rt.view())
            pg.display.flip()
    finally:
        pg.quit()
