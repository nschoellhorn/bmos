use crate::console::{Console, Position};
use crate::debug;
use crate::keyboard::{KeyEvent, KeyboardHandler};
use core::cell::RefCell;
use pc_keyboard::{DecodedKey, KeyCode, KeyState};
use spin::Mutex;

static PROMPT: &'static str = "bmos> ";

pub struct Terminal<'a> {
    cursor: Mutex<RefCell<Position>>,
    console: &'a Console<'a>,
}

impl<'a> Terminal<'a> {
    pub fn new(console: &'a Console<'a>) -> Self {
        let this = Self {
            cursor: Mutex::new(RefCell::new(Position { row: 0, column: 0 })),
            console,
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
            (KeyCode::Enter, KeyState::Down) => {
                let start = Position {
                    column: PROMPT.len() as u32,
                    row: cursor.row,
                };
                let end = Position {
                    column: core::cmp::max(cursor.column - 1, PROMPT.len() as u32),
                    row: cursor.row,
                };
                debug!("End: {:?}", end);

                let input = self.console.get_range_as_string(start, end);

                debug!("User Input: {:?}", &input);

                return;
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
