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
pub struct ArrayInfo<'ctx> {
    item_ty: Type<'ctx>,
    // use MinAlignU64 to reduce size on 16/32-bit architectures
    len: MinAlignLen,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(target_pointer_width = "16", repr(packed, align(2)))]
#[cfg_attr(target_pointer_width = "32", repr(packed, align(4)))]
struct MinAlignLen {
    value: u64,
}

impl core::fmt::Debug for ArrayInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len.value;
        write!(f, "{:?}\u{d7}{}", self.item_ty, len)
    }
}

pub type ArrayTy<'ctx> = Ty<'ctx, ArrayInfo<'ctx>>;

unsafe impl<'ctx> TypeInfo<'ctx> for ArrayInfo<'ctx> {
    const TAG: TypeTag = TypeTag::Array;
    type Flags = ();

    type Key<'a> = ArrayInit<'ctx> where 'ctx: 'a;

    #[inline]
    fn key<'a>(&'ctx self, (): Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a,
    {
        ArrayInit {
            item_ty: self.item_ty,
            len: self.len.value,
        }
    }

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, key: Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a,
    {
        Ty::create_in_place(alloc, key, ())
    }
}

impl<'ctx> ArrayTy<'ctx> {
    pub fn item_ty(self) -> Type<'ctx> {
        self.info().item_ty
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> u64 {
        self.info().len.value
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayInit<'ctx> {
    pub item_ty: Type<'ctx>,
    pub len: u64,
}

impl<'ctx> Ctor<ArrayInit<'ctx>> for ArrayInfo<'ctx> {
    #[inline]
    fn init<'a>(uninit: init::Uninit<'a, Self>, init: ArrayInit<'ctx>) -> init::Init<'a, Self> {
        uninit.write(Self {
            item_ty: init.item_ty,
            len: MinAlignLen { value: init.len },
        })
    }
}

impl HasLayoutProvider<ArrayInit<'_>> for ArrayInfo<'_> {
    type LayoutProvider = SizedLayoutProvider;
}
