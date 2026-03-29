use macroquad::prelude::*;

pub struct Enemy {
    pub col: i32,
    pub row: i32,
    pub dir: i32, // -1 left, 1 right
}

impl Enemy {
    pub fn spawn_all(grid: &mut Vec<Vec<char>>) -> Vec<Self> {
        let mut enemies = Vec::new();
        for (r, line) in grid.iter_mut().enumerate() {
            for (c, ch) in line.iter_mut().enumerate() {
                if *ch == '@' {
                    *ch = ' ';
                    enemies.push(Enemy {
                        col: c as i32,
                        row: r as i32,
                        dir: 1,
                    });
                }
            }
        }
        enemies
    }

    pub fn update(&mut self, grid: &[Vec<char>]) {
        // Try to move in current direction
        let new_col = self.col + self.dir;

        // Reverse if hitting a wall or reaching platform edge
        if !self.is_passable(grid, new_col, self.row)
            || !self.has_ground(grid, new_col, self.row)
        {
            self.dir = -self.dir;
        } else {
            self.col = new_col;
        }

        // Gravity: fall if no ground
        let below = self.row + 1;
        if self.is_passable(grid, self.col, below) {
            self.row = below;
        }
    }

    fn is_passable(&self, grid: &[Vec<char>], col: i32, row: i32) -> bool {
        if row < 0 || row >= grid.len() as i32 || col < 0 || col >= grid[0].len() as i32 {
            return false;
        }
        let ch = grid[row as usize][col as usize];
        ch != '#' && ch != '=' && ch != 'H'
    }

    fn has_ground(&self, grid: &[Vec<char>], col: i32, row: i32) -> bool {
        let below = row + 1;
        if below >= grid.len() as i32 {
            return true;
        }
        let ch = grid[below as usize][col as usize];
        ch == '#' || ch == '=' || ch == 'H'
    }

    pub fn draw(&self, tile_w: f32, tile_h: f32) {
        let x = self.col as f32 * tile_w;
        let y = self.row as f32 * tile_h;
        draw_text("@", x, y + tile_h * 0.85, tile_h * 1.2, Color::new(1.0, 0.2, 0.2, 1.0));
    }
}
