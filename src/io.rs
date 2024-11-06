pub trait Read {
    type Bit;
    type Address;

    fn read(&self, address: Self::Address) -> Result<Self::Bit, MemoryError>;

    fn read_range(
        &self,
        start_address: Self::Address,
        end_offset: Self::Address,
    ) -> Result<&[Self::Bit], MemoryError>;
}

pub trait Write {
    type Bit;
    type Address;

    fn write(&mut self, address: Self::Address, data: Self::Bit) -> Result<(), MemoryError>;

    fn write_buf(
        &mut self,
        start_address: Self::Address,
        data: &[Self::Bit],
    ) -> Result<(), MemoryError>;
}

#[derive(Debug)]
pub enum MemoryError {
    OutOfBounds,
    InvalidRange,
    DoesNotExist,
    StackOverflow,
    StackUnderflow,
}
