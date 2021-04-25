use crate::debug;
use crate::graphics::{CursorPosition, Framebuffer, GraphicsSettings};
use crate::keyboard::KeyEvent;
use crate::keyboard::KeyboardHandler;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use pc_keyboard::DecodedKey;
use psf::Font;
use spin::Mutex;

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

// base color, simply white for now
const BACKGROUND_COLOR: u32 = 0x000000;
const FOREGROUND_COLOR: u32 = 0xffffff;

pub struct Console<'a> {
    font: &'a Font<'a>,
    cursor: Mutex<RefCell<CursorPosition>>,
    framebuffer: &'a Mutex<Framebuffer>,
    width: u32,  // width of the screen/console in characters
    height: u32, // height of the screen/console in characters
    screen_buffer: Mutex<Vec<Vec<Option<char>>>>,
}

/*impl<'a> Write for Console<'a> {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        self.print(string);

        Ok(())
    }
}*/

impl<'a> Console<'a> {
    pub fn init(
        gfx_settings: &'a GraphicsSettings,
        framebuffer: &'a Mutex<Framebuffer>,
        font: &'a Font<'a>,
    ) -> Self {
        let width = gfx_settings.width / (font.width() as u32 + 1);
        let height = gfx_settings.height / font.height() as u32;

        let this = Self {
            width,
            height,
            font,
            framebuffer,
            cursor: Mutex::new(RefCell::new(CursorPosition { row: 0, column: 0 })),
            screen_buffer: Mutex::new(vec![vec![None; width as usize]; height as usize]),
        };

        this.redraw_screen();

        this
    }

    fn draw_cursor(&self, color: u32) {
        let mut framebuffer = self.framebuffer.lock();
        let lock = self.cursor.lock();
        let cursor = lock.borrow();
        let x = cursor.column;
        let y = cursor.row;

        for fb_y in 0..self.font.height() as u32 {
            for fb_x in 0..self.font.width() as u32 {
                framebuffer.draw_pixel(
                    ((self.font.width() as u32 + 1) * x) + fb_x,
                    (self.font.height() as u32 * y) + fb_y,
                    color,
                );
            }
        }
    }

    pub fn redraw_screen(&self) {
        let mut framebuffer = self.framebuffer.lock();
        framebuffer.clear();

        // Make sure we drop the lock after clearing the frame buffer
        //  because the draw_char() function needs to lock it as well to draw characters.
        drop(framebuffer);

        self.draw_cursor(FOREGROUND_COLOR);

        let screen_buffer = self.screen_buffer.lock();
        let lock = self.cursor.lock();
        let cursor = lock.borrow();
        screen_buffer.iter().enumerate().for_each(|(y, line)| {
            line.iter()
                .enumerate()
                .filter(|(_, optional_char)| optional_char.is_some())
                .map(|(x, optional_char)| (x, optional_char.unwrap()))
                .for_each(|(x, character)| {
                    let foreground_color =
                        if x == cursor.column as usize && y == cursor.row as usize {
                            BACKGROUND_COLOR
                        } else {
                            FOREGROUND_COLOR
                        };

                    self.draw_char(character, x as u32, y as u32, foreground_color);
                });
        });
    }

    /*pub fn print(&mut self, string: &str) {
        for c in string.chars() {
            self.put_char(c);
        }
        self.redraw_screen();
    }

    pub fn println(&mut self, string: &str) {
        self.print(string);
        self.put_char('\n');
        self.redraw_screen();
    }*/

    fn draw_char(&self, c: char, x: u32, y: u32, foreground_color: u32) {
        let mut framebuffer = self.framebuffer.lock();

        let glyph = match self.font.get_char(c) {
            Some(glyph) => glyph,
            None => return,
        };

        for font_y in 0..self.font.height() as u32 {
            for font_x in 0..self.font.width() as u32 {
                if let Some(is_set) = glyph.get(font_x as usize, font_y as usize) {
                    if is_set {
                        framebuffer.draw_pixel(
                            ((self.font.width() as u32 + 1) * x) + font_x,
                            (self.font.height() as u32 * y) + font_y,
                            foreground_color,
                        );
                    }
                }
            }
        }
    }

    pub fn put_char(&self, c: char, x: u32, y: u32) {
        let mut screen_buffer = self.screen_buffer.lock();
        let row = screen_buffer.get_mut(y as usize).unwrap();
        row[x as usize] = Some(c);
    }

    fn move_cursor_right(&self) {
        let lock = self.cursor.lock();
        let mut cursor = lock.borrow_mut();
        if cursor.column == self.width - 1 {
            return;
        }

        cursor.column += 1;
    }
}

impl<'a> KeyboardHandler for Console<'a> {
    fn handle_key_event(&self, event: KeyEvent) {
        if let Some(DecodedKey::Unicode(key)) = event.decoded_key() {
            let lock = self.cursor.lock();
            let cursor = lock.borrow();
            self.put_char(key, cursor.column, cursor.row);
            drop(cursor);
            drop(lock);
            self.move_cursor_right();
            self.redraw_screen();
        }
    }
}
