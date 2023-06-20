pub const WORD_BYTES: usize = (usize::BITS / u8::BITS) as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BitWidth(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Zero,
    Inline,
    Outline,
}

impl BitWidth {
    pub const BW_0: Self = Self::new(0);
    pub const BW_1: Self = Self::new(1);
    pub const BW_8: Self = Self::new(8);
    pub const BW_16: Self = Self::new(16);
    pub const BW_32: Self = Self::new(32);
    pub const BW_64: Self = Self::new(64);
    pub const BW_128: Self = Self::new(128);
    pub const BW_USIZE: Self = Self::new(usize::BITS as usize);

    pub const fn new(x: usize) -> Self {
        Self(x)
    }

    #[inline]
    pub const fn get(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn kind(self) -> Kind {
        if self.get() == 0 {
            Kind::Zero
        } else if self.get() <= usize::BITS as usize {
            Kind::Inline
        } else {
            Kind::Outline
        }
    }

    #[inline]
    pub const fn words(self) -> usize {
        self.get() / usize::BITS as usize + (0 != self.get() % usize::BITS as usize) as usize
    }

    #[inline]
    pub const fn excess_bits(self) -> usize {
        match self.get() % usize::BITS as usize {
            0 => usize::BITS as usize,
            x => x,
        }
    }

    #[inline]
    pub fn excess_bits_mask(self) -> usize {
        match self.get() % usize::BITS as usize {
            0 => usize::MAX,
            x => !(usize::MAX << x),
        }
    }

    #[inline]
    pub const fn sign_bit(self) -> Option<usize> {
        self.get().checked_sub(1)
    }
}
