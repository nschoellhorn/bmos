#![no_std]
extern crate alloc;

use crate::builtins::BUILTINS;
use crate::parser::parse_command_line;
use alloc::string::String;
use bmos_std::io::IOChannel;
use bmos_std::kdebug;
use bmos_std::syscall::print;

pub mod builtins;
pub mod parser;

fn main() {}

pub trait Shell {
    fn process_input(&self, input: String);
}

pub struct BmShell;

impl BmShell {
    pub fn new() -> Self {
        Self
    }

    fn print_parse_error<D: core::fmt::Debug>(&self, error: D) {
        kdebug!("Parsing error: {:?}", error);
        print(IOChannel::Stdout, "Invalid command syntax");
    }
}

impl Shell for BmShell {
    fn process_input(&self, input: String) {
        let parse_result = parse_command_line(input.trim());
        match parse_result {
            Ok((leftover, mut command_line)) => {
                if !leftover.is_empty() {
                    self.print_parse_error(leftover);
                    return;
                }
                kdebug!("Parsed command line: {:?}", command_line);

                let command = command_line.remove(0);
                let arguments = command_line;

                kdebug!("Command: {}, Arguments: {:?}", command, arguments);
                match (*BUILTINS).get(command) {
                    Some(builtin) => builtin.execute(arguments),
                    None => {
                        print(IOChannel::Stdout, "Command not found.");
                    }
                }
            }
            Err(error) => {
                self.print_parse_error(error);
            }
        }
    }
}
