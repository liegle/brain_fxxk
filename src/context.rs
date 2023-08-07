use crate::{program::Program, tape::Tape};

pub struct Context {
    program: Program,
    program_counter: usize,
    tape: Tape,
    pointer: usize,
}

impl Context {
    pub fn new(program: Program, tape: Tape) -> Context {
        Context {
            program,
            program_counter: 0,
            tape,
            pointer: 0,
        }
    }
}