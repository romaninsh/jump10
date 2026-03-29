mod enemy;
mod level;
mod platform;
mod player;

use level::{DEATH, LEVEL_COLS, LEVEL_ROWS, Level, SPLASH, parse_grid};
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use std::sync::mpsc;
use std::io::Cursor;

const TICK_RATE: f64 = 1.0 / 5.0;
const STATUS_BAR_HEIGHT: f32 = 30.0;

enum GameState {
    Splash,
    Intro(f32), // scroll_y offset
    Playing,
    Death,
}

const INTRO_TEXT: &str = "\
Once upon a time, you were playing
a classic retro game with you as
a dollar sign.

You were having fun but decided to
go grab a sandwich as you are
quite hungry.

You leave the game running.

You slowly make your way to the
fridge, avoiding small talk with
your parents.

But as you open the fridge and
grab the sandwich, everything
turns black.

When you open your eyes you
realise you have been turned
into a dollar sign.

Your only way out is to
finish the game.";

const INTRO_SCROLL_SPEED: f32 = 30.0;
const LEVEL_MUSIC: &[u8] = include_bytes!("../music.ogg");

/// Decode OGG to WAV bytes on a background thread.
fn decode_ogg_to_wav(ogg_data: &'static [u8]) -> mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        eprintln!("[audio] decoding started");
        let cursor = Cursor::new(ogg_data);
        let mut reader = lewton::inside_ogg::OggStreamReader::new(cursor).unwrap();

        let channels = reader.ident_hdr.audio_channels as u16;
        let sample_rate = reader.ident_hdr.audio_sample_rate;

        let mut samples: Vec<i16> = Vec::new();
        while let Ok(Some(packets)) = reader.read_dec_packet_itl() {
            samples.extend_from_slice(&packets);
        }

        // Write WAV to memory buffer using hound
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut wav_buf = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut wav_buf, spec).unwrap();
        for &s in &samples {
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
        let wav = wav_buf.into_inner();

        eprintln!("[audio] decoding finished ({} bytes WAV)", wav.len());
        let _ = tx.send(wav);
    });
    rx
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

    let mut level_music: Option<Sound> = None;
    let mut music_decoder: Option<mpsc::Receiver<Vec<u8>>> = None;

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
                    // Start background decode if not already done
                    if level_music.is_none() && music_decoder.is_none() {
                        music_decoder = Some(decode_ogg_to_wav(LEVEL_MUSIC));
                    }
                    state = GameState::Intro(screen_height());
                }
            }

            GameState::Intro(ref mut scroll_y) => {
                // Poll for decoded WAV from background thread
                if level_music.is_none() {
                    if let Some(ref rx) = music_decoder {
                        if let Ok(wav_bytes) = rx.try_recv() {
                            eprintln!("[audio] loading WAV into mixer...");
                            if let Ok(sound) = load_sound_from_bytes(&wav_bytes).await {
                                level_music = Some(sound);
                            }
                            eprintln!("[audio] ready");
                            music_decoder = None;
                        }
                    }
                }

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
                        // Fade near edges
                        let dist_from_center =
                            ((y - screen_height() / 2.0) / (screen_height() / 2.0)).abs();
                        let alpha = (1.0 - dist_from_center).max(0.0);
                        let color = Color::new(1.0, 1.0, 0.0, alpha);
                        draw_text(line, x, y, font_size, color);
                    }
                }

                *scroll_y -= INTRO_SCROLL_SPEED * get_frame_time();

                // End when all text has scrolled off, or key pressed
                if *scroll_y + total_height < 0.0 || get_last_key_pressed().is_some() {
                    if let Some(ref snd) = level_music {
                        play_sound(
                            snd,
                            PlaySoundParams {
                                looped: true,
                                volume: 0.5,
                            },
                        );
                    }
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                tick_acc += get_frame_time() as f64;

                if tick_acc >= TICK_RATE {
                    tick_acc -= TICK_RATE;

                    for p in &mut lvl.platforms {
                        p.update(&mut lvl.grid, &mut lvl.player, &mut lvl.enemies);
                    }

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
                            if let Some(ref snd) = level_music {
                                stop_sound(snd);
                            }
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
                        if let Some(ref snd) = level_music {
                            stop_sound(snd);
                        }
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
                    // Restart music from beginning
                    if let Some(ref snd) = level_music {
                        stop_sound(snd);
                        play_sound(
                            snd,
                            PlaySoundParams {
                                looped: true,
                                volume: 0.5,
                            },
                        );
                    }
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
