use crate::enemy::Enemy;
use crate::player::Player;

pub struct MovingPlatform {
    pub row: usize,
    pub left_bound: usize,
    pub right_bound: usize,
    pub start_col: usize,
    pub width: usize,
    pub dir: i32,
    pub half_tick: bool,
}

impl MovingPlatform {
    pub fn spawn_all(grid: &mut Vec<Vec<char>>) -> Vec<Self> {
        let mut platforms = Vec::new();
        for r in 0..grid.len() {
            let mut c = 0;
            while c < grid[r].len() {
                if grid[r][c] == '+' {
                    // Find matching '+'
                    if let Some(end) = grid[r][c + 1..].iter().position(|&ch| ch == '+') {
                        let end = c + 1 + end;
                        let segment: Vec<char> = grid[r][c + 1..end].to_vec();

                        // Check that segment contains only '=' and '-'
                        if !segment.is_empty()
                            && segment.iter().all(|&ch| ch == '=' || ch == '-')
                            && segment.iter().any(|&ch| ch == '=')
                        {
                            // Find the '=' block position
                            let eq_offset = segment.iter().position(|&ch| ch == '=').unwrap();
                            let eq_count = segment[eq_offset..]
                                .iter()
                                .take_while(|&&ch| ch == '=')
                                .count();

                            platforms.push(MovingPlatform {
                                row: r,
                                left_bound: c,
                                right_bound: end,
                                start_col: c + 1 + eq_offset,
                                width: eq_count,
                                dir: 1,
                                half_tick: false,
                            });

                            // Clear '+' and '-' from grid, keep '='
                            grid[r][c] = ' ';
                            grid[r][end] = ' ';
                            for i in c + 1..end {
                                if grid[r][i] == '-' {
                                    grid[r][i] = ' ';
                                }
                            }
                        }
                        c = end + 1;
                    } else {
                        c += 1;
                    }
                } else {
                    c += 1;
                }
            }
        }
        platforms
    }

    pub fn update(
        &mut self,
        grid: &mut Vec<Vec<char>>,
        player: &mut Player,
        enemies: &mut Vec<Enemy>,
    ) {
        self.half_tick = !self.half_tick;
        if !self.half_tick {
            return;
        }

        let cur_left = self.start_col;
        let cur_right = self.start_col + self.width - 1;

        // Check bounds and reverse if needed
        if self.dir > 0 && cur_right + 1 >= self.right_bound {
            self.dir = -1;
        } else if self.dir < 0 && cur_left <= self.left_bound + 1 {
            self.dir = 1;
        }

        // Shift '=' tiles in grid
        if self.dir > 0 {
            // Moving right: iterate right-to-left
            for col in (cur_left..=cur_right).rev() {
                grid[self.row][col + 1] = '=';
                grid[self.row][col] = ' ';
            }
        } else {
            // Moving left: iterate left-to-right
            for col in cur_left..=cur_right {
                grid[self.row][col - 1] = '=';
                grid[self.row][col] = ' ';
            }
        }

        self.start_col = (self.start_col as i32 + self.dir) as usize;

        // Carry passengers (entities on row above the platform)
        let passenger_row = self.row as i32 - 1;
        if passenger_row >= 0 {
            let old_left = cur_left as i32;
            let old_right = cur_right as i32;

            // Player
            if player.row == passenger_row
                && player.col >= old_left
                && player.col <= old_right
            {
                let new_col = player.col + self.dir;
                if new_col >= 0 && (new_col as usize) < grid[0].len() {
                    player.col = new_col;
                }
            }

            // Enemies
            for e in enemies.iter_mut() {
                if e.row == passenger_row as i32
                    && e.col >= old_left
                    && e.col <= old_right
                {
                    let new_col = e.col + self.dir;
                    if new_col >= 0 && (new_col as usize) < grid[0].len() {
                        e.col = new_col;
                    }
                }
            }
        }
    }
}
