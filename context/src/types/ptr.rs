use std::hash::Hash;

use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty,
};

#[derive(Clone, Copy, Eq)]
#[allow(non_camel_case_types)]
struct u24(u8, u8, u8);

impl u24 {
    fn to_u32(self) -> u32 {
        u32::from_ne_bytes([0, self.0, self.1, self.2])
    }
}

impl PartialEq for u24 {
    fn eq(&self, other: &Self) -> bool {
        self.to_u32() == other.to_u32()
    }
}

impl Hash for u24 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_u32().hash(state)
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerInfo {
    address_space: u24,
}

impl core::fmt::Debug for PointerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.address_space.to_u32() == 0 {
            write!(f, "ptr")
        } else {
            write!(f, "ptr addressspace({})", self.address_space.to_u32())
        }
    }
}

pub type Pointer<'ctx> = Ty<'ctx, PointerInfo>;

unsafe impl TypeInfo for PointerInfo {
    const TAG: TypeTag = TypeTag::Pointer;
    type Flags = ();
}

impl<'ctx> Pointer<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>, address_space: u32) -> Self {
        assert_eq!(address_space & 0xff000000, 0);
        let [a, b, c, _] = u32::to_le_bytes(address_space);
        Ty::create_in_place(ctx, u24(a, b, c), ())
    }

    pub fn address_space(self) -> u32 {
        self.info().address_space.to_u32()
    }
}

impl Ctor<u24> for PointerInfo {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, address_space: u24) -> init::Init<'_, Self> {
        uninit.write(Self { address_space })
    }
}

impl HasLayoutProvider<u24> for PointerInfo {
    type LayoutProvider = SizedLayoutProvider;
}
