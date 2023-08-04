use std::env;
use std::process;

use brain_fxxker::{self, Config};

fn main() {
    let config = Config::new(env::args_os()).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(0);
    });
    brain_fxxker::run(config);
}