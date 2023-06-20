// #![no_std]
#![forbid(
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc
)]

extern crate alloc;

mod bit_width;
mod ops;
mod pos;
mod repr;

pub use bit_width::{BitWidth, Kind};
pub use ops::{MismatchedBitWidth, Result};
pub use pos::{BitPos, DigitPos};
pub use repr::ApInt;
