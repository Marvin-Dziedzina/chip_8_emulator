use crate::io::{self, MemoryError};

#[derive(Debug)]
pub struct V {
    v: [u8; 16],
}
impl V {
    pub fn new() -> Self {
        V { v: [0u8; 16] }
    }
}

impl io::Read for V {
    type Bit = u8;
    type Address = u8;

    fn read(&self, address: u8) -> Result<u8, MemoryError> {
        self.v
            .get(address as usize)
            .copied()
            .ok_or(MemoryError::OutOfBounds)
    }

    fn read_range(&self, start_address: u8, end_offset: u8) -> Result<&[u8], MemoryError> {
        if start_address
            .checked_add(end_offset)
            .ok_or(MemoryError::InvalidRange)?
            >= self.v.len() as u8
        {
            return Err(MemoryError::OutOfBounds);
        };

        Ok(&self.v[start_address as usize..(start_address + end_offset) as usize])
    }
}

impl io::Write for V {
    type Bit = u8;
    type Address = u8;

    fn write(&mut self, address: u8, data: u8) -> Result<(), MemoryError> {
        *self
            .v
            .get_mut(address as usize)
            .ok_or(MemoryError::OutOfBounds)? = data;

        Ok(())
    }

    fn write_buf(&mut self, start_address: u8, data: &[Self::Bit]) -> Result<(), MemoryError> {
        let end_address = start_address
            .checked_add(data.len() as u8)
            .ok_or(MemoryError::OutOfBounds)?;

        self.v[start_address as usize..end_address as usize].copy_from_slice(data);

        Ok(())
    }
}

#[derive(Debug)]
pub struct I {
    i: u16,
}
impl I {
    pub fn new() -> Self {
        I { i: 0 }
    }

    pub fn read(&self) -> u16 {
        self.i
    }

    pub fn write(&mut self, data: u16) {
        self.i = data;
    }
}
