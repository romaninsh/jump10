use macroquad::prelude::*;

const SLIDE_ON_DURATION: f32 = 0.8;
const PAUSE1_DURATION: f32 = 1.5;
const SLIDE_LEFT_SPEED: f32 = 30.0; // pixels per second
const PAUSE2_DURATION: f32 = 1.5;
const SLIDE_OFF_DURATION: f32 = 0.8;
const PAUSE3_DURATION: f32 = 0.8;
const BORDER_RADIUS: f32 = 4.0;
const BORDER_COLOR: Color = Color::new(0.0, 0.8, 0.0, 0.6);
const TEXT_COLOR: Color = Color::new(0.0, 1.0, 0.0, 1.0);
const PADDING: f32 = 6.0;

enum Phase {
    SlideOn(f32),
    Pause1(f32),
    SlideLeft(f32),
    Pause2(f32),
    SlideOff(f32),
    Pause3(f32),
}

pub struct Scroller {
    col_start: usize,
    col_end: usize,
    row_start: usize,
    row_end: usize,
    lines: Vec<String>,
    current_line: usize,
    phase: Phase,
}

pub fn spawn(grid: &mut Vec<Vec<char>>, level_text: &str) -> Option<Scroller> {
    let mut min_r = usize::MAX;
    let mut max_r = 0usize;
    let mut min_c = usize::MAX;
    let mut max_c = 0usize;
    let mut found = false;

    for (r, row) in grid.iter().enumerate() {
        for (c, &ch) in row.iter().enumerate() {
            if ch == '!' {
                found = true;
                min_r = min_r.min(r);
                max_r = max_r.max(r);
                min_c = min_c.min(c);
                max_c = max_c.max(c);
            }
        }
    }

    if !found {
        return None;
    }

    for row in grid.iter_mut() {
        for ch in row.iter_mut() {
            if *ch == '!' {
                *ch = ' ';
            }
        }
    }

    let lines: Vec<String> = level_text
        .lines()
        .skip(13)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    if lines.is_empty() {
        return None;
    }

    Some(Scroller {
        col_start: min_c,
        col_end: max_c + 1,
        row_start: min_r,
        row_end: max_r + 1,
        lines,
        current_line: 0,
        phase: Phase::SlideOn(0.0),
    })
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Scroller {
    pub fn update(&mut self, dt: f32) {
        let slide_dur = self.slide_left_duration();
        match self.phase {
            Phase::SlideOn(ref mut t) => {
                *t += dt / SLIDE_ON_DURATION;
                if *t >= 1.0 {
                    self.phase = Phase::Pause1(0.0);
                }
            }
            Phase::Pause1(ref mut t) => {
                *t += dt / PAUSE1_DURATION;
                if *t >= 1.0 {
                    self.phase = Phase::SlideLeft(0.0);
                }
            }
            Phase::SlideLeft(ref mut t) => {
                if slide_dur <= 0.0 {
                    self.phase = Phase::Pause2(0.0);
                } else {
                    *t += dt / slide_dur;
                    if *t >= 1.0 {
                        self.phase = Phase::Pause2(0.0);
                    }
                }
            }
            Phase::Pause2(ref mut t) => {
                *t += dt / PAUSE2_DURATION;
                if *t >= 1.0 {
                    self.phase = Phase::SlideOff(0.0);
                }
            }
            Phase::SlideOff(ref mut t) => {
                *t += dt / SLIDE_OFF_DURATION;
                if *t >= 1.0 {
                    self.phase = Phase::Pause3(0.0);
                }
            }
            Phase::Pause3(ref mut t) => {
                *t += dt / PAUSE3_DURATION;
                if *t >= 1.0 {
                    self.current_line = (self.current_line + 1) % self.lines.len();
                    self.phase = Phase::SlideOn(0.0);
                }
            }
        }
    }

    fn slide_left_duration(&self) -> f32 {
        // We can't know tile_w here, so estimate from screen width
        let tile_w = screen_width() / 60.0;
        let rw = (self.col_end - self.col_start) as f32 * tile_w;
        let rh = (self.row_end - self.row_start) as f32
            * ((screen_height() - 30.0) / 13.0);
        let font_size = rh * 0.85;
        let line = &self.lines[self.current_line];
        let text_w = measure_text(line, None, font_size as u16, 1.0).width;
        let inner_w = rw - PADDING * 2.0;
        let overflow = text_w - inner_w;
        if overflow <= 0.0 {
            0.0
        } else {
            overflow / SLIDE_LEFT_SPEED
        }
    }

    pub fn draw(&self, tile_w: f32, tile_h: f32) {
        let rx = self.col_start as f32 * tile_w;
        let ry = self.row_start as f32 * tile_h;
        let rw = (self.col_end - self.col_start) as f32 * tile_w;
        let rh = (self.row_end - self.row_start) as f32 * tile_h;

        // Black background for the scroller box
        draw_rectangle(rx, ry, rw, rh, BLACK);

        let line = &self.lines[self.current_line];
        let font_size = rh * 0.85;
        let text_w = measure_text(line, None, font_size as u16, 1.0).width;
        let inner_w = rw - PADDING * 2.0;
        let fits = text_w <= inner_w;

        // Compute text position
        let (text_x, text_y_offset) = match self.phase {
            Phase::SlideOn(t) => {
                let eased = smoothstep(t);
                let x = if fits {
                    rx + (rw - text_w) / 2.0
                } else {
                    rx + PADDING
                };
                (x, rh * (1.0 - eased))
            }
            Phase::Pause1(_) => {
                let x = if fits {
                    rx + (rw - text_w) / 2.0
                } else {
                    rx + PADDING
                };
                (x, 0.0)
            }
            Phase::SlideLeft(t) => {
                if fits {
                    (rx + (rw - text_w) / 2.0, 0.0)
                } else {
                    let overflow = text_w - inner_w;
                    let eased = smoothstep(t);
                    (rx + PADDING - overflow * eased, 0.0)
                }
            }
            Phase::Pause2(_) => {
                if fits {
                    (rx + (rw - text_w) / 2.0, 0.0)
                } else {
                    let overflow = text_w - inner_w;
                    (rx + PADDING - overflow, 0.0)
                }
            }
            Phase::SlideOff(t) => {
                let eased = smoothstep(t);
                let x = if fits {
                    rx + (rw - text_w) / 2.0
                } else {
                    let overflow = text_w - inner_w;
                    rx + PADDING - overflow
                };
                (x, -rh * eased)
            }
            Phase::Pause3(_) => {
                // Nothing visible, just draw border
                self.draw_rounded_border(rx, ry, rw, rh);
                return;
            }
        };

        let text_y = ry + text_y_offset + rh / 2.0 + font_size * 0.3;

        // Clip text to box using GL scissor
        {
            let mut gl = unsafe { get_internal_gl() };
            gl.flush();
            gl.quad_gl.scissor(Some((rx as i32, ry as i32, rw as i32, rh as i32)));
        }

        draw_text(line, text_x, text_y, font_size, TEXT_COLOR);

        {
            let mut gl = unsafe { get_internal_gl() };
            gl.flush();
            gl.quad_gl.scissor(None);
        }

        // Border on top
        self.draw_rounded_border(rx, ry, rw, rh);
    }

    fn draw_rounded_border(&self, x: f32, y: f32, w: f32, h: f32) {
        let r = BORDER_RADIUS;
        let thick = 1.5;

        draw_line(x + r, y, x + w - r, y, thick, BORDER_COLOR);
        draw_line(x + r, y + h, x + w - r, y + h, thick, BORDER_COLOR);
        draw_line(x, y + r, x, y + h - r, thick, BORDER_COLOR);
        draw_line(x + w, y + r, x + w, y + h - r, thick, BORDER_COLOR);

        // Quarter-arc corners
        let segments = 6;
        let pi = std::f32::consts::PI;
        let half_pi = std::f32::consts::FRAC_PI_2;
        // (center_x, center_y, start_angle)
        let corners = [
            (x + r,     y + r,     pi),          // top-left: PI to 3PI/2
            (x + w - r, y + r,     pi + half_pi), // top-right: 3PI/2 to 2PI
            (x + w - r, y + h - r, 0.0),          // bottom-right: 0 to PI/2
            (x + r,     y + h - r, half_pi),       // bottom-left: PI/2 to PI
        ];
        for &(cx, cy, start) in &corners {
            for i in 0..segments {
                let a1 = start + half_pi * (i as f32 / segments as f32);
                let a2 = start + half_pi * ((i + 1) as f32 / segments as f32);
                draw_line(
                    cx + r * a1.cos(), cy + r * a1.sin(),
                    cx + r * a2.cos(), cy + r * a2.sin(),
                    thick, BORDER_COLOR,
                );
            }
        }
    }
}
