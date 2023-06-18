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

pub type Unit<'ctx> = Ty<'ctx, UnitInfo>;

unsafe impl TypeInfo for UnitInfo {
    const TAG: TypeTag = TypeTag::Unit;
    type Flags = ();
}

impl<'ctx> Unit<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>) -> Self {
        Ty::create_in_place(ctx, (), ())
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
