extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use bevy::prelude::*;
use once_cell::sync::Lazy;
use send_wrapper::SendWrapper;
use ultra_crustaceous::Color as UltraColor;
use ultra_crustaceous::*;
use wasm_bindgen::prelude::*;

static mut BEVY_APP: Lazy<SendWrapper<App>> = Lazy::new(|| SendWrapper::new(init()));

#[wasm_bindgen]
pub fn get_screen_buffer_pointer() -> *const u8 {
    unsafe {
        BEVY_APP
        .world
        .get_resource::<OutputBuffer>()
        .expect(
            "Couldn't find output buffer resource in bevy app. Did you forget to add UltraPlugin?",
        )
        .as_ptr()
    }
}

#[wasm_bindgen]
pub fn get_palette_buffer_pointer() -> *const u8 {
    unsafe {
        BEVY_APP
        .world
        .get_resource::<PaletteBuffer>()
        .expect(
            "Couldn't find output buffer resource in bevy app. Did you forget to add UltraPlugin?",
        )
        .as_ptr() as *const u8
    }
}

#[derive(Default)]
pub struct UltraInput {
    pub p1: ultra_crustaceous::Input,
    pub p2: ultra_crustaceous::Input,
}

struct UltraPlugin;

impl Plugin for UltraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutputBuffer>();
        app.init_resource::<PaletteBuffer>();
        app.init_resource::<UltraInput>();
    }
}

#[wasm_bindgen]
pub fn update(p1: u8, p2: u8) {
    let p1 = ultra_crustaceous::Input::from_bits_truncate(p1);
    let p2 = ultra_crustaceous::Input::from_bits_truncate(p2);

    // inject input
    unsafe {
        let mut ultra_input = BEVY_APP.world.get_resource_mut::<UltraInput>().expect(
            "Couldn't find output buffer resource in bevy app. Did you forget to add UltraPlugin?",
        );
        *ultra_input = UltraInput { p1, p2 };
    }

    unsafe {
        BEVY_APP.update();
    }
}

// above is boiler-plate

fn init() -> App {
    let mut app = App::new();

    app.add_plugin(UltraPlugin)
        .add_startup_system(setup_checkers);

    app
}

fn setup_checkers(mut output: ResMut<OutputBuffer>, mut palette: ResMut<PaletteBuffer>) {
    const TILE_SIZE: usize = 10;
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let is_dark_square = (y / TILE_SIZE) % 2 != (x / TILE_SIZE) % 2;
            let color = if is_dark_square { 0 } else { 1 };
            let pixel_index = y * SCREEN_WIDTH + x;
            output[pixel_index] = color;
        }
    }

    palette[0] = UltraColor::from_rgb(0xff0000);
    palette[1] = UltraColor::from_rgb(0x00ff00);
    palette[2] = UltraColor::from_rgb(0x0000ff);
}

// // const TILE_SIZE: usize = 10;
// // const MAP_SIZE: IVec2 = IVec2::new(25, 20);

// // struct SnakeGame {
// //     ticks: usize,
// //     // output_buffer: OutputBuffer,
// //     // palette: PaletteBuffer,
// //     snake: VecDeque<IVec2>,
// //     food: Option<IVec2>,
// //     direction: IVec2,
// //     sleep: u8,
// //     speed: u8,
// //     rng: Option<SmallRng>,
// // }

// // impl Default for SnakeGame {
// //     fn default() -> Self {
// //         let start_pos = IVec2::new(MAP_SIZE.x / 2, MAP_SIZE.y / 2);
// //         let mut palette = PaletteBuffer::default();
// //         palette[0] = Color::from_rgb(0x302c2e); // dark brown
// //         palette[1] = Color::from_rgb(0x7d7071); // light brown
// //         palette[2] = Color::from_rgb(0x7d7071); // light brown
// //         palette[3] = Color::from_rgb(0x71aa34); // green
// //         palette[4] = Color::from_rgb(0xa93b3b); // deep red

// //         Self {
// //             output_buffer: Default::default(),
// //             palette,
// //             snake: VecDeque::from_iter(iter::repeat(start_pos).take(5)),
// //             direction: IVec2::ZERO, // start stationary
// //             speed: 5,
// //             sleep: 0,
// //             ticks: 0,
// //             food: None,
// //             rng: None,
// //         }
// //     }
// // }

// // impl SnakeGame {
// //     fn update(&mut self, p1: Input, p2: Input) {
// //         self.ticks += 1;

// //         let input = p1.union(p2); // let either joystick control

// //         let input_dir = IVec2::new(input.x() as i32, input.y() as i32);

// //         if input_dir.x.abs() + input_dir.y.abs() == 1 && input_dir != -self.direction {
// //             // no diagonal or none movement, also no 180 turns
// //             self.direction = input_dir
// //         }

// //         if self.sleep > 0 {
// //             self.sleep -= 1;
// //             return;
// //         }

// //         // move snake
// //         if self.direction != IVec2::ZERO {
// //             let head = *self.snake.front().unwrap();

// //             let new_head_pos = head + self.direction;
// //             if new_head_pos.x >= 0
// //                 && new_head_pos.x < MAP_SIZE.x
// //                 && new_head_pos.y >= 0
// //                 && new_head_pos.y < MAP_SIZE.y
// //                 && !self.snake.iter().skip(1).any(|p| p == &new_head_pos)
// //             {
// //                 self.snake.push_front(new_head_pos);
// //                 self.snake.pop_back();
// //                 self.sleep = self.speed;

// //                 if let Some(food) = &self.food {
// //                     if self.snake.front().unwrap() == food {
// //                         self.food = None;
// //                         self.snake.push_back(*self.snake.back().unwrap());
// //                     }
// //                 }
// //             }

// //             // currently, we're getting entropy from the number of ticks
// //             // passed before we get the first input.
// //             // that works ok for snake
// //             let rng = self
// //                 .rng
// //                 .get_or_insert_with(|| SmallRng::seed_from_u64(self.ticks as u64));

// //             if self.food.is_none() {
// //                 self.food = Some(ivec2(
// //                     rng.gen_range(0..MAP_SIZE.x),
// //                     rng.gen_range(0..MAP_SIZE.y),
// //                 ));
// //             }
// //         }

// //         // clear entire screen
// //         for i in 0..OUTPUT_BUFFER_SIZE {
// //             // let y = i / SCREEN_WIDTH;
// //             // self.output_buffer[i] = if y > 120 { 1 } else { 0 };
// //             self.output_buffer[i] = 0;
// //         }

// //         // draw board
// //         for x in 0..MAP_SIZE.x {
// //             for y in 0..MAP_SIZE.y {
// //                 draw_tile(&mut self.output_buffer, ivec2(x, y), 1);
// //             }
// //         }

// //         // draw snake
// //         for tile_pos in self.snake.iter().cloned() {
// //             draw_tile(&mut self.output_buffer, tile_pos, 3);
// //         }

// //         // draw food
// //         for tile_pos in self.food.iter().cloned() {
// //             draw_tile(&mut self.output_buffer, tile_pos, 4);
// //         }
// //     }
// // }

// // const SCREEN_SIZE: IVec2 = IVec2::new(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
// // const MAP_POS: IVec2 = IVec2::new(
// //     SCREEN_SIZE.x / 2 - MAP_SIZE.x * TILE_SIZE as i32 / 2,
// //     SCREEN_SIZE.y / 2 - MAP_SIZE.y * TILE_SIZE as i32 / 2,
// // );

// // fn draw_tile(buffer: &mut OutputBuffer, tile: IVec2, color: u8) {
// //     let start_x = MAP_POS.x as usize + tile.x as usize * TILE_SIZE;
// //     let start_y = MAP_POS.y as usize + tile.y as usize * TILE_SIZE;
// //     let start = start_x + start_y * SCREEN_WIDTH;

// //     for y in 0..TILE_SIZE {
// //         for x in 0..TILE_SIZE {
// //             let i = start + x + y * SCREEN_WIDTH;
// //             if i < buffer.0.len() {
// //                 buffer[i] = color;
// //             }
// //         }
// //     }
// // }
