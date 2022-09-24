#![allow(clippy::type_complexity)]

extern crate proc_macro;
extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use bevy::prelude::*;
use send_wrapper::SendWrapper;
pub use ultra_crustaceous::Color as UltraColor;
pub use ultra_crustaceous::{self};
use ultra_crustaceous::{OutputBuffer, PaletteBuffer};
use wasm_bindgen::prelude::*;

// everything has to be static state, but we hide that as best as we can from the user
static mut BEVY_APP: Option<SendWrapper<App>> = None;

pub use ultra_bevy_derive::init;

pub mod prelude {
    pub use crate::{UltraColor, UltraInput, UltraPlugin};
    pub use rastateur::PixelBuffer;
    pub use ultra_crustaceous::*;
}

#[wasm_bindgen]
pub fn get_screen_buffer_pointer() -> *const u8 {
    unsafe {
        BEVY_APP
        .as_ref()
        .unwrap()
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
        .as_ref()
        .unwrap()
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

pub struct UltraPlugin;

impl Plugin for UltraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutputBuffer>();
        app.init_resource::<PaletteBuffer>();
        app.init_resource::<UltraInput>();
    }
}

pub fn update_app(p1: u8, p2: u8, app_init: fn() -> App) {
    let p1 = ultra_crustaceous::Input::from_bits_truncate(p1);
    let p2 = ultra_crustaceous::Input::from_bits_truncate(p2);

    let app = unsafe { BEVY_APP.get_or_insert_with(|| SendWrapper::new(app_init())) };

    let mut ultra_input = app.world.get_resource_mut::<UltraInput>().expect(
        "Couldn't find output buffer resource in bevy app. Did you forget to add UltraPlugin?",
    );
    *ultra_input = UltraInput { p1, p2 };

    app.update();
}
