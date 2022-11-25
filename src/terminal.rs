#![allow(non_camel_case_types)]

use crate::frame_buffer::{Color, Pixel};
use crate::types::{Dimensions, Position};

type c_int = i32;
#[cfg(target_os = "macos")]
type c_ulong = u64;
#[cfg(not(target_os = "macos"))]
type c_uint = u32;
type c_uchar = u8;

#[cfg(target_os = "macos")]
pub type tcflag_t = c_ulong;
#[cfg(not(target_os = "macos"))]
pub type tcflag_t = c_uint;
type cc_t = c_uchar;

const NCCS: usize = 32;
const ECHO: tcflag_t = 0o000010;
#[cfg(target_os = "macos")]
const ICANON: tcflag_t = 0x0000100;
#[cfg(not(target_os = "macos"))]
const ICANON: tcflag_t = 0o0000002;

#[repr(C)]
#[derive(Debug)]
struct Termios {
    c_iflag: tcflag_t,  // input modes
    c_oflag: tcflag_t,  // output modes
    c_cflag: tcflag_t,  // control modes
    c_lflag: tcflag_t,  // local modes
    c_cc: [cc_t; NCCS], // special characters
}

#[link(name = "c")]
extern "C" {
    fn ioctl(fildes: c_int, request: c_int, ...) -> c_int;
    fn tcgetattr(fd: c_int, termios_p: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const Termios) -> c_int;
}

const ESC: u8 = 0x1b;

pub fn get_dimenions() -> Result<Dimensions, &'static str> {
    use std::mem;

    #[cfg(target_os = "macos")]
    const TIOCGWINSZ: i32 = 0x40087468;
    #[cfg(not(target_os = "macos"))]
    const TIOCGWINSZ: i32 = 0x5413;

    struct Winsize {
        ws_row: u16,
        ws_col: u16,
        _ws_xpixel: u16, // unused
        _ws_ypixel: u16, // unused
    }

    use std::os::unix::io::AsRawFd;
    let stdin_fd = std::io::stdin().as_raw_fd();

    let winsize = unsafe {
        let mut winsize = mem::MaybeUninit::<Winsize>::uninit();
        let res = ioctl(stdin_fd, TIOCGWINSZ, winsize.as_mut_ptr());
        if res == -1 {
            return Err("Could not get terminal dimensions.");
        }
        winsize.assume_init()
    };

    Ok(Dimensions {
        x: winsize.ws_col as usize,
        y: winsize.ws_row as usize,
    })
}

impl Color {
    pub fn encode_ascii(&self, buffer: &mut [u8]) -> usize {
        let color_code = match self {
            Color::Default => &[0x30u8][..],
            Color::White => &[0x33u8, 0x37u8][..],
            Color::Black => &[0x33u8, 0x30u8][..],
            Color::Red => &[0x33u8, 0x31u8][..],
            Color::Green => &[0x33u8, 0x32u8][..],
            Color::Blue => &[0x33u8, 0x34u8][..],
            Color::Yellow => &[0x33u8, 0x33u8][..],
        };
        buffer[0] = ESC;
        buffer[1] = 0x5b;
        let mut i: usize = 2;
        for code in color_code {
            buffer[i] = *code;
            i += 1
        }
        buffer[i] = 0x6d;
        i + 1
    }
}

impl Pixel {
    pub fn encode_ascii(&self, buffer: &mut [u8]) -> usize {
        let res = self.character.encode_utf8(buffer);
        res.len()
    }
}

impl Position {
    pub fn encode_ascii(&self, buffer: &mut [u8]) -> usize {
        buffer[0] = ESC;
        buffer[1] = 0x5b;
        let mut i: usize = 2;
        let pos_y_str = (self.y + 1).to_string();
        for c in pos_y_str.chars().map(|c| c as u8) {
            buffer[i] = c;
            i += 1;
        }
        buffer[i] = 0x3b;
        i += 1;
        let pos_x_str = (self.x + 1).to_string();
        for c in pos_x_str.chars().map(|c| c as u8) {
            buffer[i] = c;
            i += 1;
        }
        buffer[i] = 0x48;
        i + 1
    }
}

pub fn reset() {
    print!("\x1bc");
}

pub fn hide_cursor() {
    print!("\x1b\x5b?25l");
}

pub fn set_mode(enable: bool) {
    use std::os::unix::io::AsRawFd;
    let stdin_fd = std::io::stdin().as_raw_fd();
    let mut termios = unsafe {
        let mut termios = std::mem::MaybeUninit::<Termios>::uninit();
        let res = tcgetattr(stdin_fd, termios.as_mut_ptr());
        if res != 0 {
            panic!("tcgetattr failed.");
        }
        termios.assume_init()
    };

    if enable {
        termios.c_lflag |= ECHO | ICANON;
    } else {
        termios.c_lflag &= !(ECHO | ICANON);
    }
    unsafe {
        let res = tcsetattr(stdin_fd, 0, &termios);
        if res != 0 {
            panic!("tcsetattr failed.");
        }
    }
}
