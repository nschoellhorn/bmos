use alloc::vec::Vec;
use bootloader::boot_info::FrameBuffer as BootFrameBuffer;

pub struct GraphicsSettings {
    pub width: u32,
    pub height: u32,
}

unsafe impl Send for Framebuffer {}
pub struct Framebuffer {
    front_buffer: *mut u8,
    back_buffer: Vec<u8>,
    fb_size: usize,
    fb_scanline: usize,
}

impl Framebuffer {
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u32) {
        let fb_offset = self.fb_scanline * y as usize + x as usize * 4;
        let target_location = (self.back_buffer.as_mut_ptr() as u64 + fb_offset as u64) as *mut u32;

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
            self.back_buffer.as_mut_slice().fill(0);
        }
    }

    pub fn flip(&mut self) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.back_buffer.as_ptr(),
                self.front_buffer,
                self.fb_size,
            );
        }
    }

    pub fn from_boot_info_framebuffer(fb: &mut BootFrameBuffer) -> Self {
        let fb_scanline = fb.info().stride * fb.info().bytes_per_pixel;
        let fb_size = fb.info().byte_len;
        let mut back_buffer = Vec::with_capacity(fb_size);
        back_buffer.resize(fb_size, 0);
        let base_framebuffer_slice = fb.buffer_mut();
        back_buffer.as_mut_slice()[..].copy_from_slice(base_framebuffer_slice);

        Self {
            fb_size,
            front_buffer: base_framebuffer_slice.as_mut_ptr(),
            back_buffer,
            fb_scanline,
        }
    }
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
