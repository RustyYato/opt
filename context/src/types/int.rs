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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerInfo {
    pub bits: NonZeroU16,
}

impl core::fmt::Debug for IntegerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i{}", self.bits)
    }
}

pub type IntegerTy<'ctx> = Ty<'ctx, IntegerInfo>;

unsafe impl<'ctx> TypeInfo<'ctx> for IntegerInfo {
    const TAG: TypeTag = TypeTag::Integer;
    type Flags = ();

    type Key<'a> = NonZeroU16 where 'ctx: 'a;

    #[inline]
    fn key<'a>(&'ctx self, (): Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a,
    {
        self.bits
    }

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, key: Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a,
    {
        IntegerTy::create(alloc, key)
    }
}

impl<'ctx> IntegerTy<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>, bits: NonZeroU16) -> Self {
        Ty::create_in_place(ctx, bits, ())
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
