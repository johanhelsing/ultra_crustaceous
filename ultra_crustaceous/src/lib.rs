#![no_std]

use bitflags::bitflags;
use derive_more::{Deref, DerefMut};

/// Width of the screen buffer in pixels
pub const SCREEN_WIDTH: usize = 320;

/// Height of the screen buffer in pixels
pub const SCREEN_HEIGHT: usize = 240;

/// Number of pixels in the screen buffer
pub const OUTPUT_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

/// Number of colors in the palette buffer
pub const PALETTE_COLORS: usize = 32;

#[derive(Deref, DerefMut)]
pub struct OutputBuffer(pub [u8; OUTPUT_BUFFER_SIZE]);

impl Default for OutputBuffer {
    fn default() -> Self {
        OutputBuffer([0; OUTPUT_BUFFER_SIZE])
    }
}

impl OutputBuffer {
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        debug_assert!(x < SCREEN_WIDTH);
        debug_assert!(y < SCREEN_HEIGHT);
        let i = x + y * SCREEN_WIDTH;
        self[i] = color;
    }

    pub fn blit_color(&mut self, x: usize, y: usize, width: usize, height: usize, color: u8) {
        for y in y..y + height {
            for x in x..x + width {
                self.set_pixel(x, y, color);
            }
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct PaletteBuffer([Color; PALETTE_COLORS]);

impl Default for PaletteBuffer {
    fn default() -> Self {
        PaletteBuffer([Color(0, 0); PALETTE_COLORS])
    }
}

// rename to palette color?
#[derive(Clone, Copy)]
pub struct Color(u8, u8);

impl Color {
    pub const fn from_rgb(rgb: u32) -> Self {
        let b = ((rgb & 0xff) >> 4) as u8;
        let g = (((rgb & 0xff00) >> 8) >> 4) as u8;
        let r = (((rgb & 0xff0000) >> 16) >> 4) as u8;
        Color(r, (g << 4) | b)
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Input: u8 {
        const UP = 1 << 0;
        const DOWN = 1 << 1;
        const LEFT = 1 << 2;
        const RIGHT = 1 << 3;
        const BUTTON_1 = 1 << 4;
        const BUTTON_2 = 1 << 5;
    }
}

impl Input {
    pub const fn x(self) -> i32 {
        match (self.contains(Input::LEFT), self.contains(Input::RIGHT)) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        }
    }

    pub const fn y(self) -> i32 {
        match (self.contains(Input::DOWN), self.contains(Input::UP)) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        }
    }
}
