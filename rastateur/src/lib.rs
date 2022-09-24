// #![no_std]

// todo: could perhaps relax trait bound on Copy
/// Implement this trait enable drawing methods on your buffer
///
/// Note: You only need to implement the four basic methods (`get_pixel`,
/// `set_pixel`, `width` and `height`) to be able to draw complex shapes,
/// everything else has default implementations.
pub trait PixelBuffer<TColor: Clone + Copy> {
    /// Set the pixel at the given position to a specific color
    ///
    /// It's up to the implementor how to store the pixels, but the drawing
    /// algorithms are optimized for contiguous x values
    fn set_pixel<TPos: Into<(i32, i32)>>(&mut self, pos: TPos, color: TColor);

    /// Get a pixel
    fn get_pixel<TPos: Into<(i32, i32)>>(&self, pos: TPos) -> TColor;

    fn width(&self) -> i32;

    fn height(&self) -> i32;

    // From here on down are default-implementations, can be overridden, if you
    // want to optimize in some way for instance.

    // todo: could perhaps move these to extension traits instead?

    fn clear(&mut self, color: TColor) {
        for y in 0..self.height() {
            // inner loop on x for efficient memory access
            for x in 0..self.width() {
                self.set_pixel((x, y), color);
            }
        }
    }

    fn draw_rect<TPos: Into<(i32, i32)>>(&mut self, pos: TPos, size: TPos, color: TColor) {
        // limit to screen
        let (x, y) = pos.into();
        let (width, height) = size.into();
        let x_max = (x + width).min(self.width());
        let y_max = (y + height).min(self.height());

        for y in y..y_max {
            // inner loop on x for efficient memory access
            for x in x..x_max {
                self.set_pixel((x, y), color);
            }
        }
    }

    #[inline]
    fn clamp_x(&self, x: i32) -> i32 {
        x.clamp(0, self.width())
    }

    #[inline]
    fn clamp_y(&self, y: i32) -> i32 {
        y.clamp(0, self.height())
    }

    // maybe use f32 for radius?
    fn draw_circle<TPos: Into<(f32, f32)>>(&mut self, center: TPos, radius: f32, color: TColor) {
        let (mut center_x, mut center_y) = center.into();
        center_x -= 0.5; // center it inside the pixel
        center_y -= 0.5;

        // limit to screen
        let y_max = self.clamp_y((center_y + radius).ceil() as i32);
        let y_min = self.clamp_y((center_y - radius) as i32);

        let x_max = self.clamp_x((center_x + radius).ceil() as i32);
        let x_min = self.clamp_y((center_x - radius) as i32);

        let r_squared = radius * radius;

        for y in y_min..=y_max {
            let j_squared = (y as f32 - center_y).powi(2);

            // inner loop on x for efficient memory access
            for x in x_min..x_max {
                let i = x as f32 - center_x;

                // todo: move check one step out
                if (i * i + j_squared) < r_squared {
                    self.set_pixel((x, y), color);
                }
            }
        }
    }

    fn draw_rect_with<TPos: Into<(i32, i32)>, TArg: From<(i32, i32)>>(
        &mut self,
        pos: TPos,
        size: TPos,
        color_fn: fn(TArg) -> TColor,
    ) {
        // limit to screen
        let (x, y) = pos.into();
        let (width, height) = size.into();
        let x_max = (x + width).min(self.width());
        let y_max = (y + height).min(self.height());

        for y in y..y_max {
            // inner loop on x for efficient memory access
            for x in x..x_max {
                let pos = (x, y);
                self.set_pixel(pos, color_fn(pos.into()));
            }
        }
    }

    fn modify_rect_with<TPos: Into<(i32, i32)>, TArg: From<(i32, i32)>>(
        &mut self,
        pos: TPos,
        size: TPos,
        mutator: fn(TArg, TColor) -> TColor,
    ) {
        // limit to screen
        let (x, y) = pos.into();
        let (width, height) = size.into();
        let x_max = (x + width).min(self.width());
        let y_max = (y + height).min(self.height());

        for y in y..y_max {
            // inner loop on x for efficient memory access
            for x in x..x_max {
                let pos = (x, y);
                // perf: slightly inefficient (extra copies and writes)
                let current_color = self.get_pixel(pos);
                self.set_pixel(pos, mutator(pos.into(), current_color));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_usage() {
        const WIDTH: usize = 320;
        const HEIGHT: usize = 240;

        // use any pixel format you like
        struct MyBuffer(pub [u8; WIDTH * HEIGHT]);

        impl PixelBuffer<u8> for MyBuffer {
            fn set_pixel<T: Into<(i32, i32)>>(&mut self, pos: T, value: u8) {
                let (x, y) = pos.into();
                self.0[y as usize * HEIGHT + x as usize] = value;
            }

            fn get_pixel<T: Into<(i32, i32)>>(&self, pos: T) -> u8 {
                let (x, y) = pos.into();
                self.0[y as usize * HEIGHT + x as usize]
            }

            fn width(&self) -> i32 {
                WIDTH as i32
            }

            fn height(&self) -> i32 {
                HEIGHT as i32
            }
        }

        let mut buffer = MyBuffer([0; WIDTH * HEIGHT]);

        buffer.set_pixel((0, 0), 123);

        assert_eq!(buffer.get_pixel((0, 0)), 123);
        assert_eq!(buffer.get_pixel((1, 1)), 0);
    }

    #[test]
    fn rgba_buffer() {
        const WIDTH: usize = 320;
        const HEIGHT: usize = 240;

        // contiguous bytes
        struct Rgba32Buffer(pub [u8; WIDTH * HEIGHT * 4]);

        impl PixelBuffer<[u8; 4]> for Rgba32Buffer {
            fn set_pixel<T: Into<(i32, i32)>>(&mut self, pos: T, color: [u8; 4]) {
                let (x, y) = pos.into();
                let i = (y as usize * HEIGHT + x as usize) * 4;
                let dst = &mut self.0[i..i + 4];
                dst.copy_from_slice(&color);
            }

            fn get_pixel<T: Into<(i32, i32)>>(&self, pos: T) -> [u8; 4] {
                let (x, y) = pos.into();
                let i = (y as usize * HEIGHT + x as usize) * 4;
                self.0[i..i + 4].try_into().unwrap()
            }

            fn width(&self) -> i32 {
                WIDTH as i32
            }

            fn height(&self) -> i32 {
                HEIGHT as i32
            }
        }

        let mut buffer = Rgba32Buffer([0; WIDTH * HEIGHT * 4]);

        buffer.set_pixel((0, 0), [0xff, 0x00, 0xff, 0xff]);

        assert_eq!(buffer.get_pixel((0, 0)), [0xff, 0x00, 0xff, 0xff]);
    }
}
