use std::fs;

use crossterm::event::{self, Event, KeyEvent, KeyCode};

use crate::{program::{Program, Key}, Config, Overflow};

pub struct Context {
    program: Program,
    program_counter: usize,
    tape: Vec<u8>,
    pointer: usize,
    virtual_pointer: i64,

    input: String,
    output: String,

    config: Config,
}

pub enum Step {
    Next,
    Err(&'static str),
    End,
}

impl Context {
    pub fn new(config: Config) -> Result<Context, String> {
        let program = match fs::read_to_string(&config.path) {
            Ok(content) => match Program::from(content) {
                Ok(program) => program,
                Err(err) => return Err(err),
            },
            Err(err) => return Err(err.to_string())
        };
        Ok(Context {
            program,
            program_counter: 0_usize,
            tape: vec![0; config.tape_length],
            pointer: 0_usize,
            virtual_pointer: 0_i64,
            input: String::new(),
            output: String::new(),
            config,
        })
    }

    pub fn step(&mut self) -> Step {
        if self.program_counter == self.program.len() - 1 {
            Step::End
        } else {
            self.program_counter += 1;
            match self.program.get(self.program_counter) {
                None => Step::Err("This should not happen!"),
                Some(key) => match *key {
                    Key::Right => {
                        self.virtual_pointer += 1;
                        match self.fix_pointer() {
                            Ok(_) => Step::Next,
                            Err(_) => Step::Err("Overflow exit."),
                        }
                    },
                    Key::Left => {
                        self.virtual_pointer -= 1;
                        match self.fix_pointer() {
                            Ok(_) => Step::Next,
                            Err(_) => Step::Err("Overflow exit."),
                        }
                    },
                    Key::Add => {
                        self.tape[self.pointer] += 1;
                        Step::Next
                    },
                    Key::Sub => {
                        self.tape[self.pointer] -= 1;
                        Step::Next
                    },
                    Key::In => {
                        loop {
                            match event::read() {
                                Err(_) => return Step::Err("Failed to read event."),
                                Ok(e) => if let Event::Key(key) = e {
                                    if let KeyCode::Char(ch) = key.code {
                                        self.tape[self.pointer] = ch as u8;
                                        self.input.push(ch);
                                        break;
                                    }
                                },
                            }
                        }
                        Step::Next
                    },
                    Key::Out => {
                        self.output.push(self.tape[self.pointer] as char);
                        Step::Next
                    },
                    Key::If(index) => {
                        if self.tape[self.pointer] == 0 {
                            self.program_counter = index;
                        }
                        Step::Next
                    },
                    Key::Back(index) => {
                        if self.tape[self.pointer] != 0 {
                            self.program_counter = index;
                        }
                        Step::Next
                    },
                }
            }
        }
    }

    pub fn refresh(&self) {

    }

    fn execute(&mut self, key: Key) {
        
    }

    fn len(&self) -> i64 {
        self.tape.len() as i64
    }

    fn fix_pointer(&mut self) -> Result<(), ()> {
        if self.virtual_pointer < 0 {
            match self.config.overflow {
                Overflow::Block => {
                    self.virtual_pointer = 0;
                    self.pointer = 0;
                },
                Overflow::Overflow => self.pointer = 0,
                Overflow::Loop => {
                    while self.virtual_pointer < 0 {
                        self.virtual_pointer += self.len();
                    }
                    self.pointer = self.virtual_pointer as usize;
                },
                Overflow::Exit => return Err(()),
            }
        } else if self.virtual_pointer >= self.len() {
            match self.config.overflow {
                Overflow::Block => {
                    self.virtual_pointer = self.len() - 1;
                    self.pointer = self.tape.len() - 1;
                },
                Overflow::Overflow => self.pointer = self.tape.len() - 1,
                Overflow::Loop => {
                    while self.virtual_pointer >= self.len() {
                        self.virtual_pointer -= self.len();
                    }
                    self.pointer = self.virtual_pointer as usize;
                },
                Overflow::Exit => return Err(()),
            }
        } else {
            self.pointer = self.virtual_pointer as usize;
        }
        Ok(())
    }
}