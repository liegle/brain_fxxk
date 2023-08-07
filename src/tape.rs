use std::process;

pub struct Tape {
    data: Vec<u8>,
}

impl Tape {
    pub fn new(length: usize) -> Tape {
        Tape {
            data: vec![0; length],
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn read(&self, index: usize) -> Option<u8> {
        match self.data.get(index) {
            Some(value) => Some(*value),
            None => None,
        }
    }

    pub fn write(&mut self, index: usize, value: u8) -> Result<(), ()> {
        if (0..self.data.len()).contains(&index) {
            self.data[index] = value;
            Ok(())
        } else {
            Err(())
        }
    }
}