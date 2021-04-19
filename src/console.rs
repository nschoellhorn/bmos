use crate::graphics::{psf2_t, CursorPosition, Framebuffer, GraphicsSettings};
use core::fmt::Write;
use spin::Mutex;

extern "C" {
    pub static mut _binary_font_psf_start: u64;
}

/*#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}*/

pub struct Console<'a> {
    font: *mut psf2_t,
    cursor: CursorPosition,
    graphics: &'a GraphicsSettings,
    framebuffer: &'a Mutex<Framebuffer>,
    char_width: u32,
    char_height: u32,
}

impl<'a> Write for Console<'a> {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        self.print(string);

        Ok(())
    }
}

impl<'a> Console<'a> {
    pub fn init(
        gfx_settings: &'a GraphicsSettings,
        framebuffer: &'a Mutex<Framebuffer>,
        font: *mut psf2_t,
    ) -> Self {
        unsafe {
            Self {
                char_width: (gfx_settings.width / ((*font).width + 1)),
                char_height: (gfx_settings.height / ((*font).height + 1)),
                graphics: gfx_settings,
                font,
                framebuffer,
                cursor: CursorPosition { row: 0, column: 0 },
            }
        }
    }

    pub fn print(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c, 0xffffff);
        }
    }

    pub fn println(&mut self, string: &str) {
        self.print(string);
        self.put_char('\n', 0xffffff);
    }

    pub fn put_char(&mut self, s: char, color: u32) {
        let mut framebuffer = self.framebuffer.lock();
        if s == '\n' {
            self.cursor.row += 1;
            self.cursor.column = 0;
            return;
        }
        unsafe {
            let font = self.font as *mut psf2_t;
            let (mut line, mut mask, mut offs): (u64, u64, u32);
            let kx = if self.cursor.column >= self.char_width {
                self.cursor.column = 0;
                self.cursor.row += 1;

                self.cursor.column
            } else {
                self.cursor.column
            };
            let ky = if self.cursor.row >= self.char_height {
                // In case we reached the screen height, we just clear the buffer and start over,
                // because we don't have a buffer that saves the printed lines, so scrolling is
                // currently not possible since we can't repaint without said buffer.
                framebuffer.clear();
                self.cursor.column = 0;
                self.cursor.row = 0;

                self.cursor.row
            } else {
                self.cursor.row
            };
            let bpl = ((*font).width + 7) / 8;

            let glyph_a: *mut u8 = (font as u64 + (*font).headersize as u64) as *mut u8;
            let mut glyph: *mut u8 = glyph_a.offset(
                (if s as u8 > 0 && (s as u32) < (*font).numglyph {
                    s as u32
                } else {
                    0
                } * ((*font).bytesperglyph)) as isize,
            );
            for y in 0..(*font).height {
                mask = 1 << ((*font).width - 1);
                for x in 0..(*font).width {
                    let mut target_value: u32 = 0;
                    if (*glyph as u64) & (mask) > 0 {
                        target_value = color;
                    }
                    framebuffer.draw_pixel(
                        ((*font).width * kx) + x,
                        ((*font).height * ky) + y,
                        target_value,
                    );
                    mask >>= 1;
                }
                glyph = glyph.offset(bpl as isize);
            }
        }
        self.cursor.column += 1;
    }
}
