mod bindings;

pub struct Lights {
    ptr: bindings::pio_info_t,
}

pub struct PixelData {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl Lights {
    pub fn new(num_pixels: u32, gpio: u32) -> Self {
        unsafe {
            Lights {
                ptr: bindings::init_neopixel(num_pixels, gpio),
            }
        }
    }

    pub fn draw(&mut self, data: Vec<PixelData>) {
        let npix: usize = self.ptr.num_pixels.try_into().unwrap();
        assert_eq!(data.len(), npix, "Data length must match number of pixels");

        let mut new_data: Vec<u8> = Vec::new();
        for d in data {
            new_data.push(d.w);
            new_data.push(d.b);
            new_data.push(d.r);
            new_data.push(d.g);
        }
        unsafe {
            bindings::write_pixels(&mut self.ptr, new_data.as_mut_ptr());
        }
    }
}
