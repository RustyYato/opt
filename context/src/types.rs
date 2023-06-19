mod type_cache;

mod address_space;

mod array;
mod float;
mod function;
mod int;
mod ptr;
mod raw_type;
mod struct_ty;
mod unit;

pub(crate) use type_cache::TypeCache;

pub use address_space::AddressSpace;

pub use array::ArrayTy;
pub use float::{FloatKind, FloatTy};
pub use function::FunctionTy;
pub use int::IntegerTy;
pub use ptr::PointerTy;
pub use raw_type::{Ty, Type};
pub use struct_ty::{StructFlags, StructTy};
pub use unit::UnitTy;

pub(crate) use array::{ArrayInfo, ArrayInit};
pub(crate) use function::{FunctionInfo, FunctionInit};
pub(crate) use int::IntegerInfo;
pub(crate) use ptr::PointerInfo;
pub(crate) use struct_ty::{StructInfo, StructInit};
