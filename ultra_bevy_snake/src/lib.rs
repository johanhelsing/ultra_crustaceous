#![allow(clippy::type_complexity)]

use bevy::math::ivec2;
use bevy::prelude::*;
use bevy_system_graph::SystemGraph;
use derive_more::From;
use iyes_loopless::prelude::{ConditionHelpers, IntoConditionalSystem};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use ultra_bevy::prelude::*;

#[derive(Component, Deref, DerefMut, Clone, Copy, From, PartialEq, Eq)]
struct TilePos(IVec2);

impl TilePos {
    fn to_screen_pos(self) -> IVec2 {
        MAP_POS + self.0 * TILE_SIZE as i32
    }

    pub fn draw_filled(&self, buffer: &mut OutputBuffer, color: u8) {
        buffer.draw_rect(self.to_screen_pos(), IVec2::splat(TILE_SIZE as i32), color);
    }
}

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

#[ultra_bevy::init()]
fn init() -> App {
    let mut app = App::new();

    app.add_plugin(UltraPlugin)
        .init_resource::<Tick>()
        .init_resource::<Random>()
        .init_resource::<State>()
        .add_startup_system(setup);

    let graph = SystemGraph::new();

    // todo: clean up when bevy stageless rfc is implemented
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

const BORDER_COLOR: u8 = 0;
const MAP_COLOR: u8 = 1;
const WORM_COLOR: u8 = 2;
const APPLE_COLOR: u8 = 3;
const DEAD_WORM_COLOR: u8 = APPLE_COLOR;

fn setup(mut commands: Commands, mut palette: ResMut<PaletteBuffer>) {
    palette[BORDER_COLOR as usize] = UltraColor::from_rgb(0x302c2e); // dark brown
    palette[MAP_COLOR as usize] = UltraColor::from_rgb(0x7d7071); // light brown
    palette[WORM_COLOR as usize] = UltraColor::from_rgb(0x71aa34); // green
    palette[APPLE_COLOR as usize] = UltraColor::from_rgb(0xa93b3b); // deep red

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
    screen.draw_rect(MAP_POS, MAP_SIZE * TILE_SIZE as i32, MAP_COLOR);
}

fn draw_apples(apples: Query<&TilePos, With<Apple>>, mut screen: ResMut<OutputBuffer>) {
    for tile in apples.iter() {
        tile.draw_filled(screen.as_mut(), APPLE_COLOR);
    }
}

fn draw_snake(
    state: Res<State>,
    heads: Query<&TilePos, Or<(With<SnakeBody>, With<SnakeHead>)>>,
    mut screen: ResMut<OutputBuffer>,
) {
    let color = match *state {
        State::GameOver => DEAD_WORM_COLOR,
        _ => WORM_COLOR,
    };

    for tile in heads.iter() {
        tile.draw_filled(screen.as_mut(), color);
    }
}

const SCREEN_SIZE: IVec2 = IVec2::new(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);

const MAP_POS: IVec2 = IVec2::new(
    SCREEN_SIZE.x / 2 - MAP_SIZE.x * TILE_SIZE as i32 / 2,
    SCREEN_SIZE.y / 2 - MAP_SIZE.y * TILE_SIZE as i32 / 2,
);
