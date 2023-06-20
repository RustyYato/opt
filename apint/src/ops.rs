use core::{cmp::Ordering, num::NonZeroUsize};

use crate::{ApInt, BitPos, DigitPos};

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub struct MismatchedBitWidth {}

pub type Result<T = (), E = MismatchedBitWidth> = core::result::Result<T, E>;

enum ZipWith<'a, 'b> {
    Empty,
    NonEmpty {
        self_last: usize,
        other_last: usize,
        self_words: &'a [usize],
        other_words: &'b [usize],
    },
}

enum ZipWithMut<'a, 'b> {
    Empty,
    NonEmpty {
        self_last: usize,
        other_last: usize,
        self_last_target: &'a mut usize,
        self_words: &'a mut [usize],
        other_words: &'b [usize],
    },
}

#[cfg(target_pointer_width = "16")]
type WideT = u32;

#[cfg(target_pointer_width = "32")]
type WideT = u64;

#[cfg(target_pointer_width = "64")]
type WideT = u128;

//
impl ApInt {
    pub fn unset_all(&mut self) {
        for word in self.words_mut() {
            *word = 0
        }
    }

    pub fn set_all(&mut self) {
        for word in self.words_mut() {
            *word = usize::MAX
        }
    }

    pub fn is_all_unset(&self) -> bool {
        for word in self.words() {
            if *word != 0 {
                return false;
            }
        }

        true
    }

    pub fn is_all_set(&self) -> bool {
        for word in self.words() {
            if *word != usize::MAX {
                return false;
            }
        }

        true
    }

    pub fn flip_all(&mut self) {
        for word in self.words_mut() {
            *word ^= usize::MAX
        }
    }

    pub fn get(&self, pos: usize) -> bool {
        let digit = DigitPos::of_pos(pos);
        let bit = BitPos::of_pos(pos);
        bit.get(self.words()[digit])
    }

    fn with_word(&mut self, pos: usize, f: impl FnOnce(&mut usize, BitPos)) {
        let digit = DigitPos::of_pos(pos);
        let bit = BitPos::of_pos(pos);
        let word = &mut self.words_mut()[digit];
        f(word, bit)
    }

    pub fn set_bit_at(&mut self, pos: usize) {
        self.with_word(pos, |word, pos| *word = pos.set(*word))
    }

    pub fn unset_bit_at(&mut self, pos: usize) {
        self.with_word(pos, |word, pos| *word = pos.unset(*word))
    }

    pub fn flip_bit_at(&mut self, pos: usize) {
        self.with_word(pos, |word, pos| *word = pos.flip(*word))
    }

    pub fn sign_bit(&self) -> bool {
        if let Some(sign_bit) = self.bit_width().sign_bit() {
            self.get(sign_bit)
        } else {
            false
        }
    }

    pub fn set_sign_bit(&mut self) {
        let sign_bit = self
            .bit_width()
            .sign_bit()
            .expect("Cannot modify sign bit of zero-sized apint");
        self.set_bit_at(sign_bit);
    }

    pub fn unset_sign_bit(&mut self) {
        let sign_bit = self
            .bit_width()
            .sign_bit()
            .expect("Cannot modify sign bit of zero-sized apint");
        self.unset_bit_at(sign_bit);
    }

    pub fn flip_sign_bit(&mut self) {
        let sign_bit = self
            .bit_width()
            .sign_bit()
            .expect("Cannot modify sign bit of zero-sized apint");
        self.flip_bit_at(sign_bit);
    }

    pub fn count_zeros(&self) -> usize {
        self.words().iter().map(|x| x.count_zeros() as usize).sum()
    }

    pub fn count_ones(&self) -> usize {
        self.words().iter().map(|x| x.count_ones() as usize).sum()
    }

    pub fn trailing_zeros(&self) -> usize {
        if let [words @ .., last] = self.words() {
            for (i, word) in words.iter().enumerate() {
                let word = NonZeroUsize::new(*word);

                if let Some(word) = word {
                    return i * usize::BITS as usize + word.trailing_zeros() as usize;
                }
            }

            words.len() * usize::BITS as usize
                + (last.trailing_zeros() as usize).min(self.bit_width().excess_bits())
        } else {
            0
        }
    }

    pub fn trailing_ones(&self) -> usize {
        if let [words @ .., last] = self.words() {
            for (i, word) in words.iter().enumerate() {
                let word = NonZeroUsize::new(!*word);

                if let Some(word) = word {
                    return i * usize::BITS as usize + word.trailing_zeros() as usize;
                }
            }

            let last = !*last;

            words.len() * usize::BITS as usize
                + (last.trailing_zeros() as usize).min(self.bit_width().excess_bits())
        } else {
            0
        }
    }

    pub fn leading_zeros(&self) -> usize {
        if let [words @ .., last] = self.words() {
            let last = last & self.bit_width().excess_bits_mask();
            let last = NonZeroUsize::new(last);

            if let Some(last) = last {
                return last.leading_zeros() as usize
                    - (usize::BITS as usize - self.bit_width().excess_bits());
            }

            for (i, word) in words.iter().rev().enumerate() {
                let word = NonZeroUsize::new(*word);

                if let Some(word) = word {
                    return i * usize::BITS as usize
                        + word.leading_zeros() as usize
                        + self.bit_width().excess_bits();
                }
            }

            usize::BITS as usize * words.len() + self.bit_width().excess_bits()
        } else {
            0
        }
    }

    pub fn leading_ones(&self) -> usize {
        if let [words @ .., last] = self.words() {
            let last = (!last) & self.bit_width().excess_bits_mask();
            let last = NonZeroUsize::new(last);

            if let Some(last) = last {
                return last.leading_zeros() as usize
                    - (usize::BITS as usize - self.bit_width().excess_bits());
            }

            for (i, word) in words.iter().rev().enumerate() {
                let word = NonZeroUsize::new(!*word);

                if let Some(word) = word {
                    return i * usize::BITS as usize
                        + word.leading_zeros() as usize
                        + self.bit_width().excess_bits();
                }
            }

            usize::BITS as usize * words.len() + self.bit_width().excess_bits()
        } else {
            0
        }
    }

    fn zip_with<'a, 'b>(&'a self, other: &'b Self) -> Result<ZipWith<'a, 'b>> {
        if self.bit_width() == other.bit_width() {
            Ok(match (self.words(), other.words()) {
                ([], []) => ZipWith::Empty,
                ([self_words @ .., self_last], [other_words @ .., other_last]) => {
                    ZipWith::NonEmpty {
                        self_last: self_last & self.bit_width().excess_bits_mask(),
                        other_last: other_last & self.bit_width().excess_bits_mask(),
                        self_words,
                        other_words,
                    }
                }
                _ => unreachable!(),
            })
        } else {
            Err(MismatchedBitWidth {})
        }
    }

    fn zip_with_mut<'a, 'b>(&'a mut self, other: &'b Self) -> Result<ZipWithMut<'a, 'b>> {
        if self.bit_width() == other.bit_width() {
            Ok(match (self.words_mut(), other.words()) {
                ([], []) => ZipWithMut::Empty,
                ([self_words @ .., self_last], [other_words @ .., other_last]) => {
                    ZipWithMut::NonEmpty {
                        self_last: *self_last & other.bit_width().excess_bits_mask(),
                        other_last: *other_last & other.bit_width().excess_bits_mask(),
                        self_last_target: self_last,
                        self_words,
                        other_words,
                    }
                }
                _ => unreachable!(),
            })
        } else {
            Err(MismatchedBitWidth {})
        }
    }

    pub fn unsigned_cmp(&self, other: &Self) -> Result<Ordering> {
        match self.zip_with(other)? {
            ZipWith::Empty => (),
            ZipWith::NonEmpty {
                self_last,
                other_last,
                self_words,
                other_words,
            } => {
                match self_last.cmp(&other_last) {
                    Ordering::Less => return Ok(Ordering::Less),
                    Ordering::Equal => (),
                    Ordering::Greater => return Ok(Ordering::Greater),
                }

                for (s, o) in self_words.iter().zip(other_words).rev() {
                    match s.cmp(o) {
                        Ordering::Less => return Ok(Ordering::Less),
                        Ordering::Equal => (),
                        Ordering::Greater => return Ok(Ordering::Greater),
                    }
                }
            }
        }

        Ok(Ordering::Equal)
    }

    pub fn signed_cmp(&self, other: &Self) -> Result<Ordering> {
        match self.zip_with(other)? {
            ZipWith::Empty => (),
            ZipWith::NonEmpty {
                self_last,
                other_last,
                self_words,
                other_words,
            } => {
                let shift_bits = usize::BITS - self.bit_width().excess_bits() as u32;
                let self_last = (self_last << shift_bits) as isize;
                let other_last = (other_last << shift_bits) as isize;

                match self_last.cmp(&other_last) {
                    Ordering::Less => return Ok(Ordering::Less),
                    Ordering::Equal => (),
                    Ordering::Greater => return Ok(Ordering::Greater),
                }

                for (s, o) in self_words.iter().zip(other_words).rev() {
                    match s.cmp(o) {
                        Ordering::Less => return Ok(Ordering::Less),
                        Ordering::Equal => (),
                        Ordering::Greater => return Ok(Ordering::Greater),
                    }
                }
            }
        }

        Ok(Ordering::Equal)
    }

    pub fn add_unsigned_word(&mut self, x: usize) {
        if let [first, words @ ..] = self.words_mut() {
            let (a, carry) = first.overflowing_add(x);
            *first = a;

            if !carry {
                return;
            }

            for word in words {
                let (a, carry) = word.overflowing_add(1);
                *word = a;
                if !carry {
                    return;
                }
            }
        }
    }

    pub fn bitnot(&mut self) {
        self.flip_all()
    }

    pub fn negate(&mut self) {
        // we store in two's complement, so -x == !x + 1
        self.bitnot();
        self.add_unsigned_word(1);
    }

    pub fn into_add(mut self, other: &Self) -> Result<Self> {
        self.add_assign(other)?;
        Ok(self)
    }

    pub fn add_assign(&mut self, other: &Self) -> Result<bool> {
        let ovf = match self.zip_with_mut(other)? {
            ZipWithMut::Empty => false,
            ZipWithMut::NonEmpty {
                self_last: _,
                other_last,
                self_last_target,
                self_words,
                other_words,
            } => {
                let mut carry = false;

                for (self_word, other_word) in self_words.iter_mut().zip(other_words) {
                    let (word, a) = self_word.overflowing_add(*other_word);
                    let (word, b) = word.overflowing_add(carry as usize);
                    *self_word = word;
                    carry = a || b
                }

                let (word, a) = self_last_target.overflowing_add(other_last);
                let (word, b) = word.overflowing_add(carry as usize);
                *self_last_target = word;
                a || b
            }
        };

        Ok(ovf)
    }

    pub fn into_sub(mut self, other: &Self) -> Result<Self> {
        self.sub_assign(other)?;
        Ok(self)
    }

    pub fn sub_assign(&mut self, other: &Self) -> Result<bool> {
        let ovf = match self.zip_with_mut(other)? {
            ZipWithMut::Empty => false,
            ZipWithMut::NonEmpty {
                self_last: _,
                other_last,
                self_last_target,
                self_words,
                other_words,
            } => {
                let mut carry = false;

                for (self_word, other_word) in self_words.iter_mut().zip(other_words) {
                    let (word, a) = self_word.overflowing_sub(*other_word);
                    let (word, b) = word.overflowing_sub(carry as usize);
                    *self_word = word;
                    carry = a || b
                }

                let (word, a) = self_last_target.overflowing_sub(other_last);
                let (word, b) = word.overflowing_sub(carry as usize);
                *self_last_target = word;
                a || b
            }
        };

        Ok(ovf)
    }
}

impl core::ops::Add<&ApInt> for ApInt {
    type Output = Self;

    fn add(self, rhs: &ApInt) -> Self::Output {
        self.into_add(rhs)
            .expect("Cannot add ApInts of different bit-widths")
    }
}

impl core::ops::Sub<&ApInt> for ApInt {
    type Output = Self;

    fn sub(self, rhs: &ApInt) -> Self::Output {
        self.into_sub(rhs)
            .expect("Cannot add ApInts of different bit-widths")
    }
}

impl Eq for ApInt {}
impl PartialEq for ApInt {
    fn eq(&self, other: &Self) -> bool {
        self.unsigned_cmp(other) == Ok(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_cmp() {
        let a = ApInt::from_u8(!(1 << 7));
        let b = ApInt::from_u8(1 << 7);
        assert_eq!(a.unsigned_cmp(&b), Ok(Ordering::Less));
        let a = ApInt::from_u128(1 << 80);
        let b = ApInt::from_u128(1 << 74);
        assert_eq!(a.unsigned_cmp(&b), Ok(Ordering::Greater));
        let a = ApInt::from_u128(1 << 127);
        let b = ApInt::from_u128(1 << 74);
        assert_eq!(a.unsigned_cmp(&b), Ok(Ordering::Greater));
    }

    #[test]
    fn test_signed_cmp() {
        let a = ApInt::from_u8(!(1 << 7));
        let b = ApInt::from_u8(1 << 7);
        assert_eq!(a.signed_cmp(&b), Ok(Ordering::Greater));
        let a = ApInt::from_u128(1 << 80);
        let b = ApInt::from_u128(1 << 74);
        assert_eq!(a.signed_cmp(&b), Ok(Ordering::Greater));
        let a = ApInt::from_u128(1 << 127);
        let b = ApInt::from_u128(1 << 74);
        assert_eq!(a.signed_cmp(&b), Ok(Ordering::Less));
    }

    #[test]
    fn test_add() {
        let a = ApInt::from_u128(10 | (80 << 64));
        let b = ApInt::from_u128(50 | (30 << 64));
        let c = ApInt::from_u128(60 | (110 << 64));
        assert_eq!(a + &b, c);

        let a = ApInt::from_u128(10 | (1 << 63) | (80 << 64));
        let b = ApInt::from_u128(50 | (1 << 63) | (30 << 64));
        let c = ApInt::from_u128(60 | (111 << 64));
        assert_eq!(a + &b, c);
    }

    #[test]
    fn test_sub() {
        let a = ApInt::from_u128(50 | (80 << 64));
        let b = ApInt::from_u128(10 | (30 << 64));
        let c = ApInt::from_u128(40 | (50 << 64));
        assert_eq!(a - &b, c);

        let a = ApInt::from_u128(10 | (1 << 63) | (80 << 64));
        let b = ApInt::from_u128(50 | (1 << 63) | (30 << 64));
        let c = ApInt::from_u128((10 | (1 << 63) | (80 << 64)) - (50 | (1 << 63) | (30 << 64)));
        assert_eq!(a - &b, c);
    }
}
