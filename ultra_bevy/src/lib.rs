extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use bevy::math::ivec2;
use bevy::prelude::*;
use bevy_system_graph::SystemGraph;
use derive_more::From;
use iyes_loopless::prelude::IntoConditionalSystem;
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

#[derive(Component, Deref, DerefMut, Clone, Copy, From)]
struct TilePos(IVec2);

#[derive(Component, Deref, DerefMut, Clone, Copy, From)]
struct Direction(IVec2);

#[derive(Component)]
struct SnakeHead;

#[derive(Default, Deref, DerefMut)]
struct Tick(usize);

fn init() -> App {
    let mut app = App::new();

    app.add_plugin(UltraPlugin)
        .init_resource::<Tick>()
        .add_startup_system(setup);

    let graph = SystemGraph::new();

    // todo: clean up when stageless is implemented
    graph
        .root(tick)
        .then(update_head_direction)
        .then(move_snake.run_if(on_step))
        .then(draw_background.run_if(on_step))
        .then(draw_snake_head.run_if(on_step));

    app.add_system_set(graph.into());

    app
}

const TILE_SIZE: usize = 10;
const MAP_SIZE: IVec2 = ivec2(25, 20);
const TICKS_PER_STEP: usize = 5;

fn setup(mut commands: Commands, mut palette: ResMut<PaletteBuffer>) {
    palette[0] = UltraColor::from_rgb(0x302c2e); // dark brown
    palette[1] = UltraColor::from_rgb(0x7d7071); // light brown
    palette[2] = UltraColor::from_rgb(0x7d7071); // light brown
    palette[3] = UltraColor::from_rgb(0x71aa34); // green
    palette[4] = UltraColor::from_rgb(0xa93b3b); // deep red

    commands.spawn_bundle((TilePos(MAP_SIZE / 2), Direction(IVec2::ZERO), SnakeHead));
}

fn tick(mut tick: ResMut<Tick>) {
    **tick += 1;
}

fn on_step(tick: Res<Tick>) -> bool {
    **tick % TICKS_PER_STEP == 0
}

fn update_head_direction(
    mut heads: Query<&mut Direction, With<SnakeHead>>,
    input: Res<UltraInput>,
) {
    // combine input
    let input = input.p1 | input.p2;

    let input_dir = IVec2::new(input.x() as i32, input.y() as i32);

    // check for diagonal or no movement
    if input_dir.x.abs() + input_dir.y.abs() == 1 {
        for mut dir in heads.iter_mut() {
            // no immediate 180 turns
            if input_dir != -**dir {
                *dir = input_dir.into();
            }
        }
    }
}

fn move_snake(mut heads: Query<(&mut TilePos, &Direction), With<SnakeHead>>) {
    for (mut pos, dir) in heads.iter_mut() {
        **pos += **dir;
    }
}

fn draw_snake_head(heads: Query<&TilePos, With<SnakeHead>>, mut screen: ResMut<OutputBuffer>) {
    for pos in heads.iter() {
        draw_tile(&mut *screen, *pos, 3);
    }
}

fn draw_background(mut screen: ResMut<OutputBuffer>) {
    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            draw_tile(&mut *screen, TilePos(ivec2(x, y)), 1);
        }
    }
}

// struct SnakeGame {
//     ticks: usize,
//     food: Option<IVec2>,
//     direction: IVec2,
//     sleep: u8,
//     speed: u8,
//     rng: Option<SmallRng>,
// }

// // impl Default for SnakeGame {
// //     fn default() -> Self {
// //         let start_pos = IVec2::new(MAP_SIZE.x / 2, MAP_SIZE.y / 2);

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

// impl SnakeGame {
//     fn update(&mut self, p1: Input, p2: Input) {
//         self.ticks += 1;

//         let input = p1.union(p2); // let either joystick control

//         let input_dir = IVec2::new(input.x() as i32, input.y() as i32);

//         if input_dir.x.abs() + input_dir.y.abs() == 1 && input_dir != -self.direction {
//             // no diagonal or none movement, also no 180 turns
//             self.direction = input_dir
//         }

//         if self.sleep > 0 {
//             self.sleep -= 1;
//             return;
//         }

//         // move snake
//         if self.direction != IVec2::ZERO {
//             let head = *self.snake.front().unwrap();

//             let new_head_pos = head + self.direction;
//             if new_head_pos.x >= 0
//                 && new_head_pos.x < MAP_SIZE.x
//                 && new_head_pos.y >= 0
//                 && new_head_pos.y < MAP_SIZE.y
//                 && !self.snake.iter().skip(1).any(|p| p == &new_head_pos)
//             {
//                 self.snake.push_front(new_head_pos);
//                 self.snake.pop_back();
//                 self.sleep = self.speed;

//                 if let Some(food) = &self.food {
//                     if self.snake.front().unwrap() == food {
//                         self.food = None;
//                         self.snake.push_back(*self.snake.back().unwrap());
//                     }
//                 }
//             }

//             // currently, we're getting entropy from the number of ticks
//             // passed before we get the first input.
//             // that works ok for snake
//             let rng = self
//                 .rng
//                 .get_or_insert_with(|| SmallRng::seed_from_u64(self.ticks as u64));

//             if self.food.is_none() {
//                 self.food = Some(ivec2(
//                     rng.gen_range(0..MAP_SIZE.x),
//                     rng.gen_range(0..MAP_SIZE.y),
//                 ));
//             }
//         }

//         // clear entire screen
//         for i in 0..OUTPUT_BUFFER_SIZE {
//             // let y = i / SCREEN_WIDTH;
//             // self.output_buffer[i] = if y > 120 { 1 } else { 0 };
//             self.output_buffer[i] = 0;
//         }

//         // draw board
//         for x in 0..MAP_SIZE.x {
//             for y in 0..MAP_SIZE.y {
//                 draw_tile(&mut self.output_buffer, ivec2(x, y), 1);
//             }
//         }

//         // draw food
//         for tile_pos in self.food.iter().cloned() {
//             draw_tile(&mut self.output_buffer, tile_pos, 4);
//         }
//     }
// }

const SCREEN_SIZE: IVec2 = IVec2::new(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);

const MAP_POS: IVec2 = IVec2::new(
    SCREEN_SIZE.x / 2 - MAP_SIZE.x * TILE_SIZE as i32 / 2,
    SCREEN_SIZE.y / 2 - MAP_SIZE.y * TILE_SIZE as i32 / 2,
);

fn draw_tile(buffer: &mut OutputBuffer, tile: TilePos, color: u8) {
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
