use bitflags::bitflags;
use lazy_static::lazy_static;
use std::sync::RwLock;
use wasm_bindgen::prelude::*;

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 240;
const OUTPUT_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

lazy_static! {
    static ref GAME: RwLock<SnakeGame> = RwLock::new(SnakeGame {
        output_buffer: [0; OUTPUT_BUFFER_SIZE]
    });
}

#[wasm_bindgen]
pub fn get_screen_buffer_pointer() -> *const u8 {
    let game = GAME.read().expect("couldn't get game read lock");
    game.output_buffer.as_ptr()
}

#[wasm_bindgen]
pub fn update(p1: u8, p2: u8) {
    let p1 = Input::from_bits_truncate(p1);
    let p2 = Input::from_bits_truncate(p2);

    GAME.write()
        .expect("Failed to get game write lock")
        .update(p1, p2);
}

bitflags! {
    struct Input: u8 {
        const UP = 1 << 0;
        const DOWN = 1 << 1;
        const LEFT = 1 << 2;
        const RIGHT = 1 << 3;
        const BUTTON_1 = 1 << 4;
        const BUTTON_2 = 1 << 5;
    }
}
// the above will probably be the same for 99% of games, move to a crate / macro?

const TILE_SIZE: usize = 10;
struct SnakeGame {
    output_buffer: [u8; OUTPUT_BUFFER_SIZE],
}

impl SnakeGame {
    fn update(&mut self, p1: Input, p2: Input) {
        let input = p1.union(p2); // let either joystick control
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let offset = if !input.is_empty() { 1 } else { 0 };
                let is_dark_square = (y / TILE_SIZE) % 2 != (x / TILE_SIZE) % 2 + offset;
                let color = if is_dark_square { 0 } else { 1 };
                let pixel_index = y * SCREEN_WIDTH + x;
                self.output_buffer[pixel_index] = color;
            }
        }
    }
}
