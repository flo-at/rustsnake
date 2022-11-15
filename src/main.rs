mod cyclic_buffer;
mod random;
mod terminal;
mod types;

// TODO implement signal handler (sigaction from signal.h)

use crate::cyclic_buffer::CyclicBuffer;
use crate::terminal::{Color, Pixel};
use crate::types::{Dimensions, Matrix2, Position};

struct FrameBuffer {
    dimensions: Dimensions,
    buffer1: Matrix2<Pixel>,
    buffer2: Matrix2<Pixel>,
    buffer1_is_front: bool,
    command_cache: Vec<u8>,
}

const FOOD_CHAR: char = '◈';
const FOOD_COLOR: Color = Color::Green;
const FOOD_SCORE: usize = 100;

const WALL_CHAR: char = '█';
const WALL_COLOR: Color = Color::Yellow;

const SNAKE_CHAR: char = '◉';
const SNAKE_COLOR: Color = Color::Blue;

const SCORE_COLOR: Color = Color::Red;
const SPEED_COLOR: Color = SCORE_COLOR;

impl FrameBuffer {
    pub fn new(dimensions: &Dimensions) -> Self {
        Self {
            dimensions: dimensions.clone(),
            buffer1: Matrix2::<Pixel>::new(dimensions),
            buffer2: Matrix2::<Pixel>::new(dimensions),
            buffer1_is_front: true,
            command_cache: vec![0; Self::command_cache_size(dimensions)],
        }
    }

    fn dimensions(&self) -> &Dimensions {
        &self.dimensions
    }

    fn command_cache_size(dimensions: &Dimensions) -> usize {
        dimensions.x * dimensions.y * (4 + 5 + 10)
    }

    fn update_command_cache(&mut self) -> &[u8] {
        let (front_buffer, back_buffer) = match self.buffer1_is_front {
            true => (&self.buffer1, &self.buffer2),
            false => (&self.buffer2, &self.buffer1),
        };

        let mut position = Position::new(0, 0);
        let mut last_position = position.clone();
        let mut last_color = Color::default();
        let mut i: usize = 0;
        for (pixel1, pixel2) in front_buffer.iter().zip(back_buffer.iter()) {
            let mut force_draw_char = false;
            if *pixel1 != *pixel2 {
                if position.y != last_position.y || position.x != last_position.x + 1 {
                    i += position.encode_ascii(&mut self.command_cache[i..]);
                }
            }
            if pixel1.color != pixel2.color {
                if pixel1.color != last_color {
                    i += pixel1.color.encode_ascii(&mut self.command_cache[i..]);
                    last_color = pixel1.color;
                }
                force_draw_char = true;
            }
            if force_draw_char || pixel1.character != pixel2.character {
                i += pixel1.encode_ascii(&mut self.command_cache[i..]);
                last_position = position.clone();
            }
            position.x += 1;
            if position.x == self.dimensions.x {
                position.x = 0;
                position.y += 1;
            }
        }
        &self.command_cache[0..i]
    }

    pub fn back_buffer(&mut self) -> &mut Matrix2<Pixel> {
        match self.buffer1_is_front {
            true => &mut self.buffer2,
            false => &mut self.buffer1,
        }
    }

    pub fn swap_buffers(&mut self) {
        use std::io::Write;

        self.buffer1_is_front = !self.buffer1_is_front;
        let command_cache = self.update_command_cache();
        // TODO serialize diff_buffer into the u8 cache and print it
        let mut stdout = std::io::stdout().lock();
        stdout.write(command_cache).unwrap();
        stdout.flush().unwrap();
        self.back_buffer().clear();
    }
}

fn draw_border(dimensions: &Dimensions, frame_buffer: &mut FrameBuffer) {
    let back_buffer = frame_buffer.back_buffer();
    for x in 0..dimensions.x {
        back_buffer.set(
            x,
            0,
            Pixel {
                character: WALL_CHAR,
                color: WALL_COLOR,
            },
        );
        back_buffer.set(
            x,
            dimensions.y - 1,
            Pixel {
                character: WALL_CHAR,
                color: WALL_COLOR,
            },
        );
    }
    for y in 1..dimensions.y - 1 {
        back_buffer.set(
            0,
            y,
            Pixel {
                character: WALL_CHAR,
                color: WALL_COLOR,
            },
        );
        back_buffer.set(
            dimensions.x - 1,
            y,
            Pixel {
                character: WALL_CHAR,
                color: WALL_COLOR,
            },
        );
    }
}

fn draw_score(score: usize, dimensions: &Dimensions, frame_buffer: &mut FrameBuffer) {
    let back_buffer = frame_buffer.back_buffer();
    let mut i: usize = 0;
    for character in format!("Score: {}", score).chars() {
        back_buffer.set(
            i + 1,
            dimensions.y - 1,
            Pixel {
                character,
                color: SCORE_COLOR,
            },
        );
        i += 1;
    }
}

fn draw_speed(speed: usize, dimensions: &Dimensions, frame_buffer: &mut FrameBuffer) {
    let back_buffer = frame_buffer.back_buffer();
    let mut i: usize = 0;
    for character in format!("Speed: {}", speed).chars().rev() {
        back_buffer.set(
            dimensions.x - i - 2,
            dimensions.y - 1,
            Pixel {
                character,
                color: SPEED_COLOR,
            },
        );
        i += 1;
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Snake {
    segments: CyclicBuffer<Position>,
    pub direction: Direction,
    score: usize,
}

impl Snake {
    fn new(dimensions: &Dimensions) -> Self {
        let mut segments = CyclicBuffer::new(Self::max_segments(dimensions));
        segments.push(Position {
            x: dimensions.x / 2,
            y: dimensions.y / 2,
        });
        Self {
            segments,
            direction: Direction::Right,
            score: 0,
        }
    }

    pub fn segments(&self) -> cyclic_buffer::Iter<'_, Position> {
        self.segments.iter()
    }

    fn score(&self) -> usize {
        self.score
    }

    fn max_segments(dimensions: &Dimensions) -> usize {
        (dimensions.x - 2) * (dimensions.y - 2)
    }

    fn draw(&self, frame_buffer: &mut FrameBuffer) {
        let back_buffer = frame_buffer.back_buffer();
        for segment in self.segments.iter() {
            back_buffer.set(
                segment.x,
                segment.y,
                Pixel {
                    character: SNAKE_CHAR,
                    color: SNAKE_COLOR,
                },
            );
        }
    }

    fn won(&self) -> bool {
        self.segments.full()
    }

    fn tick(&mut self, food: &Food) -> bool {
        let head = self.segments.iter().last().unwrap();
        let new_head = match self.direction {
            Direction::Up => Position {
                x: head.x,
                y: head.y - 1,
            },
            Direction::Down => Position {
                x: head.x,
                y: head.y + 1,
            },
            Direction::Left => Position {
                x: head.x - 1,
                y: head.y,
            },
            Direction::Right => Position {
                x: head.x + 1,
                y: head.y,
            },
        };
        let eat = self.eat(food);
        if eat {
            self.score += FOOD_SCORE;
        } else {
            self.segments.pop();
        }
        self.segments.push(new_head);
        eat
    }

    fn alive(&self, dimensions: &Dimensions) -> bool {
        let head = self.segments.iter().last().unwrap();
        head.x > 1 && head.x < dimensions.x - 1 && head.y > 1 && head.y < dimensions.y - 1
    }

    fn eat(&self, food: &Food) -> bool {
        let head = self.segments.iter().last().unwrap();
        *head == food.position
    }
}

struct Food {
    pub position: Position,
}

impl Food {
    fn new<'a, T: random::RandomNumberEngine>(
        dimensions: &Dimensions,
        rng: &mut T,
        blocked_fields: &mut impl core::iter::Iterator<Item = &'a Position>,
    ) -> Self
    where
        u32: From<<T as random::RandomNumberEngine>::ResultType>,
    {
        let fields_total = (dimensions.x - 2) * (dimensions.y - 2);
        let rand: u32 = rng.get().into();
        let rand: usize = rand as usize % (fields_total - blocked_fields.count());
        let mut free_fields: Vec<Position> = Vec::new();
        free_fields.reserve_exact(fields_total);
        for y in 1..dimensions.y - 1 {
            for x in 1..dimensions.x - 1 {
                let position = Position { x, y };
                if !blocked_fields.any(|x| *x == position) {
                    free_fields.push(Position { x, y });
                }
            }
        }
        Self {
            position: free_fields.into_iter().nth(rand as usize).unwrap(),
        }
    }

    fn advance(position: &mut Position, dimensions: &Dimensions) {
        position.x += 1;
        if position.x == dimensions.x - 2 {
            position.x = 1;
            position.y += 1;
        }
    }

    fn inside_snake(position: &Position, snake_segments: &[Position]) -> bool {
        snake_segments.iter().any(|x| x == position)
    }

    fn draw(&self, frame_buffer: &mut FrameBuffer) {
        let back_buffer = frame_buffer.back_buffer();
        back_buffer.set(
            self.position.x,
            self.position.y,
            Pixel {
                character: FOOD_CHAR,
                color: FOOD_COLOR,
            },
        );
    }
}

fn get_direction_from_stdin() -> Option<Direction> {
    let mut direction: Option<Direction> = None;
    if let Ok(n) = terminal::get_bytes_at_stdin() {
        use std::io::Read;
        for _ in 0..n {
            let stdin = std::io::stdin().lock();
            match stdin.bytes().next().unwrap() {
                Ok(b'w') => direction = Some(Direction::Up),
                Ok(b's') => direction = Some(Direction::Down),
                Ok(b'a') => direction = Some(Direction::Left),
                Ok(b'd') => direction = Some(Direction::Right),
                _ => (),
            }
        }
    }
    direction
}

fn main() {
    terminal::set_echo(false);
    terminal::reset();
    terminal::hide_cursor();
    let dimensions = terminal::get_terminal_dimenions().unwrap();
    let field_dimensions = Dimensions {
        x: dimensions.x,
        y: dimensions.y - 1,
    };
    let mut rng = random::PCG32Fast::new(None);
    let mut frame_buffer = FrameBuffer::new(&dimensions);
    let mut snake = Snake::new(&field_dimensions);
    let mut food = Food::new(&field_dimensions, &mut rng, &mut snake.segments());
    let mut speed = 0;
    loop {
        // TODO warum wird eine Zahl in der Score blaue (Farbe der Snake)?
        draw_border(&field_dimensions, &mut frame_buffer);
        draw_score(snake.score(), &dimensions, &mut frame_buffer);
        draw_speed(speed, &dimensions, &mut frame_buffer);
        snake.draw(&mut frame_buffer);
        snake.direction = get_direction_from_stdin().unwrap_or(snake.direction);
        food.draw(&mut frame_buffer);
        frame_buffer.swap_buffers();
        std::thread::sleep(std::time::Duration::from_millis(100 - speed as u64));
        if snake.tick(&food) {
            food = Food::new(&field_dimensions, &mut rng, &mut snake.segments());
            speed = std::cmp::min(speed + 10, 100); // TODO
        }
        if !snake.alive(&field_dimensions) || snake.won() {
            break;
        }
    }
    terminal::set_echo(true);
    terminal::reset();
}
