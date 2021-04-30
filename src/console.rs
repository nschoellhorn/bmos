use crate::graphics::{Framebuffer, GraphicsSettings};
use crate::terminal::CursorPosition;
use alloc::vec;
use alloc::vec::Vec;
use psf::Font;
use spin::Mutex;

// base color, simply white for now
const BACKGROUND_COLOR: u32 = 0x000000;
const FOREGROUND_COLOR: u32 = 0xffffff;

pub struct Console<'a> {
    font: &'a Font<'a>,
    framebuffer: &'a Mutex<Framebuffer>,
    width: u32,  // width of the screen/console in characters
    height: u32, // height of the screen/console in characters
    screen_buffer: Mutex<Vec<Vec<Option<char>>>>,
}

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

            screen_buffer: Mutex::new(vec![vec![None; width as usize]; height as usize]),
        };

        this.redraw_screen(CursorPosition { row: 0, column: 0 });

        this
    }

    fn draw_cursor(&self, cursor_position: CursorPosition, color: u32) {
        let mut framebuffer = self.framebuffer.lock();
        let x = cursor_position.column;
        let y = cursor_position.row;

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

    pub fn redraw_screen(&self, cursor_position: CursorPosition) {
        let mut framebuffer = self.framebuffer.lock();
        framebuffer.clear();

        // Make sure we drop the lock after clearing the frame buffer
        //  because the draw_char() function needs to lock it as well to draw characters.
        drop(framebuffer);

        self.draw_cursor(cursor_position, FOREGROUND_COLOR);

        let screen_buffer = self.screen_buffer.lock();
        screen_buffer.iter().enumerate().for_each(|(y, line)| {
            line.iter()
                .enumerate()
                .filter(|(_, optional_char)| optional_char.is_some())
                .map(|(x, optional_char)| (x, optional_char.unwrap()))
                .for_each(|(x, character)| {
                    let foreground_color = if x == cursor_position.column as usize
                        && y == cursor_position.row as usize
                    {
                        BACKGROUND_COLOR
                    } else {
                        FOREGROUND_COLOR
                    };

                    self.draw_char(character, x as u32, y as u32, foreground_color);
                });
        });

        let mut framebuffer = self.framebuffer.lock();
        framebuffer.flip();
    }

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

    pub fn delete_char(&self, x: u32, y: u32) {
        let mut screen_buffer = self.screen_buffer.lock();
        let mut current_row = screen_buffer.get_mut(y as usize).unwrap();

        current_row[x as usize] = None;
    }

    pub fn put_char(&self, c: char, x: u32, y: u32) {
        let mut screen_buffer = self.screen_buffer.lock();
        let row = screen_buffer.get_mut(y as usize).unwrap();
        row[x as usize] = Some(c);
    }

    pub fn width(&self) -> u32 {
        self.width
    }
}
