#![no_std]

use bitflags::bitflags;
use derive_more::{Deref, DerefMut};

/// Number of colors in the palette buffer
pub const PALETTE_COLORS: usize = 32;

#[derive(Deref, DerefMut)]
pub struct ScreenBuffer(pub [u8; Self::NUM_PIXELS]);

impl Default for ScreenBuffer {
    fn default() -> Self {
        ScreenBuffer([0; Self::NUM_PIXELS])
    }
}

/// todo: some of these could just be an extension trait? Maybe a crate already exists?
impl ScreenBuffer {
    /// Width of the screen buffer in pixels
    pub const WIDTH: usize = 320;

    /// Height of the screen buffer in pixels
    pub const HEIGHT: usize = 240;

    /// Number of pixels in the screen buffer
    pub const NUM_PIXELS: usize = Self::WIDTH * Self::HEIGHT;

    /// Set a single pixel to the given color
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        debug_assert!(x < Self::WIDTH);
        let i = x + y * Self::WIDTH;
        self[i] = color;
    }

    /// Gets a pixel
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        debug_assert!(x < Self::WIDTH);
        let i = x + y * Self::WIDTH;
        self[i]
    }
}

#[cfg(feature = "rastateur")]
impl rastateur::PixelBuffer<u8> for ScreenBuffer {
    #[inline]
    fn set_pixel<TPos: Into<(i32, i32)>>(&mut self, pos: TPos, color: u8) {
        let (x, y) = pos.into();
        self.set_pixel(x as usize, y as usize, color);
    }

    #[inline]
    fn get_pixel<TPos: Into<(i32, i32)>>(&self, pos: TPos) -> u8 {
        let (x, y) = pos.into();
        self.get_pixel(x as usize, y as usize)
    }

    #[inline]
    fn width(&self) -> i32 {
        Self::WIDTH as i32
    }

    #[inline]
    fn height(&self) -> i32 {
        Self::HEIGHT as i32
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
