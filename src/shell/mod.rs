use alloc::string::String;

pub mod builtins;
pub mod parser;

pub trait Shell {
    fn process_input(&mut self, input: String);
}

pub struct BmShell {}

impl BmShell {
    pub fn new() -> Self {
        Self {}
    }
}

impl Shell for BmShell {
    fn process_input(&mut self, input: String) {
        let something = "command \"with args in quotes\" and also without quotes";
    }
}
