use crate::io::{self, MemoryError};

#[derive(Debug)]
pub struct RAM {
    memory: [u8; 0x1000],
}
impl RAM {
    pub fn new() -> Self {
        RAM {
            memory: [0u8; 0x1000],
        }
    }
}

impl io::Read for RAM {
    type Bit = u8;
    type Address = u16;

    fn read(&self, address: u16) -> Result<u8, MemoryError> {
        self.memory
            .get(address as usize)
            .copied()
            .ok_or(MemoryError::OutOfBounds)
    }

    fn read_range(&self, start_address: u16, end_offset: u16) -> Result<&[Self::Bit], MemoryError> {
        if start_address
            .checked_add(end_offset)
            .ok_or(MemoryError::InvalidRange)?
            >= self.memory.len() as u16
        {
            return Err(MemoryError::OutOfBounds);
        };

        Ok(&self.memory[start_address as usize..(start_address + end_offset) as usize])
    }
}

impl io::Write for RAM {
    type Bit = u8;
    type Address = u16;

    fn write(&mut self, address: u16, data: u8) -> Result<(), MemoryError> {
        *self
            .memory
            .get_mut(address as usize)
            .ok_or(MemoryError::OutOfBounds)? = data;

        Ok(())
    }

    fn write_buf(&mut self, start_address: u16, data: &[Self::Bit]) -> Result<(), MemoryError> {
        let end_address = start_address
            .checked_add(data.len() as u16)
            .filter(|&end| end <= self.memory.len() as u16)
            .ok_or(MemoryError::OutOfBounds)?;

        self.memory[start_address as usize..end_address as usize].copy_from_slice(data);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Stack {
    stack_pointer: u8,
    stack: [u16; 16],
}
impl Stack {
    pub fn new() -> Self {
        Stack {
            stack_pointer: 0,
            stack: [0u16; 16],
        }
    }

    /// Pushes data onto the stack.
    pub fn push(&mut self, data: u16) -> Result<(), MemoryError> {
        if self.stack_pointer as usize >= self.stack.len() - 1 {
            return Err(MemoryError::StackOverflow);
        };

        *self
            .stack
            .get_mut(self.stack_pointer as usize)
            .ok_or(MemoryError::OutOfBounds)? = data;
        self.stack_pointer += 1;

        Ok(())
    }

    /// Returns the top element of the stack.
    pub fn pop(&mut self) -> Result<u16, MemoryError> {
        if self.stack_pointer == 0 {
            return Err(MemoryError::StackUnderflow);
        };

        self.stack_pointer -= 1;
        self.stack
            .get(self.stack_pointer as usize + 1)
            .cloned()
            .ok_or(MemoryError::DoesNotExist)
    }
}
