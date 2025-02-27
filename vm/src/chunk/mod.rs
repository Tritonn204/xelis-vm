mod reader;

use std::ops::{Deref, DerefMut};
use xelis_bytecode::Chunk;
use xelis_types::Path;
use super::{iterator::PathIterator, VMError};

pub use reader::ChunkReader;

// u16::MAX registers maximum
const REGISTERS_SIZE: usize = u16::MAX as usize;

// Manager for a chunk
// It contains the reader and the stacks
pub struct ChunkManager<'a> {
    reader: ChunkReader<'a>,
    // Registers are temporary and "scoped" per chunk
    registers: Vec<Path>,
    // Iterators stack
    iterators: Vec<PathIterator>,
}

impl<'a> ChunkManager<'a> {
    // Create a new chunk manager
    // It will create a reader from the chunk
    // and initialize the stack and registers
    #[inline]
    pub fn new(chunk: &'a Chunk) -> Self {
        ChunkManager {
            reader: ChunkReader::new(chunk),
            registers: Vec::new(),
            iterators: Vec::new(),
        }
    }

    // Get the registers
    #[inline]
    pub fn get_registers(&self) -> &Vec<Path> {
        &self.registers
    }

    // Add an iterator to the stack
    pub fn add_iterator(&mut self, iterator: PathIterator) {
        self.iterators.push(iterator);
    }

    // Pop an iterator from the stack
    pub fn pop_iterator(&mut self) -> Result<PathIterator, VMError> {
        self.iterators.pop().ok_or(VMError::EmptyIterator)
    }

    // Get the next value from the iterators stack
    pub fn next_iterator(&mut self) -> Result<Option<Path>, VMError> {
        Ok(self.iterators.last_mut()
            .ok_or(VMError::EmptyIterator)?
            .next()?)
    }

    // Push/set a new value into the registers
    #[inline]
    pub fn set_register(&mut self, index: usize, value: Path) -> Result<(), VMError> {
        if index >= REGISTERS_SIZE {
            return Err(VMError::RegisterMaxSize);
        }

        if self.registers.len() == index {
            self.registers.push(value);
        } else if self.registers.len() > index {
            self.registers[index] = value;
        } else {
            return Err(VMError::RegisterOverflow);
        }

        Ok(())
    }

    // Get a value from the registers
    #[inline]
    pub fn from_register(&mut self, index: usize) -> Result<&mut Path, VMError> {
        self.registers.get_mut(index).ok_or(VMError::RegisterNotFound)
    }

    // Pop a value from the registers
    #[inline]
    pub fn pop_register(&mut self) -> Result<Path, VMError> {
        self.registers.pop().ok_or(VMError::EmptyRegister)
    }
}

impl<'a> Deref for ChunkManager<'a> {
    type Target = ChunkReader<'a>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<'a> DerefMut for ChunkManager<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}
