use std::hash::Hash;

use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};
use rug::integer::BorrowInteger;

use crate::{types::IntegerTy, AllocContext};

#[derive(Debug, Clone, Copy)]
pub struct ConstIntInfo<'ctx> {
    value: BorrowInteger<'ctx>,
}

impl Eq for ConstIntInfo<'_> {}
impl PartialEq for ConstIntInfo<'_> {
    fn eq(&self, other: &Self) -> bool {
        *self.value == *other.value
    }
}

impl Hash for ConstIntInfo<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

pub type ConstInt<'ctx> = super::Val<'ctx, ConstIntInfo<'ctx>>;

unsafe impl<'ctx> super::ValueInfo for ConstIntInfo<'ctx> {
    const TAG: super::ValueTag = super::ValueTag::ConstInt;
    type Flags = bool;
}

impl<'ctx> ConstInt<'ctx> {
    #[allow(clippy::new_ret_no_self)]
    pub(crate) fn new(
        ctx: AllocContext<'ctx>,
        ty: IntegerTy<'ctx>,
        value: BorrowInteger<'ctx>,
        signed: bool,
    ) -> ConstInt<'ctx> {
        Self::create_in_place(ctx, ty.erase(), value, signed)
    }

    #[inline]
    pub fn is_signed(&self) -> bool {
        self.flags()
    }

    #[inline]
    pub fn value(&self) -> BorrowInteger<'ctx> {
        self.info().value
    }
}

impl<'ctx> Ctor<BorrowInteger<'ctx>> for ConstIntInfo<'ctx> {
    fn init<'a>(
        uninit: init::Uninit<'a, Self>,
        value: BorrowInteger<'ctx>,
    ) -> init::Init<'a, Self> {
        uninit.write(Self { value })
    }
}

impl<'ctx> HasLayoutProvider<BorrowInteger<'ctx>> for ConstIntInfo<'ctx> {
    type LayoutProvider = SizedLayoutProvider;
}
