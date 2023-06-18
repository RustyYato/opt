mod address_space;

mod array;
mod float;
mod function;
mod int;
mod ptr;
mod raw_type;
mod struct_ty;
mod unit;

pub use address_space::AddressSpace;

pub use array::Array;
pub use float::{Float, FloatKind};
pub use function::Function;
pub use int::Integer;
pub use ptr::Pointer;
pub use raw_type::{Ty, Type};
pub use struct_ty::{Struct, StructFlags};
pub use unit::Unit;
