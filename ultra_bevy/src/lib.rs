#![allow(clippy::type_complexity)]

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use bevy::math::ivec2;
use bevy::prelude::*;
use bevy_system_graph::SystemGraph;
use derive_more::From;
use iyes_loopless::prelude::{ConditionHelpers, IntoConditionalSystem};
use once_cell::sync::Lazy;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
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

#[derive(Component, Deref, DerefMut, Clone, Copy, From, PartialEq, Eq)]
struct TilePos(IVec2);

#[derive(Component, Deref, DerefMut, Clone, Copy, From)]
struct Direction(IVec2);

#[derive(Component)]
struct SnakeHead;

#[derive(Component)]
struct SnakeBody(Entity);

#[derive(Deref, DerefMut)]
struct Tail(Entity);

#[derive(Component)]
struct Apple;

#[derive(Default, Deref, DerefMut, Clone, Copy)]
struct Tick(usize);

#[derive(Default)]
struct Random(Option<SmallRng>);

impl From<Tick> for SmallRng {
    fn from(tick: Tick) -> Self {
        SmallRng::seed_from_u64(*tick as u64)
    }
}

#[derive(PartialEq, Eq)]
enum State {
    Running,
    GameOver,
}

impl Default for State {
    fn default() -> Self {
        Self::Running
    }
}

fn init() -> App {
    let mut app = App::new();

    app.add_plugin(UltraPlugin)
        .init_resource::<Tick>()
        .init_resource::<Random>()
        .init_resource::<State>()
        .add_startup_system(setup);

    let graph = SystemGraph::new();

    // todo: clean up when stageless is implemented
    graph
        .root(tick)
        .then(update_head_dir.run_if(alive))
        .then(move_head.run_if(on_step).run_if(alive))
        .then(move_body.run_if(on_step).run_if(alive))
        .then(eat_apples.run_if(on_step).run_if(alive))
        .then(update_body_dir.run_if(on_step).run_if(alive))
        .then(draw_background.run_if(on_step))
        .then(draw_apples.run_if(on_step))
        .then(draw_snake.run_if(on_step));

    app.add_system_set(graph.into());

    app
}

const TILE_SIZE: usize = 10;
const MAP_SIZE: IVec2 = ivec2(25, 20);
const TICKS_PER_STEP: usize = 5;

#[derive(Bundle)]
struct SnakeBodyBundle {
    body: SnakeBody,
    pos: TilePos,
    dir: Direction,
}

impl SnakeBodyBundle {
    pub fn new(pos: TilePos, next: Entity) -> Self {
        Self {
            body: SnakeBody(next),
            pos,
            dir: Direction(IVec2::ZERO),
        }
    }
}

fn setup(mut commands: Commands, mut palette: ResMut<PaletteBuffer>) {
    // TODO: convert from bevy colors?
    palette[0] = UltraColor::from_rgb(0x302c2e); // dark brown
    palette[1] = UltraColor::from_rgb(0x7d7071); // light brown
    palette[2] = UltraColor::from_rgb(0x7d7071); // light brown
    palette[3] = UltraColor::from_rgb(0x71aa34); // green
    palette[4] = UltraColor::from_rgb(0xa93b3b); // deep red

    let start_pos = TilePos(MAP_SIZE / 2);

    let mut tail = commands
        .spawn_bundle((SnakeHead, start_pos, Direction(IVec2::ZERO)))
        .id();

    for _ in 0..5 {
        tail = commands
            .spawn_bundle(SnakeBodyBundle::new(start_pos, tail))
            .id();
    }

    commands.insert_resource(Tail(tail));

    commands.spawn_bundle((Apple, TilePos(ivec2(2, 2))));
}

fn tick(mut tick: ResMut<Tick>) {
    **tick += 1;
}

fn on_step(tick: Res<Tick>) -> bool {
    **tick % TICKS_PER_STEP == 0
}

fn alive(state: Res<State>) -> bool {
    *state == State::Running
}

fn update_head_dir(mut heads: Query<&mut Direction, With<SnakeHead>>, input: Res<UltraInput>) {
    // combine input
    let input = input.p1 | input.p2;

    let input_dir = ivec2(input.x() as i32, input.y() as i32);

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

// combine with move_body?
fn move_head(
    mut state: ResMut<State>,
    mut heads: Query<(&mut TilePos, &Direction), With<SnakeHead>>,
    body_parts: Query<&TilePos, (With<SnakeBody>, Without<SnakeHead>)>,
) {
    for (mut pos, dir) in heads.iter_mut() {
        if **dir == IVec2::ZERO {
            continue;
        }

        let new_pos = **pos + **dir;

        let dead = new_pos.x < 0
            || new_pos.y < 0
            || new_pos.x >= MAP_SIZE.x
            || new_pos.y >= MAP_SIZE.y
            || body_parts.iter().any(|body_pos| **body_pos == new_pos);

        if dead {
            *state = State::GameOver;
        } else {
            **pos = new_pos;
        }
    }
}

fn move_body(mut parts: Query<(&mut TilePos, &Direction), With<SnakeBody>>) {
    for (mut pos, dir) in parts.iter_mut() {
        **pos += **dir;
    }
}

fn eat_apples(
    mut commands: Commands,
    heads: Query<&TilePos, (With<SnakeHead>, Without<Apple>)>,
    body_positions: Query<&TilePos, Without<Apple>>,
    mut apples: Query<&mut TilePos, With<Apple>>,
    mut tail: ResMut<Tail>,
    mut rng: ResMut<Random>,
    tick: Res<Tick>,
) {
    for head_pos in heads.iter() {
        for mut apple_pos in apples.iter_mut() {
            if *apple_pos == *head_pos {
                let tail_pos = body_positions.get(**tail).unwrap();

                // spawn extra segments for worm on the tail
                for _ in 0..5 {
                    **tail = commands
                        .spawn_bundle(SnakeBodyBundle::new(*tail_pos, **tail))
                        .id();
                }

                // move the apple
                let rng = rng.0.get_or_insert_with(|| SmallRng::from(*tick));
                loop {
                    **apple_pos = ivec2(rng.gen_range(0..MAP_SIZE.x), rng.gen_range(0..MAP_SIZE.y));
                    // Make sure we don't spawn inside snake
                    if body_positions.iter().all(|p| *p != *apple_pos) {
                        break;
                    }
                }
            }
        }
    }
}

fn update_body_dir(
    mut parts: Query<(&mut Direction, &TilePos, &SnakeBody)>,
    positions: Query<&TilePos>,
) {
    for (mut dir, pos, body) in parts.iter_mut() {
        let next = body.0;
        let next_pos = positions.get(next).unwrap();
        **dir = **next_pos - **pos;
    }
}

fn draw_background(mut screen: ResMut<OutputBuffer>) {
    for x in 0..MAP_SIZE.x {
        for y in 0..MAP_SIZE.y {
            draw_tile(&mut *screen, TilePos(ivec2(x, y)), 1);
        }
    }
}

fn draw_apples(apples: Query<&TilePos, With<Apple>>, mut screen: ResMut<OutputBuffer>) {
    for pos in apples.iter() {
        draw_tile(&mut *screen, *pos, 4);
    }
}

fn draw_snake(
    state: Res<State>,
    heads: Query<&TilePos, Or<(With<SnakeBody>, With<SnakeHead>)>>,
    mut screen: ResMut<OutputBuffer>,
) {
    let color = match *state {
        State::GameOver => 4,
        _ => 3,
    };

    for pos in heads.iter() {
        draw_tile(&mut *screen, *pos, color);
    }
}

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
