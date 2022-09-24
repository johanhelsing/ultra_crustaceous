#![allow(clippy::type_complexity)]

use bevy::math::ivec2;
use bevy::prelude::*;
use bevy_system_graph::SystemGraph;
use derive_more::From;
use iyes_loopless::prelude::{ConditionHelpers, IntoConditionalSystem};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use ultra_bevy::prelude::*;

const SCREEN_SIZE: IVec2 = IVec2::new(ScreenBuffer::WIDTH as i32, ScreenBuffer::HEIGHT as i32);

const TILE_SIZE: usize = 10;
const MAP_SIZE: IVec2 = ivec2(25, 20);
const MAP_POS: IVec2 = IVec2::new(
    SCREEN_SIZE.x / 2 - MAP_SIZE.x * TILE_SIZE as i32 / 2,
    SCREEN_SIZE.y / 2 - MAP_SIZE.y * TILE_SIZE as i32 / 2,
);

const TICKS_PER_STEP: usize = 5;

#[derive(Component, Deref, DerefMut, Clone, Copy, From, PartialEq, Eq)]
struct TilePos(IVec2);

impl TilePos {
    pub fn to_screen_pos(self) -> IVec2 {
        MAP_POS + self.0 * TILE_SIZE as i32
    }

    pub fn draw_filled(&self, buffer: &mut ScreenBuffer, color: u8) {
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

const BORDER_COLOR: u8 = 18;
const MAP_COLOR: u8 = 0;
const WORM_COLOR: u8 = 16;
const APPLE_COLOR: u8 = 3;
const APPLE_STEM_COLOR: u8 = 19;
const DEAD_WORM_COLOR: u8 = 14;

fn setup(
    mut commands: Commands,
    mut palette: ResMut<PaletteBuffer>,
    mut screen: ResMut<ScreenBuffer>,
) {
    // https://lospec.com/palette-list/autumn-glow
    const AUTUMN_GLOW: [u32; 20] = [
        0xffffe1, 0xffd8a9, 0xffb366, 0xff5b4f, 0xf2af92, 0xf39d91, 0xd38e84, 0xc37289, 0xad82cf,
        0x8455a9, 0x794d81, 0x4a3778, 0xa9548a, 0x814d6e, 0xc92e70, 0x9e2081, 0x7e9770, 0x5d7668,
        0x235a63, 0x533a44,
    ];

    for (i, color) in AUTUMN_GLOW.into_iter().enumerate() {
        palette[i] = UltraColor::from_rgb(color);
    }

    screen.clear(BORDER_COLOR);

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

fn draw_background(mut screen: ResMut<ScreenBuffer>) {
    screen.draw_rect(MAP_POS, MAP_SIZE * TILE_SIZE as i32, MAP_COLOR);
}

fn draw_apples(apples: Query<&TilePos, With<Apple>>, mut screen: ResMut<ScreenBuffer>) {
    for tile in apples.iter() {
        let pos = tile.to_screen_pos();

        screen.draw_circle(
            pos.as_vec2() + Vec2::splat(TILE_SIZE as f32 / 2.),
            4.,
            APPLE_COLOR,
        );

        screen.draw_rect(
            pos + ivec2((TILE_SIZE / 2) as i32 - 1, TILE_SIZE as i32 - 2),
            ivec2(2, 2),
            APPLE_STEM_COLOR,
        );
    }
}

fn draw_snake(
    state: Res<State>,
    heads: Query<&TilePos, Or<(With<SnakeBody>, With<SnakeHead>)>>,
    mut screen: ResMut<ScreenBuffer>,
) {
    let color = match *state {
        State::GameOver => DEAD_WORM_COLOR,
        _ => WORM_COLOR,
    };

    for tile in heads.iter() {
        tile.draw_filled(screen.as_mut(), color);
    }
}
