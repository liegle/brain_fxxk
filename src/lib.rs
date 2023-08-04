mod program;

use std::{env::ArgsOs, ffi::OsString, ops::RangeBounds, str::FromStr, fs, process};

use program::Program;

pub fn run(config: Config) {
    let program = match fs::read_to_string(config.path) {
        Ok(content) => {
            match Program::from(content) {
                Ok(program) => program,
                Err(err) => {
                    println!("{err}");
                    process::exit(0);
                }
            }
        },
        Err(err) => {
            println!("{err}");
            process::exit(0);
        }
    };
    program.print();
}

enum Overflow { Block, Overflow, Exit }

pub struct Config {
    path: OsString,
    overflow: Overflow,
    tape_length: usize,
    window_width: usize,
    tick_duration: f64,
    output_as_int: bool,
}

const KEY_VALUE_PAIRS: &str = "\
Keys                      Values\n\
overflow                  Block | Overflow | Exit\n\
tape_length               int in (0, 4096]\n\
window_width              int in (0, 64]\n\
tick_duration             float in [0, 3]\n\
output_as_int             true | false";

impl Config {
    pub fn new(mut args: ArgsOs) -> Result<Config, String> {
        args.next();
        let path = match args.next() {
            Some(arg) => if arg == "help" {
                return Err(String::from(KEY_VALUE_PAIRS))
            } else {
                arg
            },
            None => return Err(String::from("\
            Didn't find brainfxxk source file!\n\
            Usage: brain_fxxker.exe <file_path> <key>=<value> <...>\n\
            For more information, use brain_fxxker.exe help.")),
        };

        let mut config = Config::default(path);
        for arg in args {
            let arg = match arg.clone().into_string() {
                Ok(result) => result,
                Err(_) => return Err(format!("Parameters shouldn't contain not unicode characters in \"{:?}\"!", arg)),
            };
            let syntax_error = Err(format!("Parameter syntax error in \"{}\"!", arg));
            if let Err(_) = config.parse(arg) {
                return syntax_error;
            }
        }
        Ok(config)
    }

    fn default(path: OsString) -> Config {
        Config {
            path,
            overflow: Overflow::Block,
            tape_length: 256_usize,
            window_width: 32_usize,
            tick_duration: 1_f64,
            output_as_int: false,
        }
    }

    fn parse(&mut self, arg: String) -> Result<(), ()> {
        let lower_arg = arg.to_ascii_lowercase();
        let key_value: Vec<_> = lower_arg.split('=').collect();

        if key_value.len() != 2 {
            return Err(());
        }

        match key_value[0] {
            "overflow" => self.overflow = match key_value[1] {
                "block" => Overflow::Block,
                "overflow" => Overflow::Overflow,
                "exit" => Overflow::Exit,
                _ => return Err(()),
            },
            "tape_length" => match arg_to(key_value[1], 1..=4096) {
                Ok(value) => self.tape_length = value,
                Err(_) => return Err(()),
            },
            "window_width" => match arg_to(key_value[1], 1..=64) {
                Ok(value) => self.window_width = value,
                Err(_) => return Err(()),
            },
            "tick_duration" => match arg_to(key_value[1], 0.0..=3.0) {
                Ok(value) => self.tick_duration = value,
                Err(_) => return Err(()),
            },
            "output_as_int" => self.output_as_int = match key_value[1] {
                "true" => true,
                "false" => false,
                _ => return Err(()),
            },
            _ => return Err(()),
        }
        Ok(())
    }
}

fn arg_to<T, R>(arg: &str, range: R) -> Result<T, ()>
where
    T: FromStr + PartialOrd<T>,
    R: RangeBounds<T>,
{
    let value = match arg.parse() {
        Ok(value) => value,
        Err(_) => return Err(()),
    };
    if range.contains(&value) {
        Ok(value)
    } else {
        Err(())
    }
}