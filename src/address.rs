use crate::{RAM_SIZE, Word};

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Address(u8);

impl Address {
    pub fn new(byte: u8) -> Option<Self> {
        if byte as usize >= RAM_SIZE {
            return None;
        }

        Some(Address(byte))
    }

    pub fn from_word(word: Word) -> Self {
        Address(word & 0x0F)
    }

    pub fn next(self) -> Self {
        Address((self.0 + 1) & 0x0F)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_last_valid_address() {
        assert!(Address::new(16).is_none())
    }

    #[test]
    fn new_rejects_first_invalid_address() {
        assert_eq!(Address::new((15) as u8), Some(Address(15)));
    }

    #[test]
    fn next_wraps_around_after_15() {
        let address = Address(15).next();
        assert_eq!(address, Address(0))
    }
}
