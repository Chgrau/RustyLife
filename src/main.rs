extern crate termion;
use std::io::{stdout, Read, Write};
use std::thread::sleep;
use std::time::Duration;
use termion::raw::IntoRawMode;
use termion::{async_stdin, clear, color, cursor, terminal_size,style};

// To simplify types and arguments, we use a position structure.
#[derive(Debug, Clone, Copy)]
struct Pos {
    x: usize,
    y: usize,
}

fn pos(x: usize, y: usize) -> Pos {
    Pos { x, y }
}

// We define the directions around a given cell:
#[derive(Debug, PartialEq)]
enum Direction {
    NW,
    N,
    NE,
    W,
    E,
    SW,
    S,
    SE,
}

#[derive(Debug, Clone)]
struct World {
    // The width of the world
    width: usize,
    // The height of the world
    height: usize,
    // The cells in the world
    cells: Vec<Vec<bool>>,
}

fn build_empty_world(width: usize, height: usize) -> World {
    World {
        width,
        height,
        cells: vec![vec![false; width]; height],
    }
}

struct Game<R, W> {
    // The world of the game
    world: World,
    // The current generation of the world
    generation: usize,
    // The delay between generation
    delay: usize,
    // Position of cursor in world
    cursor: Pos,
    // stdin
    stdin: R,
    // stdout
    stdout: W,
    // Terminal width
    term_width : u16,
    // Terminal height
    term_height : u16,
}

impl<R: Read, W: Write> Game<R, W> {
    // Advance the world one generation.
    fn advance_world(&mut self) {
        let mut old_world = self.world.clone();
        for x_step in 0..self.world.height {
            for y_step in 0..self.world.width {
                self.world.cells[x_step][y_step] =
                    next_cell_state(&mut old_world, pos(x_step, y_step))
            }
        }
        self.generation = self.generation + 1;
    }

    // Printing the cells of the game world
    fn draw_world(&mut self, alive : char, dead : char) {
        write!(self.stdout,"{}",cursor::Goto(2,2)).unwrap();
        let mut row_number = 2;

        for rows in &self.world.cells {
            for cell in rows {
                if *cell {
                    write!(self.stdout, "{}{}", color::Fg(color::Black), alive).unwrap(); {
                    }
                } else {
                    write!(self.stdout, "{}{}", color::Fg(color::Black), dead).unwrap();
                }
            }
            row_number = row_number + 1;
            write!(self.stdout,"{}",cursor::Goto(2,row_number)).unwrap();
        }
    }

    fn splash(&mut self) {
        let term_x_mid = (self.term_height - 2) / 2;
        write!(
            self.stdout,
            "{}{}{}{}{}{}{}",
            clear::All,
            cursor::Goto((self.term_width - 22) / 2, term_x_mid),
            style::Bold,
            "Welcome to Rusty Life!",
            cursor::Goto((self.term_width - 24) / 2, term_x_mid + 1),
            "Press spacebar to begin",
            style::Reset,
        ).unwrap();
        self.stdout.flush().unwrap();

        loop {
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();
            if buf[0] == b' ' {
                return;
            }
        }
    }

    // Initialize the world by placing cells.
    fn init(&mut self) {
        loop {
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();
            match buf[0] {
                b'p' => return,
                b' ' => self.flip_cell(),
                b'w' | b'k' => self.move_cursor(Direction::N),
                b'd' | b'l' => self.move_cursor(Direction::E),
                b's' | b'j' => self.move_cursor(Direction::S),
                b'a' | b'h' => self.move_cursor(Direction::W),
                _ => {}
            }
            self.draw_world('o','-');
            self.draw_init_bar();
            self.draw_cursor();
            self.stdout.flush().unwrap();
            sleep(Duration::from_millis(10));
        }
    }

    fn draw_cursor(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}",
            cursor::Goto(self.cursor.y as u16 + 2, self.cursor.x as u16 + 2),
            style::Bold,
            "o",
            style::Reset,
        ).unwrap();
    }

    fn draw_init_bar(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}{}",
            clear::CurrentLine,
            style::Bold,
            cursor::Goto(((self.term_width - 60) / 2) as u16, 1),
            "Use wasd/kjhl to move, spacebar to switch cell, 'p' to start.",
            style::Reset
        ).unwrap();
    }

    fn draw_pause_bar(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}{}",
            clear::CurrentLine,
            style::Bold,
            cursor::Goto(((self.term_width - 30) / 2) as u16, 1),
            "Paused. Press spacebar to resume",
            style::Reset
        ).unwrap();
    }

    fn draw_run_bar(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            clear::CurrentLine,
            style::Bold,
            cursor::Goto(2, 1),
            "Gen: ",
            self.generation,
            cursor::Goto(15, 1),
            "Delay: ",
            self.delay,
            "    ",
            cursor::Goto(30, 1),
            "+/- control delay speed, 'p' pause, 'q' quit.",
            style::Reset,
        ).unwrap();
    }

    fn flip_cell(&mut self) {
        self.world.cells[self.cursor.x][self.cursor.y] =
            !(self.world.cells[self.cursor.x][self.cursor.y]);
    }

    fn move_cursor(&mut self, dir: Direction) {
        self.cursor = wrap_index(&self.world, self.cursor, dir);
    }

    fn run(&mut self) {
        write!(self.stdout,"{}",clear::All).unwrap();
        self.stdout.flush().unwrap();
        loop {
            let mut buf = [0];
            self.stdin.read(&mut buf).unwrap();
            match buf[0] {
                b'q' => break,
                b'+' => self.speed_up(),
                b'-' => self.speed_down(),
                b'p' => {
                    self.draw_pause_bar();
                    self.stdout.flush().unwrap();
                    loop {
                        self.stdin.read(&mut buf).unwrap();
                        match buf[0] {
                            b' ' => break,
                            _    => {}
                        }
                    }
                }
                _ => {}
            }
            self.advance_world();
            self.draw_world('o',' ');
            self.draw_run_bar();
            self.stdout.flush().unwrap();
            sleep(Duration::from_millis(self.delay as u64));
        }
        write!(
            self.stdout,
            "{}{}{}",
            color::Fg(color::Reset),
            clear::All,
            cursor::Goto(1, 1)
        ).unwrap();
        self.stdout.flush().unwrap();
    }

    fn speed_up(&mut self) {
        if self.delay > 10 {
            self.delay = self.delay - 10;
        }
    }

    fn speed_down(&mut self) {
        self.delay = self.delay + 10;
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let stdin = async_stdin();

    let height: usize;
    let width: usize;
    write!(
        stdout,
        "{}{}{}",
        clear::All,
        cursor::Hide,
        cursor::Goto(1, 1),
    ).unwrap();

    match terminal_size() {
        Ok((term_width, term_height)) => {
            height = term_height as usize;
            width = term_width as usize;
        }
        _err => {
            height = 32;
            width = 32;
            write!(
                stdout,
                "Couldn't read terminal size, using default size (32 x 32) for world."
            ).unwrap();
            stdout.flush().unwrap();
            sleep(Duration::from_millis(2000));
        }
    }

    let mut game = Game {
        world: build_empty_world(width-2, height-2),
        generation: 0,
        delay: 100,
        cursor: pos(1, 1),
        stdin: stdin,
        stdout: stdout,
        term_width : width as u16,
        term_height : height as u16,
    };
    game.splash();
    game.init();
    game.run()
}

// Give next state of a cell in a world.
// The rules are as follows:
//    Any live cell with fewer than two live neighbors dies, as if by under population.
//    Any live cell with two or three live neighbors lives on to the next generation.
//    Any live cell with more than three live neighbors dies, as if by overpopulation.
//    Any dead cell with exactly three live neighbors becomes a live cell, as if by reproduction.
// See [https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life#Rules]
fn next_cell_state(world: &mut World, pos: Pos) -> bool {
    let neighbours = neighbours(&world, pos);
    if world.cells[pos.x][pos.y] {
        neighbours == 2 || neighbours == 3
    } else {
        neighbours == 3
    }
}

// The below function finds the number of neightbours of given cell in a world
fn neighbours(world: &World, cell_pos: Pos) -> usize {
    let mut neighbours: [Pos; 8] = [pos(0, 0); 8];
    neighbours[0] = wrap_index(world, cell_pos, Direction::NW);
    neighbours[1] = wrap_index(world, cell_pos, Direction::N);
    neighbours[2] = wrap_index(world, cell_pos, Direction::NE);
    neighbours[3] = wrap_index(world, cell_pos, Direction::W);
    neighbours[4] = wrap_index(world, cell_pos, Direction::E);
    neighbours[5] = wrap_index(world, cell_pos, Direction::SE);
    neighbours[6] = wrap_index(world, cell_pos, Direction::S);
    neighbours[7] = wrap_index(world, cell_pos, Direction::SW);

    let mut sum = 0;
    for cell in &neighbours {
        if world.cells[cell.x][cell.y] {
            sum = sum + 1;
        }
    }
    sum
}

// We consider the world to "wrap around", meaning that in a 10 x 10 world,
// the cell at (0,0) has (0,9) and (9,0) as (among others) neighbours.
fn wrap_index(world: &World, pos: Pos, dir: Direction) -> Pos {
    let wrapped_x: usize;
    let wrapped_y: usize;
    match dir {
        Direction::NW => {
            if pos.x == 0 {
                wrapped_x = world.height - 1;
            } else {
                wrapped_x = pos.x - 1;
            }
            if pos.y == 0 {
                wrapped_y = world.width - 1;
            } else {
                wrapped_y = pos.y - 1;
            }
        }
        Direction::N => {
            if pos.x == 0 {
                wrapped_x = world.height - 1;
            } else {
                wrapped_x = pos.x - 1;
            }
            wrapped_y = pos.y;
        }
        Direction::NE => {
            if pos.x == 0 {
                wrapped_x = world.height - 1;
            } else {
                wrapped_x = pos.x - 1;
            }
            if pos.y == world.width - 1 {
                wrapped_y = 0
            } else {
                wrapped_y = pos.y + 1;
            }
        }
        Direction::W => {
            if pos.y == 0 {
                wrapped_y = world.width - 1;
            } else {
                wrapped_y = pos.y - 1;
            }
            wrapped_x = pos.x;
        }
        Direction::E => {
            if pos.y == world.width - 1 {
                wrapped_y = 0
            } else {
                wrapped_y = pos.y + 1;
            }
            wrapped_x = pos.x;
        }
        Direction::SW => {
            if pos.x == world.height - 1 {
                wrapped_x = 0;
            } else {
                wrapped_x = pos.x + 1;
            }
            if pos.y == 0 {
                wrapped_y = world.width - 1;
            } else {
                wrapped_y = pos.y - 1;
            }
        }
        Direction::S => {
            if pos.x == world.height - 1 {
                wrapped_x = 0;
            } else {
                wrapped_x = pos.x + 1;
            }
            wrapped_y = pos.y;
        }
        Direction::SE => {
            if pos.x == world.height - 1 {
                wrapped_x = 0;
            } else {
                wrapped_x = pos.x + 1;
            }
            if pos.y == world.width - 1 {
                wrapped_y = 0;
            } else {
                wrapped_y = pos.y + 1;
            }
        }
    }
    Pos {
        x: wrapped_x,
        y: wrapped_y,
    }
}
