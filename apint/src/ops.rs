use core::num::NonZeroUsize;

use crate::{ApInt, BitPos, DigitPos, Result};

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
}
