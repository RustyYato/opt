use std::{
    alloc::Layout,
    ops::{BitAnd, BitOr},
};

use init::{
    layout_provider::{HasLayoutProvider, LayoutProvider},
    Ctor,
};

use crate::ctx::AllocContext;

use super::{
    raw_type::{TypeInfo, TypeTag},
    Ty, Type,
};

#[repr(C)]
#[non_exhaustive]
#[derive(PartialEq, Eq, Hash)]
pub struct StructInfo<'ctx> {
    name: Option<istr::IStr>,
    field_tys: [Type<'ctx>],
}

impl core::fmt::Debug for StructInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct FmtFields<'a, 'ctx>(&'a [Type<'ctx>]);

        impl core::fmt::Debug for FmtFields<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[")?;
                for (i, arg) in self.0.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?
                    }
                    write!(f, "{:?}", arg)?
                }
                write!(f, "]")
            }
        }

        if let Some(name) = self.name {
            write!(f, ".{name} ")?
        }

        write!(f, "{:?}", FmtFields(&self.field_tys))
    }
}

pub type StructTy<'ctx> = Ty<'ctx, StructInfo<'ctx>>;

unsafe impl<'ctx> TypeInfo<'ctx> for StructInfo<'ctx> {
    const TAG: TypeTag = TypeTag::Struct;
    type Flags = StructFlags;

    type Key<'a> = (Self::Flags, StructInit<'ctx, 'a>) where 'ctx: 'a;

    #[inline]
    fn key<'a>(&'ctx self, flags: Self::Flags) -> Self::Key<'a>
    where
        'ctx: 'a,
    {
        (
            flags,
            StructInit {
                name: self.name,
                fields: &self.field_tys,
            },
        )
    }

    fn create_from_key<'a>(alloc: AllocContext<'ctx>, (flags, key): Self::Key<'a>) -> Ty<'ctx, Self>
    where
        'ctx: 'a,
    {
        Ty::create_in_place(alloc, key, flags)
    }
}

#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructFlags(u16);

impl BitOr for StructFlags {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for StructFlags {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl StructFlags {
    #[inline(always)]
    pub fn any(self) -> bool {
        self.0 != 0
    }

    pub fn packed(self) -> bool {
        (self & StructTy::PACKED).any()
    }

    pub fn opaque(self) -> bool {
        (self & StructTy::OPAQUE).any()
    }

    pub fn literal(self) -> bool {
        (self & StructTy::LITERAL).any()
    }

    pub fn sized(self) -> bool {
        (self & StructTy::SIZED).any()
    }
}

impl<'ctx> StructTy<'ctx> {
    pub const PACKED: StructFlags = StructFlags(1 << 0);
    pub const OPAQUE: StructFlags = StructFlags(1 << 1);
    pub const LITERAL: StructFlags = StructFlags(1 << 2);
    pub const SIZED: StructFlags = StructFlags(1 << 3);

    #[inline]
    pub fn name(self) -> Option<istr::IStr> {
        self.info().name
    }

    #[inline]
    pub fn field_tys(self) -> &'ctx [Type<'ctx>] {
        &self.info().field_tys
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructInit<'ctx, 'a> {
    pub(crate) name: Option<istr::IStr>,
    pub(crate) fields: &'a [Type<'ctx>],
}

impl<'ctx> Ctor<StructInit<'ctx, '_>> for StructInfo<'ctx> {
    fn init<'a>(
        uninit: init::Uninit<'a, Self>,
        args: StructInit<'ctx, '_>,
    ) -> init::Init<'a, Self> {
        init::init_struct! {
            uninit => Self {
                name: init::ctor(|uninit| uninit.write(args.name)),
                field_tys: args.fields
            }
        }
    }
}

impl<'ctx> HasLayoutProvider<StructInit<'ctx, '_>> for StructInfo<'ctx> {
    type LayoutProvider = FunctionInfoLayoutProvider;
}

pub struct FunctionInfoLayoutProvider;

unsafe impl<'ctx> LayoutProvider<StructInfo<'ctx>, StructInit<'ctx, '_>>
    for FunctionInfoLayoutProvider
{
    fn layout_of(args: &StructInit<'ctx, '_>) -> Option<std::alloc::Layout> {
        Some(
            Layout::new::<Option<istr::IStr>>()
                .extend(Layout::array::<Type>(args.fields.len()).ok()?)
                .ok()?
                .0,
        )
    }

    unsafe fn cast(
        ptr: std::ptr::NonNull<u8>,
        args: &StructInit<'ctx, '_>,
    ) -> std::ptr::NonNull<StructInfo<'ctx>> {
        std::ptr::NonNull::from_raw_parts(ptr.cast(), args.fields.len())
    }
}
