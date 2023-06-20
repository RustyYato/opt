use alloc::{alloc::*, boxed::Box};
use core::mem::MaybeUninit;

use crate::bit_width::{BitWidth, Kind, WORD_BYTES};

const _: [(); core::mem::size_of::<ApInt>()] = [(); 2 * core::mem::size_of::<usize>()];
pub struct ApInt {
    bit_width: BitWidth,
    data: ApIntData,
}

impl Drop for ApInt {
    fn drop(&mut self) {
        if self.bit_width.kind() != Kind::Outline {
            return;
        }

        let words =
            // SAFETY: the ApInt is in the outlined state
            unsafe { core::ptr::slice_from_raw_parts_mut(self.data.outlined, self.bit_width.words()) };

        // SAFETY: this ApInt was allocated with the global allocator
        unsafe {
            let _ = Box::from_raw(words);
        }
    }
}

union ApIntData {
    uninit: (),
    inline: usize,
    outlined: *mut usize,
}

#[repr(C)]
struct WordAlignedBytes<const N: usize> {
    word: [usize; 0],
    bytes: [u8; N],
}

#[non_exhaustive]
pub struct MismatchedBitWidth {}

pub type Result<T, E = MismatchedBitWidth> = core::result::Result<T, E>;

impl<const N: usize> WordAlignedBytes<N> {
    const N_DIV_WORD_BYTES: () = assert!(N % WORD_BYTES == 0);

    pub fn new(bytes: [u8; N]) -> Self {
        Self { word: [], bytes }
    }

    pub fn words(&self) -> &[usize] {
        #[allow(clippy::let_unit_value)]
        let () = Self::N_DIV_WORD_BYTES;

        let ptr = self.bytes.as_ptr().cast::<usize>();
        // SAFETY: this pointer is aligned to usize and cotains the correct number of bytes
        unsafe { core::slice::from_raw_parts(ptr, self.bytes.len() / WORD_BYTES) }
    }
}

impl ApInt {
    /// # Safety
    ///
    /// You must use `Self::maybe_uninit_words` to get the underlying words and every bit of all the words
    /// in the slice if `ZEROED` is false
    unsafe fn alloc<const ZEROED: bool>(bit_width: BitWidth) -> Self {
        Self {
            bit_width,
            data: match bit_width.kind() {
                Kind::Zero => ApIntData { uninit: () },
                Kind::Inline => {
                    if ZEROED {
                        ApIntData { inline: 0 }
                    } else {
                        ApIntData { uninit: () }
                    }
                }
                Kind::Outline => {
                    let layout = Layout::array::<usize>(bit_width.words()).unwrap();

                    let ptr = if ZEROED {
                        // SAFETY: This is safe because bit-width is non-zero, so the array is non-empty
                        unsafe { alloc_zeroed(layout) }
                    } else {
                        // SAFETY: This is safe because bit-width is non-zero, so the array is non-empty
                        unsafe { alloc(layout) }
                    };

                    if ptr.is_null() {
                        handle_alloc_error(layout)
                    }

                    ApIntData {
                        outlined: ptr.cast(),
                    }
                }
            },
        }
    }

    /// # Safety
    ///
    /// You may not write uninitialized memory to any of the words
    unsafe fn maybe_uninit_words(&mut self) -> &mut [MaybeUninit<usize>] {
        match self.bit_width.kind() {
            Kind::Zero => Default::default(),
            Kind::Inline => {
                // SAFETY:
                // * this this ApInt is in the inline state, so we can access that union field
                // * a `usize` has the same layout as `[MaybeUninit<usize>]`
                // * no code in this crate writes `uninit` to the words
                unsafe {
                    let ptr =
                        core::ptr::addr_of_mut!(self.data.inline).cast::<MaybeUninit<usize>>();
                    core::slice::from_mut(&mut *ptr)
                }
            }
            Kind::Outline => {
                // SAFETY:
                // * this this ApInt is in the outline state, so we can access that union field
                // * a `usize` has the same layout as `[MaybeUninit<usize>]`
                // * no code in this crate writes `uninit` to the words
                unsafe {
                    let data = self.data.outlined.cast::<MaybeUninit<usize>>();
                    core::slice::from_raw_parts_mut(data, self.bit_width.words())
                }
            }
        }
    }

    pub fn words(&self) -> &[usize] {
        match self.bit_width.kind() {
            Kind::Zero => Default::default(),
            Kind::Inline => {
                // SAFETY:
                // * this this ApInt is in the inline state, so we can access that union field
                // * a `usize` has the same layout as `[usize]`
                unsafe { core::slice::from_ref(&self.data.inline) }
            }
            Kind::Outline => {
                // SAFETY:
                // * this this ApInt is in the outline state, so we can access that union field
                // * a `usize` has the same layout as `[MaybeUninit<usize>]`
                // * no code in this crate writes `uninit` to the words
                unsafe { core::slice::from_raw_parts(self.data.outlined, self.bit_width.words()) }
            }
        }
    }

    pub fn words_mut(&mut self) -> &mut [usize] {
        match self.bit_width.kind() {
            Kind::Zero => Default::default(),
            Kind::Inline => {
                // SAFETY:
                // * this this ApInt is in the inline state, so we can access that union field
                // * a `usize` has the same layout as `[usize]`
                unsafe { core::slice::from_mut(&mut self.data.inline) }
            }
            Kind::Outline => {
                // SAFETY:
                // * this this ApInt is in the outline state, so we can access that union field
                // * a `usize` has the same layout as `[MaybeUninit<usize>]`
                // * no code in this crate writes `uninit` to the words
                unsafe {
                    core::slice::from_raw_parts_mut(self.data.outlined, self.bit_width.words())
                }
            }
        }
    }

    #[inline]
    pub fn bit_width(&self) -> BitWidth {
        self.bit_width
    }

    pub fn all_unset(bit_width: BitWidth) -> Self {
        // SAFETY: ZEROED is set true, so no safety condition
        unsafe { Self::alloc::<true>(bit_width) }
    }

    pub fn all_set(bit_width: BitWidth) -> Self {
        // SAFETY: All words are initialized in the loop below
        let mut apint = unsafe { Self::alloc::<false>(bit_width) };

        // SAFETY: no uninitialized bytes are written
        let words = unsafe { apint.maybe_uninit_words() };

        for word in words {
            *word = MaybeUninit::new(usize::MAX)
        }

        apint
    }

    fn from_bytes<const N: usize>(x: WordAlignedBytes<N>) -> Self {
        let bw = BitWidth::new(N * 8);
        let words = x.words();
        assert_eq!(words.len(), bw.words());

        // SAFETY: This only allocates 1 word, which is initialized below
        let mut ap_int = unsafe { Self::alloc::<false>(bw) };

        // SAFETY: No initialized bytes will be written to words
        let mu_words = unsafe { ap_int.maybe_uninit_words() };

        for (mu_word, word) in mu_words.iter_mut().zip(words) {
            *mu_word = MaybeUninit::new(*word)
        }

        ap_int
    }

    pub fn zero_sized() -> Self {
        // SAFETY: for bit-width zero, this function is safe
        unsafe { Self::alloc::<false>(BitWidth::BW_0) }
    }

    fn new_inline(width: BitWidth, x: usize) -> Self {
        assert!(matches!(width.kind(), Kind::Inline));
        // SAFETY: This only allocates 1 word, which is initialized below
        let mut apint = unsafe { Self::alloc::<false>(width) };

        apint.data = ApIntData { inline: x };

        apint
    }

    pub fn from_bool(x: bool) -> Self {
        Self::new_inline(BitWidth::BW_1, x as usize)
    }

    pub fn from_u8(x: u8) -> Self {
        Self::new_inline(BitWidth::BW_8, x as usize)
    }

    pub fn from_i8(x: i8) -> Self {
        Self::new_inline(BitWidth::BW_8, x as usize)
    }

    pub fn from_u16(x: u16) -> Self {
        Self::new_inline(BitWidth::BW_16, x as usize)
    }

    pub fn from_i16(x: i8) -> Self {
        Self::new_inline(BitWidth::BW_16, x as usize)
    }

    #[cfg(not(target_pointer_width = "16"))]
    pub fn from_u32(x: u32) -> Self {
        // if target pointer width is larger than 16, then it is >= 32
        // so can hold a u32 lossessly
        Self::new_inline(BitWidth::BW_32, x as usize)
    }

    #[cfg(target_pointer_width = "16")]
    pub fn from_u32(x: u32) -> Self {
        Self::from_bytes(WordAlignedBytes::new(x.to_le_bytes()))
    }

    pub fn from_i32(x: i32) -> Self {
        Self::new_inline(BitWidth::BW_32, x as usize)
    }

    #[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))]
    pub fn from_u64(x: u64) -> Self {
        // if target pointer width is larger than 32, then it is >= 64
        // so can hold a u64 lossessly
        Self::new_inline(BitWidth::BW_64, x as usize)
    }

    #[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
    pub fn from_u64(x: u64) -> Self {
        Self::from_bytes(WordAlignedBytes::new(x.to_le_bytes()))
    }

    pub fn from_i64(x: i64) -> Self {
        Self::new_inline(BitWidth::BW_64, x as usize)
    }

    pub fn from_u128(x: u128) -> Self {
        Self::from_bytes(WordAlignedBytes::new(x.to_le_bytes()))
    }

    pub fn from_i128(x: i128) -> Self {
        Self::from_u128(x as u128)
    }

    pub fn from_usize(x: usize) -> Self {
        Self::new_inline(BitWidth::BW_USIZE, x)
    }

    pub fn from_isize(x: isize) -> Self {
        Self::from_usize(x as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_aligned_bytes() {
        let x = WordAlignedBytes::new(102_u128.to_le_bytes());
        assert_eq!(x.words(), [102, 0]);
    }

    #[test]
    fn test_sign_bit() {
        let x = ApInt::from_u8(1 << 7);
        assert!(x.sign_bit());
    }

    #[test]
    fn test_get() {
        let x = ApInt::from_u8(1 << 7);
        assert!(x.get(7));
    }

    #[test]
    fn test_traling_zeros() {
        let x = ApInt::from_u8(1 << 7);
        assert_eq!(x.trailing_zeros(), 7);
        let x = ApInt::from_u128(1 << 64);
        assert_eq!(x.trailing_zeros(), 64);
    }

    #[test]
    fn test_traling_ones() {
        let x = ApInt::from_u8(!(1 << 7));
        assert_eq!(x.trailing_ones(), 7);
        let x = ApInt::from_u128(!(1 << 64));
        assert_eq!(x.trailing_ones(), 64);
    }

    #[test]
    fn test_leading_zeros() {
        let x = ApInt::from_u8(1 << 7);
        assert_eq!(x.leading_zeros(), (1u8 << 7).leading_zeros() as _);
        let x = ApInt::from_u128(1 << 64);
        assert_eq!(x.leading_zeros(), (1u128 << 64).leading_zeros() as _);
        let x = ApInt::from_u128(1 << 63);
        assert_eq!(x.leading_zeros(), (1u128 << 63).leading_zeros() as _);
        let x = ApInt::from_u128(0);
        assert_eq!(x.leading_zeros(), 0u128.leading_zeros() as _);
    }

    #[test]
    fn test_leading_ones() {
        let x = ApInt::from_u8(!(1 << 7));
        assert_eq!(x.leading_zeros(), (!(1u8 << 7)).leading_zeros() as _);
        let x = ApInt::from_u128(!(1 << 64));
        assert_eq!(x.leading_zeros(), (!(1u128 << 64)).leading_zeros() as _);
        let x = ApInt::from_u128(!(1 << 63));
        assert_eq!(x.leading_zeros(), (!(1u128 << 63)).leading_zeros() as _);
        let x = ApInt::from_u128(!0);
        assert_eq!(x.leading_zeros(), u128::MAX.leading_zeros() as _);
    }
}
