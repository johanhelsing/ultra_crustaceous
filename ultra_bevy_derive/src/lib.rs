#![allow(clippy::type_complexity)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, ItemFn};

extern crate proc_macro;

#[proc_macro_attribute]
pub fn init(_attr: TokenStream, app_init_function: TokenStream) -> TokenStream {
    // "fn answer() -> u32 { 42 }".parse().unwrap()
    let input = parse_macro_input!(app_init_function as ItemFn);
    let init_app_fn = input.sig.ident.clone();
    let output = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn update(p1: u8, p2: u8) {
            ultra_bevy::update_app(p1, p2, || #init_app_fn());
        }
        #input
    };
    output.into()
    // app_init_function
    // "fn answer() -> u32 { 42 }".parse().unwrap()
}
