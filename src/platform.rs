use crate::enemy::Enemy;
use crate::player::Player;

pub struct VerticalPlatform {
    pub row: usize,         // current row of the '=' tiles
    pub start_col: usize,   // leftmost '=' column
    pub width: usize,       // number of '=' tiles
    pub top_bound: usize,   // row of top '+' (exclusive)
    pub bottom_bound: usize,// row of bottom '+' (exclusive)
    pub dir: i32,            // -1 up, 1 down
    pub half_tick: bool,
}

impl VerticalPlatform {
    /// Scan for vertical platform patterns: a column with + | ... = ... | +
    pub fn spawn_all(grid: &mut Vec<Vec<char>>) -> Vec<Self> {
        let mut platforms = Vec::new();
        let rows = grid.len();
        let cols = if rows > 0 { grid[0].len() } else { 0 };

        // Track which '=' segments we've already claimed
        let mut claimed: Vec<Vec<bool>> = vec![vec![false; cols]; rows];

        for c in 0..cols {
            for r in 0..rows {
                if grid[r][c] != '+' {
                    continue;
                }

                // Look down from this '+' for '|' track, then '=' row, then '|' track, then '+'
                let mut scan = r + 1;

                // Skip '|' going down
                while scan < rows && grid[scan][c] == '|' {
                    scan += 1;
                }
                if scan == r + 1 || scan >= rows {
                    continue; // no '|' found
                }

                let eq_row = scan;
                if grid[eq_row][c] != '=' {
                    continue;
                }

                // Find the full '=' segment on this row
                let mut eq_left = c;
                while eq_left > 0 && grid[eq_row][eq_left - 1] == '=' {
                    eq_left -= 1;
                }
                let mut eq_right = c;
                while eq_right + 1 < cols && grid[eq_row][eq_right + 1] == '=' {
                    eq_right += 1;
                }

                // Already claimed?
                if claimed[eq_row][eq_left] {
                    continue;
                }

                // Continue down past '=' row: expect '|' then '+'
                scan = eq_row + 1;
                while scan < rows && grid[scan][c] == '|' {
                    scan += 1;
                }
                if scan == eq_row + 1 || scan >= rows || grid[scan][c] != '+' {
                    continue;
                }

                let bottom_plus = scan;
                let width = eq_right - eq_left + 1;

                // Mark claimed
                for col in eq_left..=eq_right {
                    claimed[eq_row][col] = true;
                }

                platforms.push(VerticalPlatform {
                    row: eq_row,
                    start_col: eq_left,
                    width,
                    top_bound: r,
                    bottom_bound: bottom_plus,
                    dir: 1,
                    half_tick: false,
                });

                // Clear '+' and '|' from grid, keep '='
                grid[r][c] = ' ';
                grid[bottom_plus][c] = ' ';
                for row in r + 1..eq_row {
                    if grid[row][c] == '|' {
                        grid[row][c] = ' ';
                    }
                }
                for row in eq_row + 1..bottom_plus {
                    if grid[row][c] == '|' {
                        grid[row][c] = ' ';
                    }
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

        let old_row = self.row;

        // Check bounds and reverse if needed
        if self.dir > 0 && self.row + 1 >= self.bottom_bound {
            self.dir = -1;
        } else if self.dir < 0 && self.row <= self.top_bound + 1 {
            self.dir = 1;
        }

        let new_row = (self.row as i32 + self.dir) as usize;

        // Clear old row
        for col in self.start_col..self.start_col + self.width {
            grid[old_row][col] = ' ';
        }
        // Write new row
        for col in self.start_col..self.start_col + self.width {
            grid[new_row][col] = '=';
        }

        self.row = new_row;

        // Carry passengers (entities on row above the platform)
        let old_passenger_row = old_row as i32 - 1;
        let left = self.start_col as i32;
        let right = (self.start_col + self.width - 1) as i32;

        if player.row == old_passenger_row
            && player.col >= left
            && player.col <= right
        {
            player.row += self.dir;
        }

        for e in enemies.iter_mut() {
            if e.row == old_passenger_row
                && e.col >= left
                && e.col <= right
            {
                e.row += self.dir;
            }
        }
    }
}

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
