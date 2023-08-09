use std::{
    fs,
    io::{
        self,
        Write,
    }
};

use crossterm::{
    queue,
    event::{
        self,
        Event,
        KeyCode,
    },
    terminal::{
        LeaveAlternateScreen,
        EnterAlternateScreen,
        DisableLineWrap,
        EnableLineWrap,
    },
    cursor::{
        Show,
        Hide,
        MoveTo,
        MoveToNextLine,
        MoveRight, MoveToColumn, SavePosition, RestorePosition,
    },
    style::{
        Print,
        PrintStyledContent,
        Stylize,
    }
};

use crate::{
    program::{
        Program,
        Key,
    },
    Config,
    Overflow,
};

pub struct Context {
    program: Program,
    program_counter: usize,
    command_excuted: u32,
    tape: Vec<u8>,
    pointer: usize,
    virtual_pointer: i64,

    input: String,
    output: String,

    config: Config,
    _screen: Screen,
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

        let _screen = match Screen::new(config.window_width) {
            Ok(screen) => screen,
            Err(err) => return Err(format!("Failed to init screen.{}", err))
        };

        let context = Context {
            program,
            program_counter: 0_usize,
            command_excuted: 0_u32,
            tape: vec![0; config.tape_length],
            pointer: 0_usize,
            virtual_pointer: 0_i64,
            input: String::new(),
            output: String::new(),
            config,
            _screen,
        };

        Ok(context)
    }

    pub fn step(&mut self) -> Step {
        if self.program_counter == self.program.len() - 1 {
            Step::End
        } else {
            self.program_counter += 1;
            self.command_excuted += 1;
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
        todo!();
    }

    fn fix_pointer(&mut self) -> Result<(), ()> {
        let len = self.tape.len() as i64;
        if self.virtual_pointer < 0 {
            match self.config.overflow {
                Overflow::Block => {
                    self.virtual_pointer = 0;
                    self.pointer = 0;
                },
                Overflow::Overflow => self.pointer = 0,
                Overflow::Loop => {
                    self.virtual_pointer = self.virtual_pointer % len + len;
                    self.pointer = self.virtual_pointer as usize;
                },
                Overflow::Exit => return Err(()),
            }
        } else if self.virtual_pointer >= len {
            match self.config.overflow {
                Overflow::Block => {
                    self.virtual_pointer = len - 1;
                    self.pointer = self.tape.len() - 1;
                },
                Overflow::Overflow => self.pointer = self.tape.len() - 1,
                Overflow::Loop => {
                    self.virtual_pointer = self.virtual_pointer % len;
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

struct Screen;

impl Screen {
    fn new(window_width: u16) -> Result<Screen, io::Error> {
        let length = window_width * 2 - 1;
        let mut line = String::new();
        for _ in 0..length {
            line.push('─');
        }
        queue!(
            io::stdout(),
            EnterAlternateScreen,
            Hide,
            DisableLineWrap,
            SavePosition,
            MoveTo(0, 0),
            Print("┌"), Print(&line), Print("┐"), MoveToColumn(1), PrintStyledContent("Code:".bold()), MoveToNextLine(1),
            Print("│"), MoveRight(length), Print("│"), MoveToNextLine(1),
            Print("│"), MoveRight(length), Print("│"), MoveToNextLine(1),
            Print("└"), Print(&line), Print("┘"), MoveToColumn(1), PrintStyledContent("Exectued commands:".bold()), MoveToNextLine(1),
            Print("┌"), Print(&line), Print("┐"), MoveToColumn(1), PrintStyledContent("Tape:".bold()), MoveToNextLine(1),
            Print("│"), MoveRight(length), Print("│"), MoveToNextLine(1),
            Print("│"), MoveRight(length), Print("│"), MoveToNextLine(1),
            Print("└"), Print(&line), Print("┘"), MoveToNextLine(1),
            PrintStyledContent("Input:".bold()), MoveToNextLine(3),
            //
            //
            PrintStyledContent("Output:".bold()), MoveToNextLine(3),
            //
            //
            Print(" "),
        )?;
        io::stdout().flush()?;
        Ok(Screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = queue!(
            io::stdout(),
            LeaveAlternateScreen,
            Show,
            EnableLineWrap,
            RestorePosition,
        );
        let _ = io::stdout().flush();
    }
}