mod enemy;
mod level;
mod music;
mod platform;
mod player;

use level::{DEATH, LEVEL_COLS, LEVEL_ROWS, Level, SPLASH, parse_grid};
use macroquad::prelude::*;
use music::Music;

const TICK_RATE: f64 = 1.0 / 5.0;
const STATUS_BAR_HEIGHT: f32 = 30.0;

enum GameState {
    Splash,
    Intro(f32),  // scroll_y offset
    Playing,
    Death,
    Win(f32),    // scroll_y offset
    Cheater,
}

const INTRO_TEXT: &str = include_str!("../splash/scroller.txt");
const WIN_TEXT: &str = include_str!("../splash/win.txt");
const CHEATER: &str = include_str!("../splash/cheater.txt");

const INTRO_SCROLL_SPEED: f32 = 30.0;
fn tile_color(ch: char) -> Option<Color> {
    match ch {
        '#' => Some(Color::new(0.0, 0.8, 0.0, 1.0)),
        '*' => Some(Color::new(1.0, 1.0, 0.0, 1.0)),
        's' => Some(Color::new(1.0, 0.0, 1.0, 1.0)),
        '^' | 'v' | '>' | '<' => Some(Color::new(0.0, 0.6, 0.0, 1.0)),
        'o' => Some(Color::new(1.0, 0.84, 0.0, 1.0)),
        '%' => Some(Color::new(0.6, 0.6, 0.6, 1.0)),
        'k' => Some(Color::new(0.0, 1.0, 1.0, 1.0)),
        'z' => Some(Color::new(0.0, 0.5, 1.0, 1.0)),
        'H' => Some(Color::new(0.5, 0.3, 0.0, 1.0)),
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
                let x_off = if cfg!(target_arch = "wasm32") {
                    0.0
                } else {
                    tile_w * 0.15
                };
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
    let cheater_grid = parse_grid(CHEATER);

    let music = Music::new();

    let mut state = GameState::Splash;
    let mut lvl = Level::load(0);
    let mut tick_acc = 0.0f64;
    let mut skipped_any = false;

    loop {
        let tile_w = screen_width() / LEVEL_COLS as f32;
        let tile_h = (screen_height() - STATUS_BAR_HEIGHT) / LEVEL_ROWS as f32;

        match state {
            GameState::Splash => {
                clear_background(BLACK);
                draw_grid(&splash_grid, tile_w, tile_h);

                if get_last_key_pressed().is_some() {
                    state = GameState::Intro(screen_height());
                }
            }

            GameState::Intro(ref mut scroll_y) => {
                clear_background(BLACK);

                let font_size = 28.0;
                let line_height = font_size * 1.4;
                let lines: Vec<&str> = INTRO_TEXT.lines().collect();
                let total_height = lines.len() as f32 * line_height;

                for (i, line) in lines.iter().enumerate() {
                    let y = *scroll_y + i as f32 * line_height;
                    if y > -line_height && y < screen_height() {
                        let text_width = measure_text(line, None, font_size as u16, 1.0).width;
                        let x = (screen_width() - text_width) / 2.0;
                        let dist_from_center =
                            ((y - screen_height() / 2.0) / (screen_height() / 2.0)).abs();
                        let alpha = (1.0 - dist_from_center).max(0.0);
                        let color = Color::new(1.0, 1.0, 0.0, alpha);
                        draw_text(line, x, y, font_size, color);
                    }
                }

                *scroll_y -= INTRO_SCROLL_SPEED * get_frame_time();

                if *scroll_y + total_height < 0.0 || get_last_key_pressed().is_some() {
                    music.play();
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                // Advance stun timer and auto-respawn
                if lvl.player.stunned {
                    lvl.player.stun_timer += get_frame_time();
                    let move_key = is_key_pressed(KeyCode::A)
                        || is_key_pressed(KeyCode::D)
                        || is_key_pressed(KeyCode::W)
                        || is_key_pressed(KeyCode::Left)
                        || is_key_pressed(KeyCode::Right)
                        || is_key_pressed(KeyCode::Up)
                        || is_key_pressed(KeyCode::Space);
                    if lvl.player.stun_timer >= 3.0 || move_key {
                        lvl.player.respawn();
                    }
                }

                tick_acc += get_frame_time() as f64;

                if tick_acc >= TICK_RATE {
                    tick_acc -= TICK_RATE;

                    for p in &mut lvl.platforms {
                        p.update(&mut lvl.grid, &mut lvl.player, &mut lvl.enemies);
                    }
                    for vp in &mut lvl.vplatforms {
                        vp.update(&mut lvl.grid, &mut lvl.player, &mut lvl.enemies);
                    }

                    let events = lvl.player.update(&lvl.grid);
                    if events.jumped {
                        music.play_jump();
                    }
                    if events.stepped {
                        music.play_step();
                    }
                    if events.sprung {
                        music.play_spring();
                    }
                    if events.died {
                        music.play_death();
                    }

                    for e in &mut lvl.enemies {
                        e.update(&lvl.grid);
                    }

                    // Enemy collision
                    let mut enemy_killed = false;
                    for e in &lvl.enemies {
                        if (lvl.player.col - e.col).abs() <= 1 && lvl.player.row == e.row {
                            lvl.player.die();
                            enemy_killed = true;
                        }
                        if lvl.player.col == e.col && (lvl.player.row - e.row).abs() <= 1 {
                            lvl.player.die();
                            enemy_killed = true;
                        }
                    }
                    if enemy_killed {
                        music.play_death();
                    }

                    // Spring key: turn all # into z
                    if !lvl.player.stunned && lvl.player.alive {
                        let pr = lvl.player.row as usize;
                        let pc = lvl.player.col as usize;
                        if lvl.grid[pr][pc] == 'k' {
                            lvl.grid[pr][pc] = ' ';
                            for row in lvl.grid.iter_mut() {
                                for ch in row.iter_mut() {
                                    if *ch == '#' {
                                        *ch = 'z';
                                    }
                                }
                            }
                        }
                    }

                    // Coin collection
                    if !lvl.player.stunned && lvl.player.alive {
                        let pr = lvl.player.row as usize;
                        let pc = lvl.player.col as usize;
                        if lvl.grid[pr][pc] == 'o' {
                            lvl.grid[pr][pc] = ' ';
                            lvl.score += 1;
                            // Unlock padlocks at 10 coins
                            if lvl.score >= 10 {
                                for row in lvl.grid.iter_mut() {
                                    for ch in row.iter_mut() {
                                        if *ch == '%' {
                                            *ch = ' ';
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Goal reached
                    let goal_tile = lvl.grid[lvl.player.row as usize][lvl.player.col as usize];
                    if goal_tile == '*' || goal_tile == 's' {
                        let skip_secret = goal_tile == '*';
                        if skip_secret {
                            skipped_any = true;
                        }
                        if !lvl.advance(skip_secret) {
                            music.stop();
                            if skipped_any {
                                lvl.restart();
                                state = GameState::Cheater;
                            } else {
                                state = GameState::Win(screen_height());
                            }
                        }
                        tick_acc = 0.0;
                    }

                    // Death check — on secret levels, any death advances to next non-secret level
                    if (!lvl.player.alive || lvl.player.stunned) && Level::is_secret(lvl.idx) {
                        lvl.player.stunned = false;
                        skipped_any = true;
                        if !lvl.advance(true) {
                            music.stop();
                            lvl.restart();
                            state = GameState::Splash;
                        }
                        tick_acc = 0.0;
                    } else if !lvl.player.alive {
                        state = GameState::Death;
                    }
                }

                // Secret skip key
                if is_key_pressed(KeyCode::N) {
                    skipped_any = true;
                    if !lvl.advance(true) {
                        lvl.restart();
                        music.stop();
                        state = GameState::Splash;
                    }
                    tick_acc = 0.0;
                }

                // Draw
                clear_background(BLACK);
                draw_grid(&lvl.grid, tile_w, tile_h);
                for e in &lvl.enemies {
                    e.draw(tile_w, tile_h);
                }

                // Fade level to black during stun
                if lvl.player.stunned {
                    let t = (lvl.player.stun_timer / 3.0).min(1.0);
                    draw_rectangle(
                        0.0, 0.0,
                        screen_width(), screen_height(),
                        Color::new(0.0, 0.0, 0.0, t),
                    );
                }

                lvl.player.draw(tile_w, tile_h);

                let status_y = LEVEL_ROWS as f32 * tile_h + STATUS_BAR_HEIGHT * 0.75;
                if lvl.player.stunned {
                    let lives_text = format!("Lives: {}", lvl.player.lives);
                    // Offset to align with "Lives:" in the full status bar
                    let prefix = format!("Level: {}   ", lvl.name());
                    let offset_x = 10.0 + measure_text(&prefix, None, (STATUS_BAR_HEIGHT * 0.8) as u16, 1.0).width;
                    draw_text(&lives_text, offset_x, status_y, STATUS_BAR_HEIGHT * 0.8, GREEN);
                } else {
                    let status = format!(
                        "Level: {}   Lives: {}   Score: {}   A/D=move  SPC=jump  W/S=climb  P=quit",
                        lvl.name(),
                        lvl.player.lives,
                        lvl.score
                    );
                    draw_text(&status, 10.0, status_y, STATUS_BAR_HEIGHT * 0.8, GREEN);
                }
            }

            GameState::Death => {
                clear_background(BLACK);
                draw_grid(&death_grid, tile_w, tile_h);

                let hint_y = LEVEL_ROWS as f32 * tile_h + STATUS_BAR_HEIGHT * 0.75;
                draw_text("Skip levels with 'N'", 10.0, hint_y, STATUS_BAR_HEIGHT * 0.8, GREEN);

                if get_last_key_pressed().is_some() {
                    lvl.restart();
                    skipped_any = false;
                    tick_acc = 0.0;
                    music.play();
                    state = GameState::Playing;
                }
            }

            GameState::Cheater => {
                clear_background(BLACK);
                draw_grid(&cheater_grid, tile_w, tile_h);

                if get_last_key_pressed().is_some() {
                    skipped_any = false;
                    state = GameState::Splash;
                }
            }

            GameState::Win(ref mut scroll_y) => {
                clear_background(BLACK);

                let font_size = 28.0;
                let line_height = font_size * 1.4;
                let lines: Vec<&str> = WIN_TEXT.lines().collect();
                let total_height = lines.len() as f32 * line_height;

                for (i, line) in lines.iter().enumerate() {
                    let y = *scroll_y + i as f32 * line_height;
                    if y > -line_height && y < screen_height() {
                        let text_width = measure_text(line, None, font_size as u16, 1.0).width;
                        let x = (screen_width() - text_width) / 2.0;
                        let dist_from_center =
                            ((y - screen_height() / 2.0) / (screen_height() / 2.0)).abs();
                        let alpha = (1.0 - dist_from_center).max(0.0);
                        let color = Color::new(1.0, 1.0, 0.0, alpha);
                        draw_text(line, x, y, font_size, color);
                    }
                }

                *scroll_y -= INTRO_SCROLL_SPEED * get_frame_time();

                if *scroll_y + total_height < 0.0 || get_last_key_pressed().is_some() {
                    lvl.restart();
                    skipped_any = false;
                    state = GameState::Splash;
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
