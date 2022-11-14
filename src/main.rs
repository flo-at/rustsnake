mod terminal;
mod types;

use crate::terminal::{Color, Pixel};
use crate::types::{Dimensions, Matrix2, Position};

struct FrameBuffer {
    dimensions: Dimensions,
    buffer1: Matrix2<Pixel>,
    buffer2: Matrix2<Pixel>,
    buffer1_is_front: bool,
    command_cache: Vec<u8>,
}

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
        dimensions.x * dimensions.y * (1 + 5 + 10)
    }

    fn update_command_cache(&mut self) -> &[u8] {
        let (front_buffer, back_buffer) = match self.buffer1_is_front {
            true => (&self.buffer1, &self.buffer2),
            false => (&self.buffer2, &self.buffer1),
        };

        let mut position = Position::new(0, 0);
        let mut last_color = Color::default();
        let mut i: usize = 0;
        for (pixel1, pixel2) in front_buffer.iter().zip(back_buffer.iter()) {
            let mut force_draw_char = false;
            if *pixel1 != *pixel2 {
                i += position.encode_ascii(&mut self.command_cache[i..]);
            }
            if pixel1.color != pixel2.color {
                if pixel1.color != last_color {
                    i += pixel1.color.encode_ascii(&mut self.command_cache[i..]);
                    last_color = pixel1.color;
                }
                force_draw_char = true;
            }
            if force_draw_char || pixel1.character != pixel2.character {
                self.command_cache[i] = pixel1.character;
                i += 1;
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

fn draw_border(frame_buffer: &mut FrameBuffer) {
    let dimensions = frame_buffer.dimensions().clone();
    let back_buffer = frame_buffer.back_buffer();
    const CHARACTER: u8 = 0x2b;
    const COLOR: Color = Color::Yellow;
    for x in 0..dimensions.x {
        back_buffer.set(
            x,
            0,
            Pixel {
                character: CHARACTER,
                color: COLOR,
            },
        );
        back_buffer.set(
            x,
            dimensions.y - 1,
            Pixel {
                character: CHARACTER,
                color: COLOR,
            },
        );
    }
    for y in 1..dimensions.y - 1 {
        back_buffer.set(
            0,
            y,
            Pixel {
                character: CHARACTER,
                color: COLOR,
            },
        );
        back_buffer.set(
            dimensions.x - 1,
            y,
            Pixel {
                character: CHARACTER,
                color: COLOR,
            },
        );
    }
}

fn main() {
    terminal::set_echo(false);
    terminal::reset();
    terminal::hide_cursor();
    let dimensions = terminal::get_terminal_dimenions().unwrap();
    let mut frame_buffer = FrameBuffer::new(&dimensions);
    for i in 0..5 {
        draw_border(&mut frame_buffer);
        let back_buffer = frame_buffer.back_buffer();
        back_buffer.set(
            10 + i,
            10,
            Pixel {
                character: 0x2b,
                color: Color::Green,
            },
        );
        back_buffer.set(
            11 + i,
            11,
            Pixel {
                character: 0x2b,
                color: Color::Red,
            },
        );
        frame_buffer.swap_buffers();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
    terminal::set_echo(true);
    terminal::reset();
}
