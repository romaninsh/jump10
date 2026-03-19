use macroquad::prelude::*;

pub struct Player {
    pub col: i32,
    pub row: i32,
    pub jump_remaining: i32,
    pub alive: bool,
    pub spawn_col: i32,
    pub spawn_row: i32,
    pub lives: i32,
}

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
                        spawn_col: c as i32,
                        spawn_row: r as i32,
                        lives: 10,
                    };
                }
            }
        }
        Player { col: 1, row: 1, jump_remaining: 0, alive: true, spawn_col: 1, spawn_row: 1, lives: 10 }
    }

    pub fn update(&mut self, grid: &[Vec<char>]) {
        if !self.alive {
            return;
        }

        // Input
        let mut dx = 0i32;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dx = -1;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dx = 1;
        }

        if (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) || is_key_down(KeyCode::Space))
            && self.on_ground(grid)
        {
            self.jump_remaining = 2;
        }

        // Horizontal movement — slides override input
        let on_slide = matches!(grid[self.row as usize][self.col as usize], '/' | '\\');
        if !on_slide {
            let new_col = self.col + dx;
            if self.is_passable(grid, new_col, self.row) {
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

        // Springs: if standing on 'z', launch up 4 rows
        let below = self.row + 1;
        if below >= 0 && below < grid.len() as i32 {
            if grid[below as usize][self.col as usize] == 'z' {
                self.jump_remaining = 4;
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

        // Spikes: die if touching '^'
        let current = grid[self.row as usize][self.col as usize];
        if current == '^' {
            self.die();
        }
    }

    pub fn die(&mut self) {
        self.lives -= 1;
        if self.lives <= 0 {
            self.alive = false;
        } else {
            self.col = self.spawn_col;
            self.row = self.spawn_row;
            self.jump_remaining = 0;
        }
    }

    fn on_ground(&self, grid: &[Vec<char>]) -> bool {
        !self.is_passable(grid, self.col, self.row + 1)
    }

    fn is_passable(&self, grid: &[Vec<char>], col: i32, row: i32) -> bool {
        if row < 0 || row >= grid.len() as i32 || col < 0 || col >= grid[0].len() as i32 {
            return false;
        }
        let ch = grid[row as usize][col as usize];
        ch != '#' && ch != '='
    }

    pub fn draw(&self, tile_w: f32, tile_h: f32) {
        if !self.alive {
            return;
        }
        let x = self.col as f32 * tile_w;
        let y = self.row as f32 * tile_h;
        draw_text("$", x, y + tile_h * 0.85, tile_h * 1.2, Color::new(0.0, 1.0, 0.0, 1.0));
    }
}
