#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DigitPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitPos(usize);

impl BitPos {
    #[inline]
    pub fn of_pos(pos: usize) -> Self {
        Self(pos % usize::BITS as usize)
    }

    #[inline]
    pub fn mask(self) -> usize {
        1 << self.0
    }

    #[inline]
    pub fn get(self, word: usize) -> bool {
        word & self.mask() != 0
    }

    #[inline]
    pub fn set(self, word: usize) -> usize {
        word | self.mask()
    }

    #[inline]
    pub fn unset(self, word: usize) -> usize {
        word & !self.mask()
    }

    #[inline]
    pub fn flip(self, word: usize) -> usize {
        word ^ self.mask()
    }
}

impl DigitPos {
    #[inline]
    pub fn of_pos(pos: usize) -> usize {
        pos / usize::BITS as usize
    }
}
