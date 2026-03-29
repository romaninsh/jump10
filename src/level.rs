use crate::enemy::Enemy;
use crate::platform::{MovingPlatform, VerticalPlatform};
use crate::player::Player;
use crate::scroller::Scroller;

pub const LEVEL_COLS: usize = 60;
pub const LEVEL_ROWS: usize = 13;

macro_rules! levels {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../levels/", $name, ".txt"))),)*]
    };
}

// To add a new level, just add its basename (without .txt) to this list:
const LEVELS: &[(&str, &str)] = levels![
    "01", "02", "02s", "03", "04", "04s", "04ss", "05", "06", "07", "08", "09"
];

pub const SPLASH: &str = include_str!("../splash/splash.txt");
pub const DEATH: &str = include_str!("../splash/death.txt");
pub struct Level {
    pub grid: Vec<Vec<char>>,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub platforms: Vec<MovingPlatform>,
    pub vplatforms: Vec<VerticalPlatform>,
    pub scroller: Option<Scroller>,
    pub idx: usize,
    pub score: u32,
}

impl Level {
    pub fn is_secret(idx: usize) -> bool {
        LEVELS[idx].1.starts_with('S')
    }

    pub fn name(&self) -> &'static str {
        LEVELS[self.idx].0
    }

    fn load_level(&mut self, idx: usize, lives: i32) {
        let level_text = LEVELS[idx].1;
        let mut grid = parse_grid(level_text);
        // Clear the secret marker so it doesn't render
        if grid[0][0] == 'S' {
            grid[0][0] = ' ';
        }
        let scroller = crate::scroller::spawn(&mut grid, level_text);
        self.player = Player::spawn(&mut grid);
        self.player.lives = lives;
        self.enemies = Enemy::spawn_all(&mut grid);
        self.platforms = MovingPlatform::spawn_all(&mut grid);
        self.vplatforms = VerticalPlatform::spawn_all(&mut grid);
        self.grid = grid;
        self.scroller = scroller;
        self.idx = idx;
        self.score = 0;
    }

    pub fn load(idx: usize) -> Self {
        let mut lvl = Level {
            grid: Vec::new(),
            player: Player::spawn(&mut vec![vec![' '; LEVEL_COLS]; LEVEL_ROWS]),
            enemies: Vec::new(),
            platforms: Vec::new(),
            vplatforms: Vec::new(),
            scroller: None,
            idx: 0,
            score: 0,
        };
        lvl.load_level(idx, 10);
        lvl
    }

    /// Advance to the next level. If `skip_secret` is true, secret levels are skipped.
    pub fn advance(&mut self, skip_secret: bool) -> bool {
        let lives = self.player.lives;
        let mut next = self.idx + 1;
        if skip_secret {
            while next < LEVELS.len() && Self::is_secret(next) {
                next += 1;
            }
        }
        if next < LEVELS.len() {
            self.load_level(next, lives);
            true
        } else {
            false
        }
    }

    pub fn restart(&mut self) {
        self.load_level(0, 10);
    }
}

pub fn parse_grid(text: &str) -> Vec<Vec<char>> {
    let mut grid = Vec::new();
    for line in text.lines().take(LEVEL_ROWS) {
        let mut row: Vec<char> = line.chars().collect();
        row.resize(LEVEL_COLS, ' ');
        grid.push(row);
    }
    grid.resize_with(LEVEL_ROWS, || vec![' '; LEVEL_COLS]);
    grid
}
