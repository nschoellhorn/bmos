use bootloader::boot_info::FrameBuffer as BootFrameBuffer;

pub struct GraphicsSettings {
    pub width: u32,
    pub height: u32,
}

unsafe impl Send for Framebuffer {}
pub struct Framebuffer {
    framebuffer: *mut u8,
    fb_size: usize,
    fb_scanline: usize,
}

impl Framebuffer {
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u32) {
        let fb_offset = self.fb_scanline * y as usize + x as usize * 4;
        let target_location = (self.framebuffer as u64 + fb_offset as u64) as *mut u32;

        unsafe {
            *target_location = color;
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        for xo in 0..width {
            for yo in 0..height {
                self.draw_pixel(x + xo, y + yo, color);
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            core::slice::from_raw_parts_mut(self.framebuffer, self.fb_size).fill(0);
        }
    }

    pub fn from_boot_info_framebuffer(fb: &mut BootFrameBuffer) -> Self {
        Self {
            fb_size: fb.info().byte_len,
            framebuffer: fb.buffer_mut().as_mut_ptr(),
            fb_scanline: fb.info().stride * fb.info().bytes_per_pixel,
        }
    }
}

pub struct CursorPosition {
    pub column: u32,
    pub row: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct psf2_t {
    pub magic: u32,
    pub version: u32,
    pub headersize: u32,
    pub flags: u32,
    pub numglyph: u32,
    pub bytesperglyph: u32,
    pub height: u32,
    pub width: u32,
    pub glyphs: u8,
}
