use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Stylize},
    terminal, Result,
};
use rand::Rng;
use std::collections::VecDeque;
use std::io::{stdout, Write};
use std::{thread, time::{Duration, Instant}};

const CELL_SZ: (u16, u16) = (2, 1);
const GND_SZ: (u16, u16) = (64, 32);
const TIME_STEP: u64 = 150; // game state refresh timestep in milliseconds

#[derive(Debug, Eq, PartialEq)]
struct Cell {
    pos: (u16, u16), // (horizontal coord, vertical coord)
    size: (u16, u16), // (horizontal length, vertical length)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Color {
    Red,
    Blue,
    White,
}

impl Cell {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            pos: (x, y),
            size: CELL_SZ,
        }
    }
    pub fn clone_with_pos_shift(&self, dir: Direction, steps: u16) -> Self {
        let mut x = self.pos.0;
        let mut y = self.pos.1;
        match dir {
            Direction::Up => y -= steps * self.size.1,
            Direction::Down => y += steps * self.size.1,
            Direction::Left => x -= steps * self.size.0,
            Direction::Right => x += steps * self.size.0,
        }
        Self::new(x, y)
    }
    fn render<T: Write>(&self, output: &mut T, color: Color) -> Result<()> {
        for x in self.pos.0..self.pos.0 + self.size.0 {
            for y in self.pos.1..self.pos.1 + self.size.1 {
                queue!(
                    output,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent(match color {
                        Color::Red => "█".red(),
                        Color::Blue => "█".blue(),
                        Color::White => "█".white(),
                    })
                )?;
            }
        }
        Ok(())
    }
}

struct Snake {
    body: VecDeque<Cell>,
    dir: Direction,
}

impl Snake {
    pub fn new((x, y): (u16, u16), dir: Direction, len: u16) -> Self {
        let head = Cell {
            pos: (x, y),
            size: CELL_SZ,
        };
        let dir_rev = match dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        let body: VecDeque<_> = (0..len)
            .map(|i| head.clone_with_pos_shift(dir_rev, i))
            .collect();
        Self { body, dir }
    }

    pub fn head(&self) -> &Cell {
        self.body.front().unwrap()
    }

    /// grow snake body when eating food
    pub fn grow_body(&mut self) {
        self.body
            .push_front(self.head().clone_with_pos_shift(self.dir, 1));
    }

    pub fn move_body(&mut self) {
        self.body
            .push_front(self.head().clone_with_pos_shift(self.dir, 1));
        self.body.pop_back();
    }

    pub fn check_bite_body(&self) -> bool {
        self.body.iter().skip(1).any(|c| c == self.head())
    }

    pub fn check_bite_food(&self, food: &Cell) -> bool {
        self.head() == food
    }

    /// check if the snake body overlaps with food when generating food
    pub fn check_overlap_food(&self, food: &Cell) -> bool {
        self.body.iter().any(|c| c == food)
    }

    pub fn check_collide_wall(&self, wall: &Wall) -> bool {
        wall.cells.iter().any(|c| c == self.head())
    }

    pub fn render<T: Write>(&self, buffer: &mut T) -> Result<()> {
        for cell in &self.body {
            cell.render(buffer, Color::Blue)?;
        }
        Ok(())
    }
}

struct Wall {
    cells: Vec<Cell>,
}

impl Wall {
    pub fn new() -> Self {
        let top_wall = (1..GND_SZ.0 / CELL_SZ.0).map(|i| (i * CELL_SZ.0, CELL_SZ.1));
        let btm_wall = (1..GND_SZ.0 / CELL_SZ.0).map(|i| (i * CELL_SZ.0, GND_SZ.1));
        let lft_wall = (2..GND_SZ.1 / CELL_SZ.1).map(|i| (CELL_SZ.0, i * CELL_SZ.1));
        let rht_wall = (2..GND_SZ.1 / CELL_SZ.1).map(|i| (GND_SZ.0 - CELL_SZ.0, i * CELL_SZ.1));
        Self {
            cells: top_wall
                .chain(lft_wall)
                .chain(rht_wall)
                .chain(btm_wall)
                .map(|(x, y)| Cell::new(x, y))
                .collect::<Vec<_>>(),
        }
    }

    pub fn render<T: Write>(&self, buffer: &mut T) -> Result<()> {
        for cell in &self.cells {
            cell.render(buffer, Color::White)?;
        }
        Ok(())
    }
}

struct Game {
    wall: Wall,
    snake: Snake,
    food: Cell,
    score: u16,
    time: Instant,
    time_step: Duration,
    is_over: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            wall: Wall::new(),
            snake: Snake::new((GND_SZ.0 / 2, GND_SZ.1 / 2), Direction::Right, 3),
            food: Cell::new(30, 30),
            score: 0,
            time: Instant::now(),
            time_step: Duration::from_millis(TIME_STEP),
            is_over: false,
        }
    }

    pub fn render_food<T: Write>(&self, buffer: &mut T) -> Result<()> {
        self.food.render(buffer, Color::Red)?;
        Ok(())
    }

    pub fn update_food_pos(&mut self) {
        let x = rand::thread_rng().gen_range(1..GND_SZ.0 / CELL_SZ.0 - 1) * CELL_SZ.0;
        let y = rand::thread_rng().gen_range(2..GND_SZ.1 / CELL_SZ.1 - 1) * CELL_SZ.1;
        self.food.pos = (x, y);
    }

    fn render_title<T: Write>(&self, buffer: &mut T) -> Result<()> {
        queue!(
            buffer,
            cursor::MoveTo(10, 0),
            style::PrintStyledContent("Rust Snake Game".magenta())
        )?;
        queue!(
            buffer,
            cursor::MoveTo(40, 0),
            style::PrintStyledContent(format!("Score: {}", self.score).green())
        )?;
        Ok(())
    }

    pub fn render<T: Write>(&self, buffer: &mut T) -> Result<()> {
        execute!(buffer, terminal::Clear(terminal::ClearType::All))?;
        self.render_title(buffer)?;
        self.snake.render(buffer)?;
        self.render_food(buffer)?;
        self.wall.render(buffer)?;
        buffer.flush()?;
        Ok(())
    }

    fn process_event(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(KeyEvent {code: KeyCode::Up, ..}) => {
                    if self.snake.dir != Direction::Down {
                        self.snake.dir = Direction::Up;
                    }
                }
                Event::Key(KeyEvent {code: KeyCode::Down, ..}) => {
                    if self.snake.dir != Direction::Up {
                        self.snake.dir = Direction::Down;
                    }
                }
                Event::Key(KeyEvent {code: KeyCode::Left, ..}) => {
                    if self.snake.dir != Direction::Right {
                        self.snake.dir = Direction::Left;
                    }
                }
                Event::Key(KeyEvent {code: KeyCode::Right, ..}) => {
                    if self.snake.dir != Direction::Left {
                        self.snake.dir = Direction::Right;
                    }
                }
                Event::Key(KeyEvent {code: KeyCode::Char('q'), ..}) => self.is_over = true,
                _ => (),
            };
            // flush bufferred events before next loop
            while event::poll(Duration::from_millis(0))? {
                event::read()?;
            }
        }
        Ok(())
    }

    fn update_game_state(&mut self) {
        if self.snake.check_bite_body() || self.snake.check_collide_wall(&self.wall) {
            self.is_over = true;
        }
        if self.snake.check_bite_food(&self.food) {
            self.score += 1;
            self.snake.grow_body();
            // generate new food: update food position
            loop {
                self.update_food_pos();
                if !self.snake.check_overlap_food(&self.food) {
                    break;
                }
            }
        } else {
            self.snake.move_body();
        }
    }

    pub fn looping<T: Write>(&mut self, buffer: &mut T) -> Result<()> {
        while !self.is_over {
            self.render(buffer)?;
            self.process_event()?;
            if self.time.elapsed() > self.time_step {
                self.update_game_state();
                self.time = Instant::now();
            }
            thread::sleep(self.time_step / 2); // screen refreshing rate
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut buffer = stdout();
    let mut game = Game::new();
    game.looping(&mut buffer)?;
    Ok(())
}
