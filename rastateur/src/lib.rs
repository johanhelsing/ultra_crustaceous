#![no_std]

// todo: could perhaps relax trait bound on Copy
/// Implement this trait enable drawing methods on your buffer
///
/// Note: You only need to implement the four basic methods (`get_pixel`,
/// `set_pixel`, `width` and `height`) to be able to draw complex shapes,
/// everything else has default implementations.
pub trait PixelBuffer<TPixel: Clone + Copy> {
    /// Set the pixel at the given position to a specific color
    ///
    /// It's up to the implementor how to store the pixels, but the drawing
    /// algorithms are optimized for contiguous x values
    fn set_pixel(&mut self, pos: (usize, usize), color: TPixel);

    /// Get a pixel
    fn get_pixel(&self, pos: (usize, usize)) -> TPixel;

    fn width(&self) -> usize;

    fn height(&self) -> usize;

    // From here on down are default-implementations, can be overridden, if you
    // want to optimize in some way for instance.

    // todo: could perhaps move these to extension traits instead?

    #[inline]
    fn draw_rect(
        &mut self,
        (x, y): (usize, usize),
        (width, height): (usize, usize),
        color: TPixel,
    ) {
        // limit to screen
        let x_max = (x + width).min(self.width());
        let y_max = (y + height).min(self.height());

        for y in y..y_max {
            // inner loop on x for efficient memory access
            for x in x..x_max {
                self.set_pixel((x, y), color);
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
            fn set_pixel(&mut self, (x, y): (usize, usize), value: u8) {
                self.0[y * HEIGHT + x] = value;
            }

            fn get_pixel(&self, (x, y): (usize, usize)) -> u8 {
                self.0[y * HEIGHT + x]
            }

            fn width(&self) -> usize {
                WIDTH
            }

            fn height(&self) -> usize {
                HEIGHT
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
            fn set_pixel(&mut self, (x, y): (usize, usize), value: [u8; 4]) {
                for i in 0..4 {
                    self.0[(y * HEIGHT + x) * 4 + i] = value[i];
                }
            }

            fn get_pixel(&self, (x, y): (usize, usize)) -> [u8; 4] {
                let i = (y * HEIGHT + x) * 4;
                self.0[i..i + 4].try_into().unwrap()
            }

            fn width(&self) -> usize {
                WIDTH
            }

            fn height(&self) -> usize {
                HEIGHT
            }
        }

        let mut buffer = Rgba32Buffer([0; WIDTH * HEIGHT * 4]);

        buffer.set_pixel((0, 0), [0xff, 0x00, 0xff, 0xff]);

        assert_eq!(buffer.get_pixel((0, 0)), [0xff, 0x00, 0xff, 0xff]);
    }
}
