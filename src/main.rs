use rustsnake::cyclic_buffer;
use rustsnake::frame_buffer::{Color, FrameBuffer, Pixel};
use rustsnake::random;
use rustsnake::terminal;
use rustsnake::types::{Dimensions, Position};

const FOOD_CHAR: char = 'x';
const FOOD_COLOR: Color = Color::Green;
const FOOD_SCORE: usize = 100;

const WALL_CHAR: char = '█';
const WALL_COLOR: Color = Color::Yellow;

const SNAKE_CHAR: char = '◉';
const SNAKE_COLOR: Color = Color::Blue;

const SCORE_COLOR: Color = Color::Red;
const SPEED_COLOR: Color = SCORE_COLOR;

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
    for (i, character) in format!("Score: {}", score).chars().enumerate() {
        back_buffer.set(
            i + 1,
            dimensions.y - 1,
            Pixel {
                character,
                color: SCORE_COLOR,
            },
        );
    }
}

fn draw_speed(speed: usize, dimensions: &Dimensions, frame_buffer: &mut FrameBuffer) {
    let back_buffer = frame_buffer.back_buffer();
    for (i, character) in format!("Speed: {}", speed).chars().rev().enumerate() {
        back_buffer.set(
            dimensions.x - i - 2,
            dimensions.y - 1,
            Pixel {
                character,
                color: SPEED_COLOR,
            },
        );
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn is_opposite(&self, other: Direction) -> bool {
        *self == Self::Up && other == Self::Down
            || *self == Self::Down && other == Self::Up
            || *self == Self::Left && other == Self::Right
            || *self == Self::Right && other == Self::Left
    }
}

struct Snake {
    segments: cyclic_buffer::CyclicBuffer<Position>,
    pub direction: Direction,
    score: usize,
}

impl Snake {
    fn new(dimensions: &Dimensions) -> Self {
        let mut segments = cyclic_buffer::CyclicBuffer::new(Self::max_segments(dimensions));
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
        let head_id = self.segments.count() - 1;
        let hit_wall =
            head.x < 1 || head.x >= dimensions.x - 1 || head.y < 1 || head.y >= dimensions.y - 1;
        let bit_self = self
            .segments
            .iter()
            .enumerate()
            .any(|(id, x)| x == head && id != head_id);
        !hit_wall && !bit_self
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
        blocked_fields: &mut (impl core::iter::Iterator<Item = &'a Position> + Clone),
    ) -> Self
    where
        u32: From<<T as random::RandomNumberEngine>::ResultType>,
    {
        let fields_total = (dimensions.x - 2) * (dimensions.y - 2);
        let rand: u32 = rng.get().into();
        let rand: usize = rand as usize % (fields_total - blocked_fields.clone().count());
        let mut free_fields: Vec<Position> = Vec::new();
        free_fields.reserve_exact(fields_total);
        for y in 1..dimensions.y - 1 {
            for x in 1..dimensions.x - 1 {
                let position = Position { x, y };
                if !blocked_fields.clone().any(|x| *x == position) {
                    free_fields.push(Position { x, y });
                }
            }
        }
        Self {
            position: free_fields.into_iter().nth(rand).unwrap(),
        }
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

fn get_direction_from_stdin(rx: &std::sync::mpsc::Receiver<u8>) -> Option<Direction> {
    let mut direction: Option<Direction> = None;

    for byte in rx.try_iter() {
        match byte {
            b'w' => direction = Some(Direction::Up),
            b's' => direction = Some(Direction::Down),
            b'a' => direction = Some(Direction::Left),
            b'd' => direction = Some(Direction::Right),
            _ => {}
        }
    }
    direction
}

fn main() {
    terminal::set_mode(false);
    terminal::reset();
    terminal::hide_cursor();
    let dimensions = terminal::get_dimenions().unwrap();
    let field_dimensions = Dimensions {
        x: dimensions.x,
        y: dimensions.y - 1,
    };
    let mut rng = random::PCG32Fast::new(None);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut stdin = std::io::stdin();
        let mut buffer = [0u8; 1];
        loop {
            stdin.read_exact(&mut buffer).unwrap();
            tx.send(*buffer.first().unwrap()).unwrap();
        }
    });

    let mut frame_buffer = FrameBuffer::new(&dimensions);
    let mut snake = Snake::new(&field_dimensions);
    let mut food = Food::new(&field_dimensions, &mut rng, &mut snake.segments());
    let mut speed = 0;
    loop {
        if snake.tick(&food) {
            food = Food::new(&field_dimensions, &mut rng, &mut snake.segments());
            speed = std::cmp::min(speed + 5, 50);
        }
        // TODO warum wird eine Zahl in der Score blaue (Farbe der Snake)?
        draw_border(&field_dimensions, &mut frame_buffer);
        draw_score(snake.score(), &dimensions, &mut frame_buffer);
        draw_speed(speed, &dimensions, &mut frame_buffer);
        snake.draw(&mut frame_buffer);
        let new_direction = get_direction_from_stdin(&rx).unwrap_or(snake.direction);
        if !new_direction.is_opposite(snake.direction) {
            snake.direction = new_direction;
        }
        food.draw(&mut frame_buffer);
        frame_buffer.swap_buffers();
        std::thread::sleep(std::time::Duration::from_millis(100 - speed as u64));
        if !snake.alive(&field_dimensions) || snake.won() {
            break;
        }
    }
    terminal::set_mode(true);
    terminal::reset();
    println!("Final score: {}", snake.score());
}
