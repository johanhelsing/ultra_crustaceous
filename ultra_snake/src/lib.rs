extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use glam::{ivec2, IVec2};
use lazy_static::lazy_static;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::{collections::VecDeque, iter, sync::RwLock};
use ultra_crustaceous::{Color, Input, PaletteBuffer, ScreenBuffer};
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref GAME: RwLock<SnakeGame> = RwLock::new(SnakeGame::default());
}

#[wasm_bindgen]
pub fn get_screen_buffer_pointer() -> *const u8 {
    let game = GAME.read().expect("couldn't get game read lock");
    game.output_buffer.as_ptr()
}

#[wasm_bindgen]
pub fn get_palette_buffer_pointer() -> *const u8 {
    let game = GAME.read().expect("couldn't get game read lock");
    game.palette.as_ptr() as *const u8
}

#[wasm_bindgen]
pub fn update(p1: u8, p2: u8) {
    let p1 = Input::from_bits_truncate(p1);
    let p2 = Input::from_bits_truncate(p2);

    GAME.write()
        .expect("Failed to get game write lock")
        .update(p1, p2);
}

// above is boiler-plate

const TILE_SIZE: usize = 10;
const MAP_SIZE: IVec2 = IVec2::new(25, 20);

struct SnakeGame {
    ticks: usize,
    output_buffer: ScreenBuffer,
    palette: PaletteBuffer,
    snake: VecDeque<IVec2>,
    food: Option<IVec2>,
    direction: IVec2,
    sleep: u8,
    speed: u8,
    rng: Option<SmallRng>,
}

impl Default for SnakeGame {
    fn default() -> Self {
        let start_pos = IVec2::new(MAP_SIZE.x / 2, MAP_SIZE.y / 2);
        let mut palette = PaletteBuffer::default();
        palette[0] = Color::from_rgb(0x302c2e); // dark brown
        palette[1] = Color::from_rgb(0x7d7071); // light brown
        palette[2] = Color::from_rgb(0x7d7071); // light brown
        palette[3] = Color::from_rgb(0x71aa34); // green
        palette[4] = Color::from_rgb(0xa93b3b); // deep red

        Self {
            output_buffer: Default::default(),
            palette,
            snake: VecDeque::from_iter(iter::repeat(start_pos).take(5)),
            direction: IVec2::ZERO, // start stationary
            speed: 5,
            sleep: 0,
            ticks: 0,
            food: None,
            rng: None,
        }
    }
}

impl SnakeGame {
    fn update(&mut self, p1: Input, p2: Input) {
        self.ticks += 1;

        let input = p1.union(p2); // let either joystick control

        let input_dir = IVec2::new(input.x() as i32, input.y() as i32);

        if input_dir.x.abs() + input_dir.y.abs() == 1 && input_dir != -self.direction {
            // no diagonal or none movement, also no 180 turns
            self.direction = input_dir
        }

        if self.sleep > 0 {
            self.sleep -= 1;
            return;
        }

        // move snake
        if self.direction != IVec2::ZERO {
            let head = *self.snake.front().unwrap();

            let new_head_pos = head + self.direction;
            if new_head_pos.x >= 0
                && new_head_pos.x < MAP_SIZE.x
                && new_head_pos.y >= 0
                && new_head_pos.y < MAP_SIZE.y
                && !self.snake.iter().skip(1).any(|p| p == &new_head_pos)
            {
                self.snake.push_front(new_head_pos);
                self.snake.pop_back();
                self.sleep = self.speed;

                if let Some(food) = &self.food {
                    if self.snake.front().unwrap() == food {
                        self.food = None;
                        self.snake.push_back(*self.snake.back().unwrap());
                    }
                }
            }

            // currently, we're getting entropy from the number of ticks
            // passed before we get the first input.
            // that works ok for snake
            let rng = self
                .rng
                .get_or_insert_with(|| SmallRng::seed_from_u64(self.ticks as u64));

            if self.food.is_none() {
                self.food = Some(ivec2(
                    rng.gen_range(0..MAP_SIZE.x),
                    rng.gen_range(0..MAP_SIZE.y),
                ));
            }
        }

        // clear entire screen
        for i in 0..ScreenBuffer::NUM_PIXELS {
            self.output_buffer[i] = 0;
        }

        // draw board
        for x in 0..MAP_SIZE.x {
            for y in 0..MAP_SIZE.y {
                draw_tile(&mut self.output_buffer, ivec2(x, y), 1);
            }
        }

        // draw snake
        for tile_pos in self.snake.iter().cloned() {
            draw_tile(&mut self.output_buffer, tile_pos, 3);
        }

        // draw food
        for tile_pos in self.food.iter().cloned() {
            draw_tile(&mut self.output_buffer, tile_pos, 4);
        }
    }
}

const SCREEN_SIZE: IVec2 = IVec2::new(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
const MAP_POS: IVec2 = IVec2::new(
    SCREEN_SIZE.x / 2 - MAP_SIZE.x * TILE_SIZE as i32 / 2,
    SCREEN_SIZE.y / 2 - MAP_SIZE.y * TILE_SIZE as i32 / 2,
);

fn draw_tile(buffer: &mut ScreenBuffer, tile: IVec2, color: u8) {
    let start_x = MAP_POS.x as usize + tile.x as usize * TILE_SIZE;
    let start_y = MAP_POS.y as usize + tile.y as usize * TILE_SIZE;
    let start = start_x + start_y * SCREEN_WIDTH;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let i = start + x + y * SCREEN_WIDTH;
            if i < buffer.0.len() {
                buffer[i] = color;
            }
        }
    }
}
