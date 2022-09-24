pub trait PixelBuffer<TPixel> {
    /// It's up to the implementor how to store the pixels, but the drawing
    /// algorithms are optimized for contiguous x values
    fn set_pixel(&mut self, x: usize, y: usize, value: TPixel);
    fn get_pixel(&self, x: usize, y: usize) -> TPixel;
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
            fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
                self.0[y * HEIGHT + x] = value;
            }

            fn get_pixel(&self, x: usize, y: usize) -> u8 {
                self.0[y * HEIGHT + x]
            }
        }

        let mut buffer = MyBuffer([0; WIDTH * HEIGHT]);

        buffer.set_pixel(0, 0, 123);

        assert_eq!(buffer.get_pixel(0, 0), 123);
        assert_eq!(buffer.get_pixel(1, 1), 0);
    }

    #[test]
    fn rgba_buffer() {
        const WIDTH: usize = 320;
        const HEIGHT: usize = 240;

        // contiguous bytes
        struct Rgba32Buffer(pub [u8; WIDTH * HEIGHT * 4]);

        impl PixelBuffer<[u8; 4]> for Rgba32Buffer {
            fn set_pixel(&mut self, x: usize, y: usize, value: [u8; 4]) {
                for i in 0..4 {
                    self.0[(y * HEIGHT + x) * 4 + i] = value[i];
                }
            }

            fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
                let i = (y * HEIGHT + x) * 4;
                self.0[i..i + 4].try_into().unwrap()
            }
        }

        let mut buffer = Rgba32Buffer([0; WIDTH * HEIGHT * 4]);

        buffer.set_pixel(0, 0, [0xff, 0x00, 0xff, 0xff]);

        assert_eq!(buffer.get_pixel(0, 0), [0xff, 0x00, 0xff, 0xff]);
    }
}
