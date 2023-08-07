use std::fmt::Display;

enum Key {
    Right, Left,               // > <
    Add, Sub,                  // + -
    Out, In,                   // . ,
    If(usize), Back(usize),    // [ ]
}

pub struct Program {
    code: Vec<Key>,
}

impl Program {
    pub fn from(source: String) -> Result<Program, String> {
        let syntax_error = Err(String::from("Brainfxxk source syntax error!"));

        let mut code = Vec::new();
        for ch in source.chars() {
            code.push(match ch {
                '>' => Key::Right,
                '<' => Key::Left,
                '+' => Key::Add,
                '-' => Key::Sub,
                '.' => Key::Out,
                ',' => Key::In,
                '[' => Key::If(0),
                ']' => match code.last() {
                    Some(key) => match key {
                        Key::If(_) => {
                            code.pop();
                            continue;
                        },
                        _ => Key::Back(0),
                    }
                    None => return syntax_error,
                },
                _ => continue,
            });
        }

        let mut stack = Vec::new();
        for (index, key) in code.iter_mut().enumerate() {
            match key {
                Key::If(_) => stack.push((index, key)),
                Key::Back(back_index) => match stack.pop() {
                    Some(counterpart) => {
                        *back_index = counterpart.0 + 1;
                        if let Key::If(if_index) = counterpart.1 {
                            *if_index = index + 1;
                        }
                    }
                    None => return syntax_error,
                },
                _ => (),
            }
        }
        if stack.len() != 0 {
            syntax_error
        } else {
            Ok(Program { code })
        }
    }

    pub fn get(&self, index: usize) -> Option<&Key> {
        self.code.get(index)
    }

    pub fn code_to_string(&self, divider: char, show_jump: bool) -> String {
        let mut out = String::new();
        for key in &self.code {
            out.push(match key {
                Key::Right => '>',
                Key::Left => '<',
                Key::Add => '+',
                Key::Sub => '-',
                Key::Out => '.',
                Key::In => ',',
                Key::If(value) => {
                    if show_jump {
                        out.push('[');
                        out.push_str(&value.to_string());
                        out.push(divider);
                        continue;
                    } else {
                        '['
                    }
                },
                Key::Back(value) => {
                    if show_jump {
                        out.push(']');
                        out.push_str(&value.to_string());
                        out.push(divider);
                        continue;
                    } else {
                        ']'
                    }
                },
            });
            out.push(divider);
        }
        out
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.code_to_string(' ', false))
    }
}