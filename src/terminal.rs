use crate::console::Console;
use crate::keyboard::{KeyEvent, KeyboardHandler};
use core::cell::RefCell;
use pc_keyboard::{DecodedKey, KeyCode, KeyState};
use spin::Mutex;

#[derive(Clone, Copy)]
pub struct CursorPosition {
    pub column: u32,
    pub row: u32,
}

pub struct Terminal<'a> {
    cursor: Mutex<RefCell<CursorPosition>>,
    console: &'a Console<'a>,
}

impl<'a> Terminal<'a> {
    pub fn new(console: &'a Console<'a>) -> Self {
        Self {
            cursor: Mutex::new(RefCell::new(CursorPosition { row: 0, column: 0 })),
            console,
        }
    }

    fn move_cursor_right(&self) -> CursorPosition {
        let lock = self.cursor.lock();
        let mut cursor = lock.borrow_mut();
        if cursor.column == self.console.width() - 1 {
            drop(cursor);
            return (*lock).clone().into_inner();
        }

        cursor.column += 1;

        drop(cursor);
        (*lock).clone().into_inner()
    }

    fn move_cursor_left(&self) -> CursorPosition {
        let lock = self.cursor.lock();
        let mut cursor = lock.borrow_mut();
        if cursor.column == 0 {
            drop(cursor);
            return (*lock).clone().into_inner();
        }

        cursor.column -= 1;

        drop(cursor);
        (*lock).clone().into_inner()
    }
}

impl<'a> KeyboardHandler for Terminal<'a> {
    fn handle_key_event(&self, event: KeyEvent) {
        let lock = self.cursor.lock();
        let mut cursor = *lock.borrow();
        drop(lock);

        match (event.key_code(), event.key_state()) {
            (KeyCode::Backspace, KeyState::Down) => {
                cursor = self.move_cursor_left();
                self.console.delete_char(cursor.column, cursor.row);
                self.console.redraw_screen(cursor);
                return;
            }
            (KeyCode::ArrowLeft, KeyState::Down) => {
                cursor = self.move_cursor_left();
            }
            (KeyCode::ArrowRight, KeyState::Down) => {
                cursor = self.move_cursor_right();
            }
            _ => {}
        }

        if let Some(DecodedKey::Unicode(key)) = event.decoded_key() {
            if cursor.column < self.console.width() - 1 {
                self.console.put_char(key, cursor.column, cursor.row);
                cursor = self.move_cursor_right();
            }
        }

        if event.key_state() == KeyState::Down {
            self.console.redraw_screen(cursor);
        }
    }
}
