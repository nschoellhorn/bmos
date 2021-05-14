use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use bmos_std::io::IOChannel;
use bmos_std::kdebug;
use bmos_std::syscall;
use hashbrown::HashMap;
use lazy_static::lazy_static;

pub trait ShellBuiltin {
    fn execute(&self, arguments: Vec<&str>);
}

pub struct Echo;

impl ShellBuiltin for Echo {
    fn execute(&self, arguments: Vec<&str>) {
        if arguments.is_empty() {
            syscall::print(IOChannel::Stdout, "");
            return;
        }
        let full_string = arguments.join(" ");
        syscall::print(IOChannel::Stdout, full_string.as_str());
    }
}

pub struct Something;

impl ShellBuiltin for Something {
    fn execute(&self, arguments: Vec<&str>) {
        kdebug!("SOMETHING!");
    }
}

lazy_static! {
    pub static ref BUILTINS: HashMap<String, Box<(dyn ShellBuiltin + Send + Sync + 'static)>> = {
        let mut builtins =
            HashMap::<_, Box<(dyn ShellBuiltin + Send + Sync + 'static)>>::with_capacity(1);
        builtins.insert(String::from("echo"), Box::new(Echo));
        builtins.insert(String::from("something"), Box::new(Something));

        builtins
    };
}

fn builtin_echo(arguments: Vec<&str>) {}
