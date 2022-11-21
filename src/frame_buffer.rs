use crate::types::{Dimensions, Matrix2, Position};

#[derive(Clone, PartialEq, Copy)]
pub struct Pixel {
    pub character: char,
    pub color: Color,
}

impl std::default::Default for Pixel {
    fn default() -> Self {
        Pixel {
            character: ' ',
            color: Color::default(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum Color {
    #[default]
    Default,
    White,
    Black,
    Red,
    Green,
    Blue,
    Yellow,
}

pub struct FrameBuffer {
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

    fn command_cache_size(dimensions: &Dimensions) -> usize {
        dimensions.x * dimensions.y * (4 + 5 + 10)
    }

    fn update_command_cache(&mut self) -> &[u8] {
        let (front_buffer, back_buffer) = match self.buffer1_is_front {
            true => (&self.buffer1, &self.buffer2),
            false => (&self.buffer2, &self.buffer1),
        };

        let mut position = Position { x: 0, y: 0 };
        let mut last_position = position.clone();
        let mut last_color = Color::default();
        let mut i: usize = 0;
        for (pixel1, pixel2) in front_buffer.iter().zip(back_buffer.iter()) {
            let mut force_draw_char = false;
            if *pixel1 != *pixel2
                && (position.y != last_position.y || position.x != last_position.x + 1)
            {
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
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(command_cache).unwrap();
        stdout.flush().unwrap();
        self.back_buffer().clear();
    }
}
