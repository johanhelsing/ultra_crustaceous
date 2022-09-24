# Rastateur

Simple stupid CPU drawing. Bring your own buffer type (pixel format agnostic).
Made for old-school pixel graphics.

Contains traits and extension methods for drawing on the CPU.

## Motivation

I wanted drawing functions for a buffer using 1 byte (u8) per pixel, and
couldn't find anything that didn't require 4 bytes per pixel.

I wanted something simple stupid that I could just use with my existing buffers.

## Out-of-scope

- Anti-aliasing (I'm using this for pixel graphics. At least it would have to be
opt-in)
- GPU-acceleration

## Basic usage

```rust
struct MyBuffer(Vec<u8>);

impl Rastateur
```