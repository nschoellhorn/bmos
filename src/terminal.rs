use crate::console::{Console, Position};
use crate::debug;
use crate::keyboard::{KeyEvent, KeyboardHandler};
use alloc::string::String;
use core::cell::RefCell;
use pc_keyboard::{DecodedKey, KeyCode, KeyState};
use spin::{Mutex, RwLock};

static PROMPT: &'static str = "bmos> ";

pub struct Terminal<'a> {
    cursor: Mutex<RefCell<Position>>,
    console: &'a Console<'a>,
    input_buffer: RwLock<String>,
}

impl<'a> Terminal<'a> {
    pub fn new(console: &'a Console<'a>) -> Self {
        let this = Self {
            cursor: Mutex::new(RefCell::new(Position { row: 0, column: 0 })),
            console,
            input_buffer: RwLock::new(String::new()),
        };

        this.draw_prompt();

        this
    }

    fn draw_prompt(&self) {
        let lock = self.cursor.lock();
        let mut cursor = lock.borrow_mut();

        self.console.print(PROMPT, cursor.column, cursor.row);

        cursor.column = core::cmp::min(
            cursor.column + PROMPT.len() as u32,
            self.console.width() - 1,
        );

        self.console.redraw_screen(cursor.clone());
    }

    fn move_cursor_right(&self) -> Position {
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

    fn move_cursor_left(&self) -> Position {
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

    fn relative_cursor_position(&self) -> Option<Position> {
        let mut cursor = self.cursor.lock().clone().into_inner();

        match cursor.column.checked_sub(PROMPT.len() as u32) {
            Some(new_column) => {
                cursor.column = new_column;

                Some(cursor)
            }
            None => None,
        }
    }

    fn redraw_line_until(&self, position: Position) {
        let input_buffer = self.input_buffer.read();
        for index in PROMPT.len()..PROMPT.len() + position.column as usize {
            self.console.delete_char(index as u32, position.row);
            let input_buffer_pos = index - PROMPT.len();
            if let Some(c) = input_buffer.chars().nth(input_buffer_pos) {
                self.console.put_char(c, index as u32, position.row);
            }
        }
    }
}

impl<'a> KeyboardHandler for Terminal<'a> {
    fn handle_key_event(&self, event: KeyEvent) {
        let lock = self.cursor.lock();
        let mut cursor = *lock.borrow();
        drop(lock);

        match (event.key_code(), event.key_state()) {
            (KeyCode::Backspace, KeyState::Down) => {
                let relative_cursor_position = self.relative_cursor_position();

                match relative_cursor_position {
                    Some(relative_cursor) => {
                        if relative_cursor.column == 0 {
                            return;
                        }
                    }
                    None => return,
                };

                cursor = self.move_cursor_left();
                let relative_cursor = self.relative_cursor_position().unwrap();
                let mut input_buffer = self.input_buffer.write();
                let char_amount = input_buffer.len();
                let _ = input_buffer.remove(relative_cursor.column as usize);
                drop(input_buffer);
                self.redraw_line_until(Position {
                    column: char_amount as u32,
                    row: relative_cursor.row,
                });
                self.console.redraw_screen(cursor);
                return;
            }
            (KeyCode::ArrowLeft, KeyState::Down) => {
                let relative_cursor = self.relative_cursor_position().unwrap();
                if relative_cursor.column == 0 {
                    return;
                }
                cursor = self.move_cursor_left();
            }
            (KeyCode::ArrowRight, KeyState::Down) => {
                let relative_cursor = self.relative_cursor_position().unwrap();
                if relative_cursor.column == self.input_buffer.read().len() as u32 {
                    return;
                }
                cursor = self.move_cursor_right();
            }
            (KeyCode::Enter, KeyState::Down) => {
                let input_buffer = self.input_buffer.read();
                let input = input_buffer.as_str();
                debug!("User Input: {:?}", &input);

                return;
            }

            _ => {}
        }

        if let Some(DecodedKey::Unicode(key)) = event.decoded_key() {
            if cursor.column < self.console.width() - 1 {
                let mut input_buffer = self.input_buffer.write();
                let relative_cursor = self.relative_cursor_position().unwrap();
                input_buffer.insert(relative_cursor.column as usize, key);
                let char_amount = input_buffer.len();
                drop(input_buffer);
                self.redraw_line_until(Position {
                    row: relative_cursor.row,
                    column: PROMPT.len() as u32 + char_amount as u32,
                });
                cursor = self.move_cursor_right();
            }
        }

        if event.key_state() == KeyState::Down {
            self.console.redraw_screen(cursor);
        }
    }
}
