use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitInfo;

impl core::fmt::Debug for UnitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unit")
    }
}

pub type UnitTy<'ctx> = Ty<'ctx, UnitInfo>;

unsafe impl<'ctx> TypeInfo<'ctx> for UnitInfo {
    const TAG: TypeTag = TypeTag::Unit;
    type Flags = ();

    type Key<'a> = ()
    where
        'ctx: 'a;

    fn key<'a>(&'ctx self, (): Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a,
    {
    }

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, (): Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a,
    {
        Ty::create_in_place(alloc, (), ())
    }
}

impl<'ctx> UnitTy<'ctx> {
    #[must_use]
    pub(crate) fn create(alloc: AllocContext<'ctx>) -> Self {
        Ty::create_in_place(alloc, (), ())
    }
}

impl Ctor for UnitInfo {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, (): ()) -> init::Init<'_, Self> {
        uninit.write(Self)
    }
}

impl HasLayoutProvider for UnitInfo {
    type LayoutProvider = SizedLayoutProvider;
}
