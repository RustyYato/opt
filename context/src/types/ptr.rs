use std::hash::Hash;

use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    address_space::AddressSpace,
    raw_type::{TypeInfo, TypeTag},
    Ty,
};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerInfo {
    address_space: AddressSpace,
}

impl core::fmt::Debug for PointerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.address_space.is_default() {
            write!(f, "ptr")
        } else {
            write!(f, "ptr addressspace({})", self.address_space.get())
        }
    }
}

pub type PointerTy<'ctx> = Ty<'ctx, PointerInfo>;

unsafe impl<'ctx> TypeInfo<'ctx> for PointerInfo {
    const TAG: TypeTag = TypeTag::Pointer;
    type Flags = ();

    type Key<'a> = AddressSpace where 'ctx: 'a;

    #[inline]
    fn key<'a>(&'ctx self, (): Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a,
    {
        self.address_space
    }

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, key: Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a,
    {
        Ty::create_in_place(alloc, key, ())
    }
}

impl<'ctx> PointerTy<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>, address_space: AddressSpace) -> Self {
        Ty::create_in_place(ctx, address_space, ())
    }

    pub fn address_space(self) -> AddressSpace {
        self.info().address_space
    }
}

impl Ctor<AddressSpace> for PointerInfo {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, address_space: AddressSpace) -> init::Init<'_, Self> {
        uninit.write(Self { address_space })
    }
}

impl HasLayoutProvider<AddressSpace> for PointerInfo {
    type LayoutProvider = SizedLayoutProvider;
}
