use crate::enemy::Enemy;
use crate::player::Player;

pub const LEVEL_COLS: usize = 60;
pub const LEVEL_ROWS: usize = 13;

const LEVELS: &[&str] = &[
    include_str!("../levels/01.txt"),
    include_str!("../levels/02.txt"),
];

pub const SPLASH: &str = include_str!("../splash/splash.txt");
pub const DEATH: &str = include_str!("../splash/death.txt");

pub struct Level {
    pub grid: Vec<Vec<char>>,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub idx: usize,
}

impl Level {
    pub fn load(idx: usize) -> Self {
        let mut grid = parse_grid(LEVELS[idx]);
        let player = Player::spawn(&mut grid);
        let enemies = Enemy::spawn_all(&mut grid);
        Level { grid, player, enemies, idx }
    }

    pub fn advance(&mut self) -> bool {
        let lives = self.player.lives;
        self.idx += 1;
        if self.idx < LEVELS.len() {
            let mut grid = parse_grid(LEVELS[self.idx]);
            self.player = Player::spawn(&mut grid);
            self.player.lives = lives;
            self.enemies = Enemy::spawn_all(&mut grid);
            self.grid = grid;
            true
        } else {
            false
        }
    }

    pub fn restart(&mut self) {
        self.idx = 0;
        let mut grid = parse_grid(LEVELS[self.idx]);
        self.player = Player::spawn(&mut grid);
        self.enemies = Enemy::spawn_all(&mut grid);
        self.grid = grid;
    }
}

pub fn parse_grid(text: &str) -> Vec<Vec<char>> {
    let mut grid = Vec::new();
    for line in text.lines() {
        let mut row: Vec<char> = line.chars().collect();
        row.resize(LEVEL_COLS, ' ');
        grid.push(row);
    }
    grid.resize_with(LEVEL_ROWS, || vec![' '; LEVEL_COLS]);
    grid
}
