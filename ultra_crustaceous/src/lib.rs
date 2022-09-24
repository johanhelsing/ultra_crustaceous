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

/// todo: some of these could just be an extension trait? Maybe a crate already exists?
impl OutputBuffer {
    /// Set a single pixel to the given color
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        debug_assert!(x < SCREEN_WIDTH);
        let i = x + y * SCREEN_WIDTH;
        self[i] = color;
    }

    /// Gets a pixel
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        debug_assert!(x < SCREEN_WIDTH);
        let i = x + y * SCREEN_WIDTH;
        self[i]
    }
}

#[cfg(feature = "rastateur")]
impl rastateur::PixelBuffer<u8> for OutputBuffer {
    #[inline]
    fn set_pixel<TPos: Into<(usize, usize)>>(&mut self, pos: TPos, color: u8) {
        let (x, y) = pos.into();
        self.set_pixel(x, y, color);
    }

    #[inline]
    fn get_pixel<TPos: Into<(usize, usize)>>(&self, pos: TPos) -> u8 {
        let (x, y) = pos.into();
        self.get_pixel(x, y)
    }

    #[inline]
    fn width(&self) -> usize {
        SCREEN_WIDTH
    }

    #[inline]
    fn height(&self) -> usize {
        SCREEN_HEIGHT
    }
}

// todo: maybe do blanket impls for this?
#[cfg(feature = "rastateur")]
impl rastateur::PixelBuffer<u8, i32> for OutputBuffer {
    #[inline]
    fn set_pixel<TPos: Into<(i32, i32)>>(&mut self, pos: TPos, color: u8) {
        let (x, y) = pos.into();
        OutputBuffer::set_pixel(self, x as usize, y as usize, color);
    }

    #[inline]
    fn get_pixel<TPos: Into<(i32, i32)>>(&self, pos: TPos) -> u8 {
        let (x, y) = pos.into();
        OutputBuffer::get_pixel(self, x as usize, y as usize)
    }

    #[inline]
    fn width(&self) -> i32 {
        SCREEN_WIDTH as i32
    }

    #[inline]
    fn height(&self) -> i32 {
        SCREEN_HEIGHT as i32
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
