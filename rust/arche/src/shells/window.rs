//! A window shell (macroquad).

use std::f32::consts::PI;

use macroquad::prelude::*;

use super::action_for;
use crate::runtime::Runtime;
use crate::scene::{Kind, Node, Rgb};

fn color(c: Rgb) -> Color {
    Color::from_rgba(c.0, c.1, c.2, 255)
}

fn key_name(code: KeyCode) -> Option<&'static str> {
    use KeyCode::*;
    Some(match code {
        Up => "up", Down => "down", Left => "left", Right => "right",
        Enter | KpEnter => "return", Space => "space", Escape => "escape",
        Tab => "tab", Backspace => "backspace", F1 => "f1", GraveAccent => "`", Slash => "?",
        A => "a", D => "d", H => "h", J => "j", K => "k", L => "l", P => "p",
        Q => "q", R => "r", S => "s", U => "u", W => "w", Z => "z",
        _ => return None,
    })
}

pub struct Renderer {
    cell: f32,
    font: Option<Font>,
    font_size: u16,
    ascent: f32,
}

impl Renderer {
    /// Uses the first monospace font it finds, or macroquad's built-in one.
    pub async fn new(cell: f32) -> Self {
        let candidates = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/System/Library/Fonts/Menlo.ttc",
            "C:\\Windows\\Fonts\\consola.ttf",
        ];
        let mut font = None;
        for path in candidates {
            if let Ok(f) = load_ttf_font(path).await {
                font = Some(f);
                break;
            }
        }
        // size the font so one character fills half a cell
        let probe = measure_text("M", font.as_ref(), cell as u16, 1.0).width.max(1.0);
        let font_size = (cell * cell * 0.46 / probe) as u16;
        let ascent = measure_text("M", font.as_ref(), font_size, 1.0).offset_y;
        Renderer { cell, font, font_size, ascent }
    }

    pub fn draw(&self, nodes: &[Node]) {
        let c = self.cell;
        for node in nodes {
            match node {
                Node::Clear { bg } => clear_background(color(*bg)),

                Node::Fill { x, y, w, h, bg } => {
                    draw_rectangle(*x as f32 * c, *y as f32 * c, *w as f32 * c, *h as f32 * c, color(*bg))
                }

                Node::Frame { x, y, w, h, fg, bg } => {
                    let (px, py, pw, ph) = (*x as f32 * c, *y as f32 * c, *w as f32 * c, *h as f32 * c);
                    if let Some(bg) = bg {
                        draw_rectangle(px + c / 8.0, py + c / 8.0, pw - c / 4.0, ph - c / 4.0, color(*bg));
                    }
                    draw_rectangle_lines(px + c / 2.0 - 2.0, py + c / 2.0 - 2.0, pw - c + 4.0, ph - c + 4.0, 3.0, color(*fg));
                }

                Node::Tile { x, y, kind, fg, bg } => {
                    let (px, py) = (*x as f32 * c, *y as f32 * c);
                    if let Some(bg) = bg {
                        draw_rectangle(px, py, c, c, color(*bg));
                    }
                    self.tile(px, py, *kind, *fg);
                }

                Node::Text { x, y, text, fg, bg, align } => {
                    let half = c / 2.0;
                    let start = Node::start_column(*x, text, *align) as f32;
                    if let Some(bg) = bg {
                        draw_rectangle(start * half, *y as f32 * c, text.chars().count() as f32 * half, c, color(*bg));
                    }
                    let params = TextParams {
                        font: self.font.as_ref(),
                        font_size: self.font_size,
                        color: color(*fg),
                        ..Default::default()
                    };
                    let mut buf = [0u8; 4];
                    for (i, ch) in text.chars().enumerate() {
                        let s: &str = ch.encode_utf8(&mut buf);
                        let m = measure_text(s, self.font.as_ref(), self.font_size, 1.0);
                        let cx = (start + i as f32) * half + (half - m.width) / 2.0;
                        let baseline = *y as f32 * c + (c + self.ascent) / 2.0;
                        draw_text_ex(s, cx, baseline, params.clone());
                    }
                }

                Node::Dim { amount } => {
                    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, *amount))
                }
            }
        }
    }

    fn tile(&self, px: f32, py: f32, kind: Kind, fg: Rgb) {
        let c = self.cell;
        let (cx, cy) = (px + c / 2.0, py + c / 2.0);
        let (main, dark, light) = (color(fg), color(fg.darken(0.35)), color(fg.lighten(40)));
        match kind {
            Kind::Plain => {}
            Kind::Solid => {
                draw_rectangle(px, py, c, c, dark);
                draw_rectangle(px + 2.0, py, c - 4.0, c - 4.0, main);
            }
            Kind::Crate => {
                let inset = c / 12.0;
                let (x, y, s) = (px + inset, py + inset, c - 2.0 * inset);
                draw_rectangle(x, y + 3.0, s, s, dark);
                draw_rectangle(x, y, s, s, main);
                let (ix, iy, is) = (x + c / 8.0, y + c / 8.0, s - c / 4.0);
                draw_rectangle_lines(ix, iy, is, is, 2.0, dark);
                draw_line(ix, iy, ix + is, iy + is, 2.0, dark);
                draw_line(ix + is, iy, ix, iy + is, 2.0, dark);
            }
            Kind::Ball => {
                draw_circle(cx, cy + 3.0, c * 0.36, dark);
                draw_circle(cx, cy, c * 0.36, main);
                draw_circle(cx - c * 0.12, cy - c * 0.12, c * 0.09, light);
            }
            Kind::Dot => draw_circle(cx, cy, c * 0.13, main),
            Kind::Ring => draw_circle_lines(cx, cy, c * 0.3, 3.0, main),
            Kind::Star => {
                let point = |i: usize| {
                    let r = if i.is_multiple_of(2) { c * 0.42 } else { c * 0.18 };
                    let a = i as f32 * PI / 5.0;
                    vec2(cx + a.sin() * r, cy - a.cos() * r)
                };
                for i in 0..10 {
                    draw_triangle(vec2(cx, cy), point(i), point((i + 1) % 10), main);
                }
            }
        }
    }
}

/// Save what's on screen right now as a PNG.
pub fn screenshot(path: &str) {
    let mut image = get_screen_data();
    for pixel in image.bytes.chunks_exact_mut(4) {
        pixel[3] = 255; // overlays leave alpha behind in the framebuffer
    }
    image.export_png(path);
}

/// Open a window and run until the game quits. `hook` runs after every frame
/// is drawn, with the frame number: handy for tours, bots and screenshots.
pub fn run(
    rt: Runtime,
    title: &str,
    cell: f32,
    keys: &'static [(&'static str, &'static str)],
    mut hook: impl FnMut(&mut Runtime, u64) + 'static,
) {
    let (w, h) = rt.app().size;
    let conf = Conf {
        window_title: title.to_owned(),
        window_width: (w as f32 * cell) as i32,
        window_height: (h as f32 * cell) as i32,
        window_resizable: false,
        ..Default::default()
    };
    macroquad::Window::from_config(conf, async move {
        let mut rt = rt;
        let renderer = Renderer::new(cell).await;
        let mut frame = 0;
        while rt.running() {
            let mut pressed: Vec<_> = get_keys_pressed().into_iter().filter_map(key_name).collect();
            pressed.sort(); // the set is unordered; keep input deterministic
            for key in pressed {
                if let Some(action) = action_for(keys, key) {
                    rt.send(action);
                }
            }
            if !rt.advance(get_frame_time() as f64) {
                break;
            }
            renderer.draw(&rt.view());
            hook(&mut rt, frame);
            frame += 1;
            next_frame().await;
        }
    });
}
