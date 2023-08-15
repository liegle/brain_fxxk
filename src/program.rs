pub enum Key {
    Right,
    Left, // > <
    Add,
    Sub, // + -
    Out,
    In, // . ,
    If(usize),
    Back(usize), // [ ]
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
                        }
                        _ => Key::Back(0),
                    },
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
                        *back_index = counterpart.0;
                        if let Key::If(if_index) = counterpart.1 {
                            *if_index = index;
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

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn slice_string(&self, left: usize, width: usize) -> String {
        let right = self.code.len().min(left + width);
        let mut out = String::new();
        for key in &self.code[left..right] {
            out.push(match key {
                Key::Right => '>',
                Key::Left => '<',
                Key::Add => '+',
                Key::Sub => '-',
                Key::Out => '.',
                Key::In => ',',
                Key::If(_) => '[',
                Key::Back(_) => ']',
            });
            out.push(' ');
        }

        let diff = left + width - right;
        if diff > 0 {
            for _ in 0..diff {
                out.push_str("  ");
            }
        }

        out.pop();
        out
    }
}
