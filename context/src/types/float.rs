use init::{
    layout_provider::{HasLayoutProvider, SizedLayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty,
};

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatKind {
    Ieee16Bit,
    Ieee32Bit,
    Ieee64Bit,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloatInfo {
    pub kind: FloatKind,
}

impl core::fmt::Debug for FloatInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            FloatKind::Ieee16Bit => write!(f, "f16"),
            FloatKind::Ieee32Bit => write!(f, "f32"),
            FloatKind::Ieee64Bit => write!(f, "f64"),
        }
    }
}

pub type FloatTy<'ctx> = Ty<'ctx, FloatInfo>;

unsafe impl TypeInfo for FloatInfo {
    const TAG: TypeTag = TypeTag::Integer;
    type Flags = ();
}

impl<'ctx> FloatTy<'ctx> {
    #[must_use]
    pub(crate) fn create(ctx: AllocContext<'ctx>, kind: FloatKind) -> Self {
        Ty::create_in_place(ctx, kind, ())
    }

    #[inline]
    pub fn kind(self) -> FloatKind {
        self.info().kind
    }
}

impl Ctor<FloatKind> for FloatInfo {
    #[inline]
    fn init(uninit: init::Uninit<'_, Self>, kind: FloatKind) -> init::Init<'_, Self> {
        uninit.write(Self { kind })
    }
}

impl HasLayoutProvider<FloatKind> for FloatInfo {
    type LayoutProvider = SizedLayoutProvider;
}
