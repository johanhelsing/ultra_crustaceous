use wasm_bindgen::prelude::*;

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 240;
const TILE_SIZE: usize = 10;
const OUTPUT_BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

static mut OUTPUT_BUFFER: [u8; OUTPUT_BUFFER_SIZE] = [0; OUTPUT_BUFFER_SIZE];

#[wasm_bindgen]
pub fn get_screen_buffer_pointer() -> *const u8 {
    unsafe { OUTPUT_BUFFER.as_ptr() }
}

#[wasm_bindgen]
pub fn update() {
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let is_dark_square = (y / TILE_SIZE) % 2 != (x / TILE_SIZE) % 2;
            let color = if is_dark_square { 0 } else { 1 };
            let pixel_index = y * SCREEN_WIDTH + x;
            unsafe {
                OUTPUT_BUFFER[pixel_index] = color;
            }
        }
    }
}
