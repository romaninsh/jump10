mod enemy;
mod level;
mod player;

use level::{parse_grid, Level, LEVEL_COLS, LEVEL_ROWS, SPLASH, DEATH};
use macroquad::prelude::*;

const TICK_RATE: f64 = 1.0 / 5.0;
const STATUS_BAR_HEIGHT: f32 = 30.0;

enum GameState {
    Splash,
    Playing,
    Death,
}

fn tile_color(ch: char) -> Option<Color> {
    match ch {
        '#' => Some(Color::new(0.0, 0.8, 0.0, 1.0)),
        '*' => Some(Color::new(1.0, 1.0, 0.0, 1.0)),
        '^' => Some(Color::new(0.0, 0.6, 0.0, 1.0)),
        'z' => Some(Color::new(0.0, 0.5, 1.0, 1.0)),
        '/' => Some(Color::new(0.5, 0.5, 0.0, 1.0)),
        '\\' => Some(Color::new(0.5, 0.5, 0.0, 1.0)),
        _ => None,
    }
}

fn draw_grid(grid: &[Vec<char>], tile_w: f32, tile_h: f32) {
    for (row, line) in grid.iter().enumerate() {
        for (col, &ch) in line.iter().enumerate() {
            let x = col as f32 * tile_w;
            let y = row as f32 * tile_h;

            if ch == '#' {
                if let Some(color) = tile_color(ch) {
                    draw_rectangle(x, y, tile_w, tile_h, color);
                }
            } else if ch == '=' {
                // Checkerboard pattern for hatched platforms
                let color = Color::new(0.0, 0.8, 0.0, 1.0);
                let steps = 4;
                let cw = tile_w / steps as f32;
                let ch = tile_h / steps as f32;
                for cy in 0..steps {
                    for cx in 0..steps {
                        if (cx + cy) % 2 == 0 {
                            draw_rectangle(x + cx as f32 * cw, y + cy as f32 * ch, cw, ch, color);
                        }
                    }
                }
            } else if ch != ' ' {
                let s = String::from(ch);
                let font_size = tile_h * 1.0;
                let color = tile_color(ch).unwrap_or(GREEN);
                let x_off = if cfg!(target_arch = "wasm32") { 0.0 } else { tile_w * 0.15 };
                draw_text(&s, x + x_off, y + tile_h * 0.75, font_size, color);
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Jump10".to_owned(),
        window_width: 960,
        window_height: 416,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let splash_grid = parse_grid(SPLASH);
    let death_grid = parse_grid(DEATH);

    let mut state = GameState::Splash;
    let mut lvl = Level::load(0);
    let mut tick_acc = 0.0f64;

    loop {
        let tile_w = screen_width() / LEVEL_COLS as f32;
        let tile_h = (screen_height() - STATUS_BAR_HEIGHT) / LEVEL_ROWS as f32;

        match state {
            GameState::Splash => {
                clear_background(BLACK);
                draw_grid(&splash_grid, tile_w, tile_h);

                if get_last_key_pressed().is_some() {
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                tick_acc += get_frame_time() as f64;

                if tick_acc >= TICK_RATE {
                    tick_acc -= TICK_RATE;

                    lvl.player.update(&lvl.grid);

                    for e in &mut lvl.enemies {
                        e.update(&lvl.grid);
                    }

                    // Enemy collision
                    for e in &lvl.enemies {
                        if (lvl.player.col - e.col).abs() <= 1 && lvl.player.row == e.row {
                            lvl.player.die();
                        }
                        if lvl.player.col == e.col && (lvl.player.row - e.row).abs() <= 1 {
                            lvl.player.die();
                        }
                    }

                    // Goal reached
                    if lvl.grid[lvl.player.row as usize][lvl.player.col as usize] == '*' {
                        if !lvl.advance() {
                            lvl.restart();
                            state = GameState::Splash;
                        }
                        tick_acc = 0.0;
                    }

                    // Death check
                    if !lvl.player.alive {
                        state = GameState::Death;
                    }
                }

                // Secret skip key
                if is_key_pressed(KeyCode::S) {
                    if !lvl.advance() {
                        lvl.restart();
                        state = GameState::Splash;
                    }
                    tick_acc = 0.0;
                }

                // Draw
                clear_background(BLACK);
                draw_grid(&lvl.grid, tile_w, tile_h);
                lvl.player.draw(tile_w, tile_h);
                for e in &lvl.enemies {
                    e.draw(tile_w, tile_h);
                }

                let status = format!(
                    "Level: {}   Lives: {}   A/D=move  W=jump  P=quit",
                    lvl.idx + 1,
                    lvl.player.lives
                );
                let status_y = LEVEL_ROWS as f32 * tile_h + STATUS_BAR_HEIGHT * 0.75;
                draw_text(&status, 10.0, status_y, STATUS_BAR_HEIGHT * 0.8, GREEN);
            }

            GameState::Death => {
                clear_background(BLACK);
                draw_grid(&death_grid, tile_w, tile_h);

                if get_last_key_pressed().is_some() {
                    lvl.restart();
                    tick_acc = 0.0;
                    state = GameState::Playing;
                }
            }
        }

        // Quit
        if is_key_pressed(KeyCode::P) {
            break;
        }

        next_frame().await
    }
}
