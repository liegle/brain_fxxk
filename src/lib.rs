mod program;
mod context;

use std::{
    env::ArgsOs,
    ffi::OsString,
    ops::RangeBounds,
    str::FromStr,
    process,
    thread,
    time::Duration,
};

use context::{
    Context,
    Step,
};

pub fn run(config: Config) {
    let dur = Duration::from_secs_f64(config.tick_duration);
    let mut context = Context::new(config).unwrap_or_else(|err| {
        println!("{err}");
        process::exit(1);
    });

    loop {
        match context.step() {
            Step::Next => {
                context.refresh();
                thread::sleep(dur);
            },
            Step::End => break,
            Step::Err(err) => {
                println!("{err}");
                process::exit(1);
            }
        }
    }
}

pub enum Overflow {
    Block,     // 指针在边界向外移动时，什么也不发生
    Overflow,  // 指针能移动到边界外，但实际读写的是边界的内存
    Loop,      // 循环移动指针
    Exit,      // 指针在边界向外移动时，立即报错退出
}

pub struct Config {
    path: OsString,
    overflow: Overflow,
    tape_length: usize,
    window_width: u16,
    tick_duration: f64,
    output_as_int: bool,
}

const KEY_VALUE_PAIRS: &str = "\
Keys                      Values\n\
overflow                  Block | Overflow | Loop | Exit\n\
tape_length               int in (0, 256]\n\
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
            if let Err(discription) = config.parse(&arg) {
                return Err(format!("Parameter syntax error in \"{}\"! {}", arg, discription));
            }
        }
        Ok(config)
    }

    fn default(path: OsString) -> Config {
        Config {
            path,
            overflow: Overflow::Block,
            tape_length: 64,
            window_width: 32,
            tick_duration: 0.02,
            output_as_int: false,
        }
    }

    fn parse(&mut self, arg: &str) -> Result<(), &'static str> {
        let lower_arg = arg.to_ascii_lowercase();
        let key_value: Vec<_> = lower_arg.split('=').collect();

        if key_value.len() != 2 {
            return Err("Wrong parameter syntax, please use <key>=<value>.");
        }

        match key_value[0] {
            "overflow" => self.overflow = match key_value[1] {
                "block" => Overflow::Block,
                "overflow" => Overflow::Overflow,
                "exit" => Overflow::Exit,
                _ => return Err("Wrong overflow value."),
            },
            "tape_length" => match arg_to(key_value[1], 1..=256) {
                Ok(value) => self.tape_length = value,
                Err(_) => return Err("Wrong tape_length value."),
            },
            "window_width" => match arg_to(key_value[1], 1..=64) {
                Ok(value) => self.window_width = value,
                Err(_) => return Err("Wrong window_width value."),
            },
            "tick_duration" => match arg_to(key_value[1], 0.0..=3.0) {
                Ok(value) => self.tick_duration = value,
                Err(_) => return Err("Wrong tick_duration value."),
            },
            "output_as_int" => self.output_as_int = match key_value[1] {
                "true" => true,
                "false" => false,
                _ => return Err("Wrong output_as_int value."),
            },
            _ => return Err("Given key doesn't exist."),
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