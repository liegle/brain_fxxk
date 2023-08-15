use std::{
    fs,
    io::{self, Write}
};

use crossterm::{
    queue,
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    cursor,
    style::{self, Stylize}
};

use crate::{
    program::{Program, Key},
    Config,
    Overflow,
};

pub struct Context {
    program: Program,
    program_counter: usize,
    command_executed: Option<u32>,
    tape: Vec<u8>,
    pointer: usize,
    virtual_pointer: i64,

    program_left: usize,
    tape_left: i64,
    input: String,
    output: String,

    config: Config,
    _screen: Screen,
}

pub enum Step {
    Next,
    Err(String),
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

        let _screen = match Screen::new(config.window_width as u16) {
            Ok(screen) => screen,
            Err(err) => return Err(format!("Failed to init screen.{}", err))
        };

        Ok(Context {
            program,
            program_counter: 0,
            command_executed: None,
            tape: vec![0; config.tape_length],
            pointer: 0,
            virtual_pointer: 0,

            program_left: 0,
            tape_left: 0,
            input: String::new(),
            output: String::new(),

            config,
            _screen,
        })
    }

    pub fn step(&mut self) -> Step {
        if self.program_counter == self.program.len() {
            return Step::End;
        }
        let step = match self.command_executed {
            None => {
                self.command_executed = Some(0);
                Step::Next
            },
            Some(n) => {
                self.command_executed = Some(n + 1);
                self.command()
            },
        };
        match self.refresh() {
            Ok(_) => step,
            Err(err) => Step::Err(err),
        }
    }

    fn command(&mut self) -> Step {
        match self.program.get(self.program_counter) {
            None => Step::Err(String::from("This should not happen!")),
            Some(key) => match *key {
                Key::Right => {
                    self.virtual_pointer += 1;
                    self.program_counter += 1;
                    match self.fix_pointer() {
                        Ok(_) => Step::Next,
                        Err(_) => Step::Err(String::from("Overflow exit.")),
                    }
                },
                Key::Left => {
                    self.virtual_pointer -= 1;
                    self.program_counter += 1;
                    match self.fix_pointer() {
                        Ok(_) => Step::Next,
                        Err(_) => Step::Err(String::from("Overflow exit.")),
                    }
                },
                Key::Add => {
                    match self.tape[self.pointer] {
                        u8::MAX => self.tape[self.pointer] = 0,
                        _ => self.tape[self.pointer] += 1,
                    }
                    self.program_counter += 1;
                    Step::Next
                },
                Key::Sub => {
                    match self.tape[self.pointer] {
                        0 => self.tape[self.pointer] = u8::MAX,
                        _ => self.tape[self.pointer] -= 1,
                    }
                    self.program_counter += 1;
                    Step::Next
                },
                Key::In => {
                    loop {
                        match event::read() {
                            Err(_) => return Step::Err(String::from("Failed to read event.")),
                            Ok(e) => if let Event::Key(key) = e {
                                if let KeyCode::Char(ch) = key.code {
                                    self.tape[self.pointer] = ch as u8;
                                    self.input.push(ch);
                                    break;
                                }
                            },
                        }
                    }
                    let _ = event::read();// 按一次键盘会有两个 event，不知道是不是 bug
                    self.program_counter += 1;
                    Step::Next
                },
                Key::Out => {
                    if self.config.output_as_int {
                        self.output.push_str(&self.tape[self.pointer].to_string());
                        self.output.push(' ');
                    } else {
                        self.output.push(self.tape[self.pointer] as char);
                    }
                    self.program_counter += 1;
                    Step::Next
                },
                Key::If(index) => {
                    if self.tape[self.pointer] == 0 {
                        self.program_counter = index;
                    } else {
                        self.program_counter += 1;
                    }
                    Step::Next
                },
                Key::Back(index) => {
                    if self.tape[self.pointer] != 0 {
                        self.program_counter = index;
                    } else {
                        self.program_counter += 1;
                    }
                    Step::Next
                },
            }
        }
    }

    fn refresh(&mut self) -> Result<(), String> {
        let width = self.config.window_width;
        let len = 2 * width as u16;

        if self.program_counter < self.program_left {
            self.program_left = self.program_counter;
        } else if self.program_counter >= self.program_left + width {
            self.program_left = self.program_counter - width + 1;
        }
        let program_output = self.program.slice_string(self.program_left, width);
        let program_pin = 2 * (self.program_counter - self.program_left) as u16 + 1;
        if let Err(err) = queue!(
            io::stdout(),
            cursor::MoveTo(7, 0),
            style::Print(self.program_counter),
            cursor::MoveTo(1, 1),
            style::Print(program_output),
            cursor::MoveTo(0, 2),
            terminal::Clear(ClearType::CurrentLine),
            style::Print('│'),
            cursor::MoveTo(program_pin, 2),
            style::Print('^'),
            cursor::MoveTo(len, 2),
            style::Print('│'),
            cursor::MoveTo(20, 3),
            style::Print(self.command_executed.unwrap_or_else(|| 0)),
        ) { return Err(err.to_string()) }

        let width = width as i64 / 2;

        if self.virtual_pointer < self.tape_left {
            self.tape_left = self.virtual_pointer;
        } else if self.virtual_pointer >= self.tape_left + width {
            self.tape_left = self.virtual_pointer - width + 1;
        }
        let tape_output = self.slice_tape(self.tape_left, width);
        let tape_pin = 4 * (self.virtual_pointer - self.tape_left) as u16 + 2;
        if let Err(err) = queue!(
            io::stdout(),
            cursor::MoveTo(7, 4),
            style::Print(self.virtual_pointer),
            cursor::MoveTo(1, 5),
            style::Print(tape_output),
            cursor::MoveTo(0, 6),
            terminal::Clear(ClearType::CurrentLine),
            style::Print('│'),
            cursor::MoveTo(tape_pin, 6),
            style::Print('^'),
            cursor::MoveTo(len, 6),
            style::Print('│'),
        ) { return Err(err.to_string()) }

        if let Err(err) = queue!(
            io::stdout(),
            cursor::MoveTo(0, 9),
            style::Print(&self.input),
            cursor::MoveTo(0, 12),
            style::Print(&self.output),
        ) { return Err(err.to_string()) }
        
        if let Err(err) = io::stdout().flush() {
            return Err(err.to_string())
        }
        Ok(())
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

    fn slice_tape(&self, left: i64, width: i64) -> String {
        let mut out = String::new();
        let tape_range = 0..self.tape.len() as i64;
        for index in left..left + width {
            if tape_range.contains(&index) {
                let num = self.tape[index as usize];
                out.push((num / 100 + 48) as char);
                out.push((num % 100 / 10 + 48) as char);
                out.push((num % 10 + 48) as char);
            } else {
                out.push_str("---");
            }
            out.push(' ');
        }
        out.pop();
        out
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
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::DisableLineWrap,
            cursor::MoveTo(0, 0),
            style::Print("┌"), style::Print(&line), style::Print("┐"), cursor::MoveToColumn(1), style::PrintStyledContent("Code@ ".bold()), cursor::MoveToNextLine(1),
            style::Print("│"), cursor::MoveRight(length), style::Print("│"), cursor::MoveToNextLine(1),
            style::Print("│"), cursor::MoveRight(length), style::Print("│"), cursor::MoveToNextLine(1),
            style::Print("└"), style::Print(&line), style::Print("┘"), cursor::MoveToColumn(1), style::PrintStyledContent("Executed commands: ".bold()), cursor::MoveToNextLine(1),
            style::Print("┌"), style::Print(&line), style::Print("┐"), cursor::MoveToColumn(1), style::PrintStyledContent("Tape@ ".bold()), cursor::MoveToNextLine(1),
            style::Print("│"), cursor::MoveRight(length), style::Print("│"), cursor::MoveToNextLine(1),
            style::Print("│"), cursor::MoveRight(length), style::Print("│"), cursor::MoveToNextLine(1),
            style::Print("└"), style::Print(&line), style::Print("┘"), cursor::MoveToNextLine(1),
            style::PrintStyledContent("Input:".bold()), cursor::MoveToNextLine(3),
            //
            //
            style::PrintStyledContent("Output:".bold()), cursor::MoveToNextLine(3),
            //
            //
            style::Print(" "),
        )?;
        io::stdout().flush()?;
        Ok(Screen)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = queue!(
            io::stdout(),
            terminal::LeaveAlternateScreen,
            cursor::Show,
            terminal::EnableLineWrap,
        );
        let _ = io::stdout().flush();
    }
}