use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal, Result,
};
use std::io::{stdout, Write};
use std::{collections::VecDeque, io::Stdout};
use tui::Terminal;

const CELL_SZ: (u16, u16) = (2, 1);
const GROUND_SZ: (u16, u16) = (64, 32);

#[derive(Debug, Eq, PartialEq)]
struct Cell {
    pos: (u16, u16), // (horz, vert)
    size: (u16, u16),
}

#[derive(Debug, Copy, Clone)]
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
    fn render<T: Write>(&self, output: &mut T, color: Color) {
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
                );
            }
        }
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
        let body: VecDeque<_> = (0..len)
            .map(|i| head.clone_with_pos_shift(dir, i))
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

    pub fn check_cllide_wall(&self, wall: &Wall) -> bool {
        wall.cells.iter().any(|c| c == self.head())
    }
}

struct Wall {
    cells: Vec<Cell>,
}

impl Wall {
    pub fn new() -> Self {
        let top_wall = (0..GROUND_SZ.0 / CELL_SZ.0 - 1).map(|i| (i * CELL_SZ.0, CELL_SZ.1));
        let btm_wall =
            (0..GROUND_SZ.0 / CELL_SZ.0 - 1).map(|i| (i * CELL_SZ.0, GROUND_SZ.1 - CELL_SZ.1));
        let lft_wall = (1..GROUND_SZ.1 / CELL_SZ.1 - 2).map(|i| (0, i * CELL_SZ.1));
        let rht_wall =
            (1..GROUND_SZ.1 / CELL_SZ.1 - 2).map(|i| (GROUND_SZ.0 - CELL_SZ.0, i * CELL_SZ.1));
        let res = Self {
            cells: top_wall
                .chain(btm_wall)
                .chain(lft_wall)
                .chain(rht_wall)
                .map(|(x, y)| Cell::new(x, y))
                .collect::<Vec<_>>(),
        };
        // println!("cells numver: {}", res.cells.len());
        res
        // let cells: Vec<_> = (0..GROUND_SZ.0 / CELL_SZ.0 - 1)
        //     .map(|i| Cell::new(i * CELL_SZ.0, CELL_SZ.1))
        //     .chain((0..GROUND_SZ.0 / CELL_SZ.0 - 1).map(|i| Cell::new(i * CELL_SZ.0, GROUND_SZ.1)))
        //     .chain((2..GROUND_SZ.1 / CELL_SZ.1 - 1).map(|i| Cell::new(0, i * CELL_SZ.1)))
        //     .chain((2..GROUND_SZ.1 / CELL_SZ.1 - 1).map(|i| Cell::new(GROUND_SZ.0, i * CELL_SZ.1)))
        //     .collect();
        // Self { cells }
    }

    pub fn render<T: Write>(&self, buffer: &mut T) {
        for cell in &self.cells {
            cell.render(buffer, Color::White);
        }
    }
}

struct Game {
    wall: Wall,
    snake: Snake,
    food: Cell,
    score: u16,
}

impl Game {
    pub fn new() -> Self {
        Self {
            wall: Wall::new(),
            snake: Snake::new((GROUND_SZ.0 / 2, GROUND_SZ.1 / 2), Direction::Right, 3),
            food: Cell::new(30, 30),
            score: 0,
        }
    }
}

fn update_screen<T: Write>(wall: Wall, buffer: &mut T) -> Result<()> {
    execute!(buffer, terminal::Clear(terminal::ClearType::All))?;
    wall.render(buffer);
    buffer.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    // let mut buffer = stdout();
    // let wall = Wall::new();
    // update_screen(wall, &mut buffer);

    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    for y in 0..32 {
        for x in 0..64 {
            if (y == 0 || y == 32 - 1) || (x == 0 || x == 64 - 1) {
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent("█".magenta())
                )?;
            }
        }
    }
    stdout.flush()?;
    Ok(())
}
