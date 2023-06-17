use std::num::NonZeroU16;

use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty,
};

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct IntegerInfo {
    pub bits: NonZeroU16,
}

pub type Integer<'ctx> = Ty<'ctx, IntegerInfo>;

unsafe impl TypeInfo for IntegerInfo {
    const TAG: TypeTag = TypeTag::Integer;
}

impl<'ctx> Integer<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>, bits: NonZeroU16) -> Self {
        Ty::create_in_place(ctx, bits)
    }

    #[inline]
    pub fn bits(self) -> NonZeroU16 {
        self.info().bits
    }
}

impl Ctor<NonZeroU16> for IntegerInfo {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, bits: NonZeroU16) -> init::Init<'_, Self> {
        uninit.write(Self { bits })
    }
}

impl HasLayoutProvider<NonZeroU16> for IntegerInfo {
    type LayoutProvider = SizedLayoutProvider;
}