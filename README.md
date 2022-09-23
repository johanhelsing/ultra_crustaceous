# Ultra Crustaceous

This repo contains open source sample games for the *Ultra Zeus* fantasy console
emulator written in rust.

It also contains a sample emulator written in javascript.

## Ultra Zeus spec

Ultra Zeus is a made up spec and interface for running old school games in an
ultra-portable and performant way.

It's built on top of web assembly modules, which means its games run extremely
fast almost anywhere, including web browsers, Unity, C, C#, Rust etc.

On most platforms, a basic emulator can be written in roughly 100 lines of code.

An Ultra Zeus game, is simply a web assembly module that exports three functions:

- `void update(i32 player_1, i32 player_2)`. Called once per frame by the
emulator with the latest player input. This is where you should update your game
state.
- `i32 get_screen_buffer_pointer()`. returns a pointer to where in the module's
memory the screen buffer is.
- (optional) `i32 get_palette_buffer_pointer()`. returns a pointer to where in
the module's memory the current palette buffer is. If this is not exported, the
default palette is used instead.

This means Ultra Zeus games can be *really* tiny, they are simply byte code with
no extra libraries. The snake example is currently 29k.

That means you can even fit some roms in [a regular link](http://localhost:8000/index.html?url=data:application/wasm;base64,AGFzbQEAAAABCgJgAAF/YAJ/fwADBAMBAAAFAwEAEgdMBAZtZW1vcnkCABlnZXRfc2NyZWVuX2J1ZmZlcl9wb2ludGVyAAEaZ2V0X3BhbGV0dGVfYnVmZmVyX3BvaW50ZXIAAgZ1cGRhdGUAAAqJAQN3AQR/QQAhAUGAgMAAIQIDQCABQQpuIQNBACEAA0AgACACaiIEQQFqIAMgAEH//wNxQQpuc0F/c0EBcSIFOgAAIAQgBToAACAAQQJqIgBBwAJHDQALIAJBwAJqIQIgAUEBaiIBQfABRw0AC0GA2MQAQYCMGDYAAAsHAEGAgMAACwcAQYDYxAALAG8JcHJvZHVjZXJzAghsYW5ndWFnZQEEUnVzdAAMcHJvY2Vzc2VkLWJ5AwVydXN0Yx0xLjYzLjAgKDRiOTFhNmVhNyAyMDIyLTA4LTA4KQZ3YWxydXMGMC4xOS4wDHdhc20tYmluZGdlbgYwLjIuODI=)
It also means you can write games in any language that can target web assembly.

### Graphics

Ultra Zeus graphics are inspired by 1990s home computers with low resolution
pixel graphics using a customizable palette.

- The resolution is 320x240 (4:3)
- The screen buffer is 320x240 = 76,800 bytes long
- Each byte in the buffer is a palette index
- Up to 32 palette colors are supported
- Each palette color consist of 2 bytes.
    - These bytes specifies red, blue and green components, with 4 bits per channel
    - The bit arrangement is `(0000rrrr, ggggbbbb)`
    - This means up to 2^12 = 4096 different colors are possible
    - The 4 most significant bits of the red byte are reserved
- This means the palette buffer is 32 * 2 = 64 bytes long

See the `checker_palette` example for a minimal example rom using palettes.

### Audio

TODO

### Metadata

TODO

### Sample games

See the `ultra_snake` folder for a complete example using rust to implement a
complete game with input and graphics.

The samples can be built by running `cargo xtask dist <game_name>`.

### Emulators

An example/reference emulator written for web is included in `index.html`.

It uses simple vanilla js with no extra dependencies.