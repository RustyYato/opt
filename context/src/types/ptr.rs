use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty, Type,
};

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerInfo<'ctx> {
    target_ty: Type<'ctx>,
}

impl core::fmt::Debug for PointerInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.target_ty.tag() {
            TypeTag::Unit
            | TypeTag::Integer
            | TypeTag::Pointer
            | TypeTag::Function
            | TypeTag::Struct => {
                write!(f, "*{:?}", self.target_ty)
            }
            TypeTag::Array => write!(f, "*({:?})", self.target_ty),
        }
    }
}

pub type Pointer<'ctx> = Ty<'ctx, PointerInfo<'ctx>>;

unsafe impl TypeInfo for PointerInfo<'_> {
    const TAG: TypeTag = TypeTag::Pointer;
}

impl<'ctx> Pointer<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>, target_ty: Type<'ctx>) -> Self {
        Ty::create_in_place(ctx, target_ty)
    }

    pub fn target(self) -> Type<'ctx> {
        self.info().target_ty
    }
}

impl<'ctx> Ctor<Type<'ctx>> for PointerInfo<'ctx> {
    #[inline]
    fn init<'a>(uninit: init::Uninit<'a, Self>, target_ty: Type<'ctx>) -> init::Init<'a, Self> {
        uninit.write(Self { target_ty })
    }
}

impl HasLayoutProvider<Type<'_>> for PointerInfo<'_> {
    type LayoutProvider = SizedLayoutProvider;
}
