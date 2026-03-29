use macroquad::prelude::*;

pub struct PlayerEvents {
    pub jumped: bool,
    pub stepped: bool,
    pub died: bool,
    pub sprung: bool,
}

pub struct Player {
    pub col: i32,
    pub row: i32,
    pub jump_remaining: i32,
    pub alive: bool,
    pub stunned: bool,
    pub stun_timer: f32,
    pub on_ladder: bool,
    pub spawn_col: i32,
    pub spawn_row: i32,
    pub lives: i32,
}

const STUN_DURATION: f32 = 3.0;

impl Player {
    pub fn spawn(grid: &mut Vec<Vec<char>>) -> Self {
        for (r, line) in grid.iter_mut().enumerate() {
            for (c, ch) in line.iter_mut().enumerate() {
                if *ch == '$' {
                    *ch = ' ';
                    return Player {
                        col: c as i32,
                        row: r as i32,
                        jump_remaining: 0,
                        alive: true,
                        stunned: false,
                        stun_timer: 0.0,
                        on_ladder: false,
                        spawn_col: c as i32,
                        spawn_row: r as i32,
                        lives: 10,
                    };
                }
            }
        }
        Player { col: 1, row: 1, jump_remaining: 0, alive: true, stunned: false, stun_timer: 0.0, on_ladder: false, spawn_col: 1, spawn_row: 1, lives: 10 }
    }

    fn tile_at(&self, grid: &[Vec<char>], col: i32, row: i32) -> char {
        if row < 0 || row >= grid.len() as i32 || col < 0 || col >= grid[0].len() as i32 {
            return ' ';
        }
        grid[row as usize][col as usize]
    }

    pub fn update(&mut self, grid: &[Vec<char>]) -> PlayerEvents {
        let mut events = PlayerEvents { jumped: false, stepped: false, died: false, sprung: false };

        if !self.alive || self.stunned {
            return events;
        }

        let old_col = self.col;

        // Input
        let mut dx = 0i32;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dx = -1;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dx = 1;
        }
        let want_up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) || is_key_down(KeyCode::Space);
        let want_down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);

        let current_tile = self.tile_at(grid, self.col, self.row);
        let below_tile = self.tile_at(grid, self.col, self.row + 1);

        // Enter ladder: on ground, H is below, press down
        if !self.on_ladder && self.on_ground(grid) && below_tile == 'H' && want_down {
            self.on_ladder = true;
            self.row += 1; // step into the ladder
        } else if self.on_ladder {
            // Ladder movement: up/down through H tiles, left/right to exit
            if want_up {
                let above = self.tile_at(grid, self.col, self.row - 1);
                if above == 'H' {
                    // Climb up within ladder
                    self.row -= 1;
                } else if self.is_passable(grid, self.col, self.row - 1) {
                    // Climb off the top — land above the ladder
                    self.row -= 1;
                    self.on_ladder = false;
                }
            }
            if want_down {
                if self.tile_at(grid, self.col, self.row + 1) == 'H' {
                    self.row += 1;
                }
            }

            // Horizontal movement — exits ladder if destination is not H
            if dx != 0 {
                let new_col = self.col + dx;
                if self.is_passable(grid, new_col, self.row) {
                    self.col = new_col;
                    self.on_ladder = false;
                }
            }
        } else {
            // Normal movement

            // Jump
            if is_key_down(KeyCode::Space) && self.on_ground(grid) {
                self.jump_remaining = 2;
                events.jumped = true;
            }

            // Horizontal movement — slides override input
            let on_slide = matches!(grid[self.row as usize][self.col as usize], '/' | '\\');
            if !on_slide && dx != 0 {
                let new_col = self.col + dx;
                if self.tile_at(grid, new_col, self.row) == 'H' {
                    self.col = new_col;
                    self.on_ladder = true;
                } else if self.is_passable(grid, new_col, self.row) {
                    self.col = new_col;
                }
            }

            // Vertical: jump or gravity
            if self.jump_remaining > 0 {
                let new_row = self.row - 1;
                if self.is_passable(grid, self.col, new_row) {
                    self.row = new_row;
                    self.jump_remaining -= 1;
                } else {
                    self.jump_remaining = 0;
                }
            } else {
                // Gravity: fall 1 row
                let new_row = self.row + 1;
                if self.is_passable(grid, self.col, new_row) {
                    self.row = new_row;
                }
            }

            // Springs: if overlapping 'z', lift up first
            if grid[self.row as usize][self.col as usize] == 'z' {
                self.row -= 1;
            }

            // Springs: if standing on 'z', launch up 4 rows
            let below = self.row + 1;
            if below >= 0 && below < grid.len() as i32 {
                if grid[below as usize][self.col as usize] == 'z' {
                    self.jump_remaining = 4;
                    events.sprung = true;
                }
            }

            // Slides: push player in designated direction
            let current = grid[self.row as usize][self.col as usize];
            let slide_dx = match current {
                '/' => -1,
                '\\' => 1,
                _ => 0,
            };
            if slide_dx != 0 {
                let new_col = self.col + slide_dx;
                if self.is_passable(grid, new_col, self.row) {
                    self.col = new_col;
                }
            }
        }

        // Spikes: die if touching
        let current = grid[self.row as usize][self.col as usize];
        if matches!(current, '^' | 'v' | '>' | '<') {
            self.die();
            events.died = true;
        }

        if self.col != old_col && self.on_ground(grid) {
            events.stepped = true;
        }

        events
    }

    pub fn die(&mut self) {
        if self.stunned {
            return;
        }
        self.lives -= 1;
        if self.lives <= 0 {
            self.alive = false;
        } else {
            self.stunned = true;
            self.stun_timer = 0.0;
            self.jump_remaining = 0;
        }
    }

    pub fn respawn(&mut self) {
        self.stunned = false;
        self.on_ladder = false;
        self.col = self.spawn_col;
        self.row = self.spawn_row;
        self.jump_remaining = 0;
    }

    fn on_ground(&self, grid: &[Vec<char>]) -> bool {
        !self.is_passable(grid, self.col, self.row + 1)
    }

    fn is_passable(&self, grid: &[Vec<char>], col: i32, row: i32) -> bool {
        if row < 0 || row >= grid.len() as i32 || col < 0 || col >= grid[0].len() as i32 {
            return false;
        }
        let ch = grid[row as usize][col as usize];
        ch != '#' && ch != '=' && ch != 'H'
    }

    pub fn draw(&self, tile_w: f32, tile_h: f32) {
        if !self.alive {
            return;
        }
        let x = self.col as f32 * tile_w;
        let y = self.row as f32 * tile_h;
        let color = if self.stunned {
            let t = (self.stun_timer / STUN_DURATION).min(1.0);
            Color::new(t, 1.0 - t, 0.0, 1.0)
        } else {
            Color::new(0.0, 1.0, 0.0, 1.0)
        };
        draw_text("$", x, y + tile_h * 0.85, tile_h * 1.2, color);
    }
}
