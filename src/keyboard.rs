use crate::debug;
use pc_keyboard::{DecodedKey, KeyCode, KeyState};

type Handler<'a> = &'a (dyn KeyboardHandler + Send + Sync);

pub static mut KEYBOARD_REGISTRY: Option<KeyboardEventRegistry<'static>> = None;

pub fn init() {
    unsafe { KEYBOARD_REGISTRY = Some(KeyboardEventRegistry::new()) };
}

pub struct KeyboardEventRegistry<'a> {
    handlers: [Option<Handler<'a>>; 8],
}

pub enum RegistryError {
    NoAvailableSlot,
}

impl<'a> KeyboardEventRegistry<'a> {
    pub fn new() -> Self {
        Self {
            handlers: [None; 8],
        }
    }

    pub fn register(&mut self, handler: Handler<'a>) -> Result<(), RegistryError> {
        let empty_slot = self
            .handlers
            .iter()
            .enumerate()
            .find(|(_, slot)| slot.is_none());
        match empty_slot {
            Some((index, _)) => {
                self.handlers[index] = Some(handler);

                Ok(())
            }
            None => Err(RegistryError::NoAvailableSlot),
        }
    }

    pub fn dispatch_event(&self, event: KeyEvent) {
        self.handlers
            .iter()
            .enumerate()
            .filter(|(_, handler)| handler.is_some())
            .map(|(slot, handler_option)| (slot, handler_option.unwrap()))
            .for_each(|(slot, handler)| {
                debug!("Dispatching event to handler in slot {}", slot);
                handler.handle_key_event(event)
            });
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    key_code: KeyCode,
    key_state: KeyState,
    decoded_key: Option<DecodedKey>,
}

impl KeyEvent {
    pub fn new(key_code: KeyCode, key_state: KeyState, decoded_key: Option<DecodedKey>) -> Self {
        Self {
            key_code,
            key_state,
            decoded_key,
        }
    }

    pub fn key_code(&self) -> KeyCode {
        self.key_code
    }

    pub fn key_state(&self) -> KeyState {
        self.key_state
    }

    pub fn decoded_key(&self) -> Option<DecodedKey> {
        self.decoded_key
    }
}

pub trait KeyboardHandler {
    fn handle_key_event(&self, event: KeyEvent);
}
